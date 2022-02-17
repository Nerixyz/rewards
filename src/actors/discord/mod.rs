mod messages;

use crate::{
    log_err,
    services::discord::{send_user_webhook_message, WebhookReq},
};
use actix::{Actor, Context, ContextFutureSpawner, Handler, WrapFuture};
use models::discord;
use sqlx::PgPool;

pub use messages::*;

pub struct DiscordActor {
    db: PgPool,
}

impl DiscordActor {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

impl Actor for DiscordActor {
    type Context = Context<Self>;
}

impl Handler<LogToDiscordMessage> for DiscordActor {
    type Result = ();

    fn handle(&mut self, msg: LogToDiscordMessage, ctx: &mut Self::Context) -> Self::Result {
        let db = self.db.clone();
        async move {
            if let Ok(Some(url)) = discord::get_discord_webhook_url(&msg.user_id, &db).await {
                log_err!(
                    send_user_webhook_message(&url, &WebhookReq::Embeds(vec![msg.embed])).await,
                    "Could not send user discord message"
                );
            }
        }
        .into_actor(self)
        .spawn(ctx);
    }
}
