use std::{collections::HashMap, time::Duration};

use actix::{
    Actor, ActorFutureExt, Addr, AsyncContext, Context, ContextFutureSpawner,
    Handler, Recipient, ResponseFuture, WrapFuture,
};
use anyhow::Error as AnyError;
use tokio::sync::mpsc::UnboundedReceiver;
use twitch_irc::{
    login::RefreshingLoginCredentials,
    message::{ClearChatAction, ClearChatMessage, ServerMessage},
    ClientConfig, MetricsConfig, SecureTCPTransport, TwitchIRCClient,
};

use token_storage::PgTokenStorage;

use crate::{
    actors::{db::DbActor, timeout::TimeoutActor},
    chat::{parse::opt_next_space, try_parse_command},
    log_err,
};
use config::CONFIG;

mod messages;
mod token_storage;
use crate::actors::{
    chat::ExecuteCommandMessage, timeout::ChannelTimeoutMessage,
};
pub use messages::*;

type IrcCredentials = RefreshingLoginCredentials<PgTokenStorage>;
type IrcClient = TwitchIRCClient<SecureTCPTransport, IrcCredentials>;

pub struct IrcActor {
    client: IrcClient,

    incoming: Option<UnboundedReceiver<ServerMessage>>,

    executor: Recipient<ExecuteCommandMessage>,

    last_messages: HashMap<String, String>,

    timeout_handler: Addr<TimeoutActor>,
}

impl IrcActor {
    pub fn new(
        db: Addr<DbActor>,
        executor: Recipient<ExecuteCommandMessage>,
        timeout_handler: Addr<TimeoutActor>,
    ) -> Self {
        let config = ClientConfig {
            metrics_config: MetricsConfig::default(),

            // TODO: improve this but the user isn't a verified bot yet FeelsMan
            new_connection_every: Duration::from_secs(12),
            max_channels_per_connection: 20,

            ..ClientConfig::new_simple(IrcCredentials::init_with_username(
                Some(CONFIG.twitch.login.to_string()),
                CONFIG.twitch.client_id.to_string(),
                CONFIG.twitch.client_secret.to_string(),
                PgTokenStorage(db),
            ))
        };

        let (incoming, client) = IrcClient::new(config);

        Self {
            client,

            incoming: Some(incoming),

            executor,
            last_messages: HashMap::new(),

            timeout_handler,
        }
    }
}

impl Actor for IrcActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let mut incoming =
            self.incoming.take().expect("This was set in Self:new");
        let addr = ctx.address();
        let timeouter = self.timeout_handler.clone();

        ctx.spawn(
            async move {
                while let Some(message) = incoming.recv().await {
                    match message {
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

    fn handle(
        &mut self,
        msg: JoinMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        log_err!(self.client.join(msg.0), "Cannot join");
    }
}

impl Handler<PartMessage> for IrcActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: PartMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.last_messages.remove(&msg.0);
        self.client.part(msg.0)
    }
}

impl Handler<JoinAllMessage> for IrcActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: JoinAllMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        for channel in msg.0 {
            log_err!(self.client.join(channel), "Cannot join");
        }
    }
}

impl Handler<SayMessage> for IrcActor {
    type Result = ResponseFuture<Result<(), AnyError>>;

    fn handle(
        &mut self,
        mut msg: SayMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let client = self.client.clone();

        if let Some(last) = self.last_messages.get_mut(&msg.0) {
            if *last == msg.1 {
                msg.1.push('\u{180d}');
            }
            *last = msg.1.clone();
        } else {
            self.last_messages.insert(msg.0.clone(), msg.1.clone());
        }

        log::info!("Send message login={}; message={}", msg.0, msg.1);
        Box::pin(async move { Ok(client.say(msg.0, msg.1).await?) })
    }
}

impl Handler<ChatMessage> for IrcActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: ChatMessage,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        if !msg.0.message_text.starts_with(&CONFIG.bot.prefix)
            || msg.0.message_text.len() < CONFIG.bot.prefix.len()
        {
            return;
        }
        let (command, args) = opt_next_space(
            msg.0.message_text[CONFIG.bot.prefix.len()..].trim(),
        );
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
