use crate::actors::db_actor::DbActor;
use crate::actors::messages::db_messages::{GetToken, SaveToken};
use crate::actors::messages::irc_messages::{ChatMessage, JoinAllMessage, JoinMessage, TimeoutMessage, TimedModeMessage};
use crate::constants::{TWITCH_CLIENT_ID, TWITCH_CLIENT_SECRET, TWITCH_CLIENT_USER_LOGIN};
use actix::{Actor, Addr, AsyncContext, Context, Handler, ResponseFuture};
use anyhow::Error as AnyError;
use async_trait::async_trait;
use std::fmt::{Debug, Formatter};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task;
use tokio::task::JoinHandle;
use twitch_irc::login::{RefreshingLoginCredentials, TokenStorage, UserAccessToken};
use twitch_irc::message::ServerMessage;
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
        self.0
            .send(GetToken {})
            .await?
            .map_err(|e| AnyError::new(e))
    }

    async fn update_token(&mut self, token: &UserAccessToken) -> Result<(), Self::UpdateError> {
        self.0
            .send(SaveToken(UserAccessToken {
                refresh_token: token.refresh_token.clone(),
                access_token: token.access_token.clone(),
                created_at: token.created_at.clone(),
                expires_at: token.expires_at.clone(),
            }))
            .await?
            .map_err(|e| AnyError::new(e))
    }
}

pub struct IrcActor {
    client: IrcClient,
    incoming: Option<UnboundedReceiver<ServerMessage>>,
    listener: Option<JoinHandle<()>>,
}

impl IrcActor {
    pub fn new(db: Addr<DbActor>) -> Self {
        let config = ClientConfig::new_simple(IrcCredentials::new(
            TWITCH_CLIENT_USER_LOGIN.to_string(),
            TWITCH_CLIENT_ID.to_string(),
            TWITCH_CLIENT_SECRET.to_string(),
            PgTokenStorage(db.clone()),
        ));

        let (incoming, client) = IrcClient::new(config);

        Self {
            listener: None,
            incoming: Some(incoming),
            client,
        }
    }
}

impl Actor for IrcActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let mut incoming = self.incoming.take().expect("This was set in Self:new");

        let addr = ctx.address().clone();
        self.listener = Some(task::spawn(async move {
            while let Some(message) = incoming.recv().await {
                if let ServerMessage::Privmsg(msg) = message {
                    addr.do_send(ChatMessage(msg));
                }
            }
        }));
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        // this won't be called but should be implemented probably
        todo!()
    }
}

impl Handler<JoinMessage> for IrcActor {
    type Result = ();

    fn handle(&mut self, msg: JoinMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.client.join(msg.0)
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

impl Handler<TimeoutMessage> for IrcActor {
    type Result = ResponseFuture<Result<(), AnyError>>;

    fn handle(&mut self, msg: TimeoutMessage, _ctx: &mut Self::Context) -> Self::Result {
        let client = self.client.clone();
        Box::pin(async move {
            Ok(client
                .privmsg(
                    msg.broadcaster,
                    format!("/timeout {} {}", msg.user, msg.duration.as_secs()),
                )
                .await?)
        })
    }
}

impl Handler<TimedModeMessage> for IrcActor {
    type Result = ();

    fn handle(&mut self, msg: TimedModeMessage, _ctx: &mut Self::Context) -> Self::Result {
        let client = self.client.clone();
        task::spawn(async move {
            if let Err(e) = client
                .privmsg(msg.broadcaster.clone(), format!("/{}", msg.mode))
                .await
            {
                println!("Could not enter {}: {:?}", msg.mode, e);
            }
            tokio::time::sleep(msg.duration).await;
            if let Err(e) = client
                .privmsg(msg.broadcaster, format!("/{}off", msg.mode))
                .await
            {
                println!("Could not leave {}: {:?}", msg.mode, e);
            }
        });
    }
}
