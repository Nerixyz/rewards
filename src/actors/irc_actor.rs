use crate::actors::db_actor::DbActor;
use crate::actors::messages::db_messages::{GetToken, SaveToken};
use crate::actors::messages::irc_messages::{ChatMessage, JoinAllMessage, JoinMessage, PartMessage, TimedModeMessage, TimeoutMessage, WhisperMessage, SayMessage};
use crate::constants::{TWITCH_CLIENT_ID, TWITCH_CLIENT_SECRET, TWITCH_CLIENT_USER_LOGIN};
use actix::{Actor, Addr, Context, Handler, ResponseFuture};
use anyhow::Error as AnyError;
use async_trait::async_trait;
use std::fmt::{Debug, Formatter};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::watch::{channel, Receiver, Sender};
use tokio::task;
use tokio::task::JoinHandle;
use twitch_irc::login::{RefreshingLoginCredentials, TokenStorage, UserAccessToken};
use twitch_irc::message::{NoticeMessage, ServerMessage};
use twitch_irc::{ClientConfig, TCPTransport, TwitchIRCClient};

type IrcCredentials = RefreshingLoginCredentials<PgTokenStorage>;
type IrcClient = TwitchIRCClient<TCPTransport, IrcCredentials>;

struct PgTokenStorage(Addr<DbActor>);

impl Debug for PgTokenStorage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("PgTokenStorage")
    }
}

#[async_trait]
impl TokenStorage for PgTokenStorage {
    type LoadError = AnyError;
    type UpdateError = AnyError;

    async fn load_token(&mut self) -> Result<UserAccessToken, Self::LoadError> {
        self.0.send(GetToken {}).await?.map_err(AnyError::new)
    }

    async fn update_token(&mut self, token: &UserAccessToken) -> Result<(), Self::UpdateError> {
        self.0
            .send(SaveToken(UserAccessToken {
                refresh_token: token.refresh_token.clone(),
                access_token: token.access_token.clone(),
                created_at: token.created_at,
                expires_at: token.expires_at,
            }))
            .await?
            .map_err(AnyError::new)
    }
}

pub struct IrcActor {
    client: IrcClient,
    incoming: Option<UnboundedReceiver<ServerMessage>>,
    listener: Option<JoinHandle<()>>,
    notice_tx: Option<Sender<Option<NoticeMessage>>>,
    notice_rx: Receiver<Option<NoticeMessage>>,
}

impl IrcActor {
    pub fn new(db: Addr<DbActor>) -> Self {
        let config = ClientConfig::new_simple(IrcCredentials::new(
            TWITCH_CLIENT_USER_LOGIN.to_string(),
            TWITCH_CLIENT_ID.to_string(),
            TWITCH_CLIENT_SECRET.to_string(),
            PgTokenStorage(db),
        ));

        let (incoming, client) = IrcClient::new(config);
        let (notice_tx, notice_rx) = channel(None);

        Self {
            listener: None,
            incoming: Some(incoming),
            client,
            notice_tx: Some(notice_tx),
            notice_rx,
        }
    }
}

impl Actor for IrcActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        let mut incoming = self.incoming.take().expect("This was set in Self:new");
        let tx = self.notice_tx.take().expect("This was set in Self::new");

        self.listener = Some(task::spawn(async move {
            while let Some(message) = incoming.recv().await {
                match message {
                    ServerMessage::Notice(notice) => tx.send(Some(notice)).ok(),
                    _ => None,
                };
            }
        }));
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        if let Some(listener) = &self.listener {
            listener.abort();
        }
    }
}

impl Handler<JoinMessage> for IrcActor {
    type Result = ();

    fn handle(&mut self, msg: JoinMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.client.join(msg.0)
    }
}

impl Handler<PartMessage> for IrcActor {
    type Result = ();

    fn handle(&mut self, msg: PartMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.client.part(msg.0)
    }
}

