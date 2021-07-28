use actix::{Actor, Context, ContextFutureSpawner, Handler, WrapFuture};
use sqlx::PgPool;
mod messages;

use crate::actors::irc::SayMessage;
pub use messages::*;

pub struct ChatActor {
    pool: PgPool,
}

impl ChatActor {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Actor for ChatActor {
    type Context = Context<Self>;
}

impl Handler<ExecuteCommandMessage> for ChatActor {
    type Result = ();

    fn handle(&mut self, mut msg: ExecuteCommandMessage, ctx: &mut Self::Context) -> Self::Result {
        let pool = self.pool.clone();
        async move {
            let broadcaster = msg.raw.channel_login.clone();
            let sender = msg.raw.sender.login.clone();
            let message = match msg.executor.execute(msg.raw, &pool).await {
                Ok(res) => SayMessage(broadcaster, res),
                Err(e) => SayMessage(broadcaster, format!("@{}, ⚠ {}", sender, e)),
            };

            match msg.addr.send(message).await {
                Ok(Ok(_)) => (),
                _ => log::warn!("Could not send"),
            };
        }
        .into_actor(self)
        .spawn(ctx);
    }
}
