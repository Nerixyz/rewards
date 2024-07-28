use std::{collections::HashMap, time::Duration};

use actix::{
    Actor, ActorFutureExt, Addr, AsyncContext, Context, ContextFutureSpawner,
    Handler, Recipient, WrapFuture,
};
use tokio::sync::mpsc::UnboundedReceiver;
use twitch_irc::{
    login::StaticLoginCredentials,
    message::{ClearChatAction, ClearChatMessage, ServerMessage},
    ClientConfig, MetricsConfig, SecureTCPTransport, TwitchIRCClient,
};

use crate::{
    actors::timeout::TimeoutActor,
    chat::{parse::opt_next_space, try_parse_command},
    log_err,
    services::twitch::{self, requests::send_chat_message},
};
use config::CONFIG;

mod messages;
use crate::actors::{
    chat::ExecuteCommandMessage, timeout::ChannelTimeoutMessage,
};
pub use messages::*;

type IrcClient = TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>;

pub struct IrcActor {
    client: IrcClient,

    incoming: Option<UnboundedReceiver<ServerMessage>>,

    executor: Recipient<ExecuteCommandMessage>,

    last_messages: HashMap<String, String>,

    timeout_handler: Addr<TimeoutActor>,
}

impl IrcActor {
    pub fn new(
        executor: Recipient<ExecuteCommandMessage>,
        timeout_handler: Addr<TimeoutActor>,
    ) -> Self {
        let config = ClientConfig {
            metrics_config: MetricsConfig::default(),

            // TODO: improve this but the user isn't a verified bot yet FeelsMan
            new_connection_every: Duration::from_secs(12),
            max_channels_per_connection: 20,

            ..ClientConfig::new_simple(StaticLoginCredentials::anonymous())
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
                                is_self: false,
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
                    })
                    .into_actor(self)
                    .map(|res, _, _| {
                        log_err!(res, "Could not deliver");
                    })
                    .spawn(ctx);
            }
            Some(Err(e)) => {
                ctx.spawn(
                    async move {
                        log_err!(
                            send_chat_message(
                                &msg.0.channel_id,
                                &format!("@{}, ⚠ {}", msg.0.sender.login, e),
                                &twitch::get_token(),
                            )
                            .await,
                            "Failed to send chat"
                        );
                    }
                    .into_actor(self),
                );
            }
            None => (),
        }
    }
}