impl Handler<JoinAllMessage> for IrcActor {
    type Result = ();

    fn handle(&mut self, msg: JoinAllMessage, _ctx: &mut Self::Context) -> Self::Result {
        for channel in msg.0 {
            self.client.join(channel)
        }
    }
}

impl Handler<ChatMessage> for IrcActor {
    type Result = ();
    // do nothing for now
    fn handle(&mut self, _msg: ChatMessage, _ctx: &mut Self::Context) -> Self::Result {
        // println!("{:?}", msg.0);
    }
}

impl Handler<WhisperMessage> for IrcActor {
    type Result = ResponseFuture<Result<(), AnyError>>;

    fn handle(&mut self, msg: WhisperMessage, _ctx: &mut Self::Context) -> Self::Result {
        let client = self.client.clone();
        Box::pin(async move {
            Ok(client
                .privmsg(
                    TWITCH_CLIENT_USER_LOGIN.to_string(),
                    format!("/w {} {}", msg.0, msg.1),
                )
                .await?)
        })
    }
}

impl Handler<SayMessage> for IrcActor {
    type Result = ResponseFuture<Result<(), AnyError>>;

    fn handle(&mut self, msg: SayMessage, _ctx: &mut Self::Context) -> Self::Result {
        let client = self.client.clone();
        log::info!("Send message login={}; message={}", msg.0, msg.1);
        Box::pin(async move { Ok(client.say(msg.0, msg.1).await?) })
    }
}

impl Handler<TimeoutMessage> for IrcActor {
    type Result = ResponseFuture<Result<(), AnyError>>;

    fn handle(&mut self, msg: TimeoutMessage, _ctx: &mut Self::Context) -> Self::Result {
        let client = self.client.clone();
        let mut rx = self.notice_rx.clone();
        Box::pin(async move {
            log::info!(
                "timeout in {} for {} for {:?}s",
                msg.broadcaster,
                msg.user,
                msg.duration
            );
            client
                .privmsg(
                    msg.broadcaster.clone(),
                    format!("/timeout {} {}", msg.user, msg.duration),
                )
                .await?;

            while let Ok(Ok(_)) = tokio::time::timeout(Duration::from_secs(5), rx.changed()).await {
                let value = (*rx.borrow()).clone();
                let notice = match value {
                    Some(v) => v,
                    None => continue,
                };
                if Some(&msg.broadcaster) != notice.channel_login.as_ref() {
                    continue;
                }

                if let Some(id) = &notice.message_id {
                    match id.as_str() {
                        "timeout_success" => return Ok(()),
                        "usage_timeout"
                        | "timeout_no_timeout"
                        | "invalid_user"
                        | "bad_timeout_duration" => return Err(AnyError::msg(id.clone())),
                        _ => {
                            if id.starts_with("bad_timeout") {
                                return Err(AnyError::msg(id.clone()));
                            }
                        }
                    }
                }
            }

            Err(AnyError::msg("No reply"))
        })
    }
}

impl Handler<TimedModeMessage> for IrcActor {
    type Result = ();

    fn handle(&mut self, msg: TimedModeMessage, _ctx: &mut Self::Context) -> Self::Result {
        let client = self.client.clone();
        task::spawn(async move {
            log::info!(
                "{} in {} for {:?}s",
                msg.mode,
                msg.broadcaster,
                msg.duration
            );
            if let Err(e) = client
                .privmsg(msg.broadcaster.clone(), format!("/{}", msg.mode))
                .await
            {
                println!("Could not enter {}: {:?}", msg.mode, e);
            }
            tokio::time::sleep(std::time::Duration::from_secs(msg.duration)).await;

            log::info!("Leave {} in {}", msg.mode, msg.broadcaster);
            if let Err(e) = client
                .privmsg(msg.broadcaster, format!("/{}off", msg.mode))
                .await
            {
                println!("Could not leave {}: {:?}", msg.mode, e);
            }
        });
    }
}
