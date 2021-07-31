use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use actix::{
    Actor, ActorFutureExt, Addr, AsyncContext, Context, ContextFutureSpawner, Handler, Recipient,
    ResponseFuture, WrapFuture,
};
use anyhow::Error as AnyError;
use sqlx::PgPool;
use tokio::{
    sync::{
        mpsc::UnboundedReceiver,
        watch::{channel, Receiver, Sender},
    },
    task,
};
use twitch_irc::{
    login::RefreshingLoginCredentials,
    message::{ClearChatAction, ClearChatMessage, NoticeMessage, ServerMessage},
    ClientConfig, SecureTCPTransport, TwitchIRCClient,
};

use token_storage::PgTokenStorage;

use crate::{
    actors::{db::DbActor, timeout::TimeoutActor},
    chat::{parse::opt_next_space, try_parse_command},
    config::CONFIG,
    log_err,
    models::timed_mode::TimedMode,
};
mod messages;
mod token_storage;
use crate::actors::{
    chat::ExecuteCommandMessage,
    timeout::{ChannelTimeoutMessage, RemoveTimeoutMessage},
};
pub use messages::*;

type IrcCredentials = RefreshingLoginCredentials<PgTokenStorage>;
type IrcClient = TwitchIRCClient<SecureTCPTransport, IrcCredentials>;

type NoticeChannelMessage = (Instant, NoticeMessage);
pub struct IrcActor {
    client: IrcClient,
    pool: PgPool,

    incoming: Option<UnboundedReceiver<ServerMessage>>,
    notice_tx: Option<Sender<Option<NoticeChannelMessage>>>,
    notice_rx: Receiver<Option<NoticeChannelMessage>>,

    executor: Recipient<ExecuteCommandMessage>,
    last_message: Option<String>,
    command_cooldown: HashMap<String, Instant>,

    timeout_handler: Addr<TimeoutActor>,
}

impl IrcActor {
    pub fn new(
        db: Addr<DbActor>,
        pool: PgPool,
        executor: Recipient<ExecuteCommandMessage>,
        timeout_handler: Addr<TimeoutActor>,
    ) -> Self {
        let config = ClientConfig {
            metrics_identifier: Some("rewardmore".into()),
            ..ClientConfig::new_simple(IrcCredentials::new(
                CONFIG.twitch.login.to_string(),
                CONFIG.twitch.client_id.to_string(),
                CONFIG.twitch.client_secret.to_string(),
                PgTokenStorage(db),
            ))
        };

        let (incoming, client) = IrcClient::new(config);
        let (notice_tx, notice_rx) = channel(None);

        Self {
            client,
            pool,

            incoming: Some(incoming),
            notice_tx: Some(notice_tx),
            notice_rx,

            executor,
            last_message: None,
            command_cooldown: HashMap::new(),

            timeout_handler,
        }
    }

    /// Returns true if there's _no_ cooldown for the channel.
    fn check_update_cooldown(&mut self, login: &str) -> bool {
        let now = Instant::now();
        if let Some(v) = self.command_cooldown.get_mut(login) {
            if now.duration_since(*v) < Duration::from_secs(4) {
                false
            } else {
                *v = now;
                true
            }
        } else {
            self.command_cooldown.insert(login.to_string(), now);
            true
        }
    }
}

impl Actor for IrcActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let mut incoming = self.incoming.take().expect("This was set in Self:new");
        let tx = self.notice_tx.take().expect("This was set in Self::new");
        let addr = ctx.address();
        let timeouter = self.timeout_handler.clone();

        ctx.spawn(
            async move {
                while let Some(message) = incoming.recv().await {
                    match message {
                        ServerMessage::Notice(notice) => {
                            tx.send(Some((Instant::now(), notice))).ok();
                        }
                        ServerMessage::Privmsg(privmsg) => {
                            addr.do_send(ChatMessage(privmsg));
                        }
                        ServerMessage::ClearChat(ClearChatMessage {
                            action:
                                ClearChatAction::UserTimedOut {
                                    user_id,
                                    timeout_length,
                                    ..
                                },
                            channel_id,
                            ..
                        }) => {
                            timeouter.do_send(ChannelTimeoutMessage {
                                channel_id,
                                user_id,
                                duration: timeout_length,
                            });
                        }
                        _ => (),
                    };
                }
            }
            .into_actor(self),
        );
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
        self.command_cooldown.remove(&msg.0);
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

impl Handler<WhisperMessage> for IrcActor {
    type Result = ResponseFuture<Result<(), AnyError>>;

    fn handle(&mut self, msg: WhisperMessage, _ctx: &mut Self::Context) -> Self::Result {
        let client = self.client.clone();
        log::info!("whisper to={} message={}", msg.0, msg.1);
        Box::pin(async move {
            Ok(client
                .privmsg(
                    CONFIG.twitch.login.to_string(),
                    format!("/w {} {}", msg.0, msg.1),
                )
                .await?)
        })
    }
}

impl Handler<SayMessage> for IrcActor {
    type Result = ResponseFuture<Result<(), AnyError>>;

    fn handle(&mut self, mut msg: SayMessage, _ctx: &mut Self::Context) -> Self::Result {
        let client = self.client.clone();

        if self.last_message.as_ref() == Some(&msg.1) {
            log::info!("eq");
            msg.1.push('\u{180d}');
        }
        self.last_message = Some(msg.1.clone());

        log::info!("Send message login={}; message={}", msg.0, msg.1);
        Box::pin(async move { Ok(client.say(msg.0, msg.1).await?) })
    }
}

impl Handler<TimeoutMessage> for IrcActor {
    type Result = ResponseFuture<Result<(), AnyError>>;

    fn handle(&mut self, msg: TimeoutMessage, _ctx: &mut Self::Context) -> Self::Result {
        let client = self.client.clone();
        let mut rx = self.notice_rx.clone();
        let timeout_handler = self.timeout_handler.clone();
        Box::pin(async move {
            log::info!(
                "timeout in {} for {} for {:?}s",
                msg.broadcaster,
                msg.user,
                msg.duration
            );
            client
                .timeout(
                    msg.broadcaster.clone(),
                    &msg.user,
                    Duration::from_secs(msg.duration),
                    Some("Channel Point Reward"),
                )
                .await?;
            let sent = Instant::now();

            while let Ok(Ok(_)) = tokio::time::timeout(Duration::from_secs(5), rx.changed()).await {
                let value = (*rx.borrow()).clone();
                let notice = match value {
                    Some((now, v)) if now > sent => v,
                    _ => continue,
                };
                log::info!(
                    "NOTICE: channel={:?} id={:?} text={}",
                    notice.channel_login,
                    notice.message_id,
                    notice.message_text
                );
                if Some(&msg.broadcaster) != notice.channel_login.as_ref() {
                    continue;
                }

                if let Some(id) = &notice.message_id {
                    match id.as_str() {
                        "timeout_success" => {
                            timeout_handler.do_send(RemoveTimeoutMessage {
                                channel_id: msg.broadcaster_id,
                                user_id: msg.user_id,
                                later: Duration::from_secs(5),
                            });
                            return Ok(());
                        }
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
            log::error!("No reply for timeout");

            Err(AnyError::msg("No reply"))
        })
    }
}

impl Handler<TimedModeMessage> for IrcActor {
    type Result = ();

    fn handle(&mut self, msg: TimedModeMessage, _ctx: &mut Self::Context) -> Self::Result {
        let client = self.client.clone();
        let pool = self.pool.clone();
        task::spawn(async move {
            log::info!(
                "{} in {} for {:?}s",
                msg.mode,
                msg.broadcaster,
                msg.duration
            );

            let mode_id = TimedMode::create_mode(
                &msg.broadcaster_id,
                msg.mode,
                std::time::Duration::from_secs(msg.duration),
                &pool,
            )
            .await
            .unwrap_or(0);

            if let Err(e) = client
                .privmsg(msg.broadcaster.clone(), format!("/{}", msg.mode))
                .await
            {
                log::warn!("Could not enter {}: {:?}", msg.mode, e);
            }
            tokio::time::sleep(std::time::Duration::from_secs(msg.duration)).await;

            log::info!("Leave {} in {}", msg.mode, msg.broadcaster);
            if let Err(e) = client
                .privmsg(msg.broadcaster, format!("/{}off", msg.mode))
                .await
            {
                log::warn!("Could not leave {}: {:?}", msg.mode, e);
            }

            log_err!(
                TimedMode::delete_mode(mode_id, &pool).await,
                "Failed to delete timed mode"
            );
        });
    }
}

impl Handler<ChatMessage> for IrcActor {
    type Result = ();

    fn handle(&mut self, msg: ChatMessage, ctx: &mut Self::Context) -> Self::Result {
        if !msg.0.message_text.starts_with("::")
            || msg.0.message_text.len() < 2
            || !self.check_update_cooldown(&msg.0.channel_login)
        {
            return;
        }
        let (command, args) = opt_next_space(&msg.0.message_text[2..]);
        match try_parse_command(command, args) {
            Some(Ok(ex)) => {
                self.executor
                    .send(ExecuteCommandMessage {
                        executor: ex,
                        raw: msg.0,
                        addr: ctx.address().recipient(),
                    })
                    .into_actor(self)
                    .map(|res, _, _| {
                        log_err!(res, "Could not deliver");
                    })
                    .spawn(ctx);
            }
            Some(Err(e)) => {
                ctx.notify(SayMessage(
                    msg.0.channel_login,
                    format!("@{}, ⚠ {}", msg.0.sender.login, e),
                ));
            }
            None => (),
        }
    }
}
