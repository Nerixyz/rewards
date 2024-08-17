use crate::{
    log_err,
    services::twitch::{self, requests::send_chat_message},
    AppAccessToken, RedisConn, RedisPool,
};
use actix::{Actor, Context, ContextFutureSpawner, Handler, WrapFuture};
use deadpool_redis::{redis, redis::AsyncCommands};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

mod messages;
pub use messages::*;

pub struct ChatActor {
    pool: PgPool,
    redis: RedisPool,
    app_access_token: Arc<RwLock<AppAccessToken>>,
}

impl ChatActor {
    pub fn new(
        pool: PgPool,
        redis: RedisPool,
        app_access_token: Arc<RwLock<AppAccessToken>>,
    ) -> Self {
        Self {
            pool,
            redis,
            app_access_token,
        }
    }
}

impl Actor for ChatActor {
    type Context = Context<Self>;
}

impl Handler<ExecuteCommandMessage> for ChatActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: ExecuteCommandMessage,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let pool = self.pool.clone();
        let redis = self.redis.clone();
        let app_access_token = self.app_access_token.clone();
        async move {
            log_err!(
                try_handle_command(msg, pool, redis, app_access_token).await,
                "Could not handle command"
            );
        }
        .into_actor(self)
        .spawn(ctx);
    }
}

async fn check_update_cooldown(
    conn: &mut RedisConn,
    channel: &str,
    user: &str,
) -> anyhow::Result<bool> {
    let user_key = format!("rewards:cooldown:{}:{}", channel, user);
    let channel_key = format!("rewards:channel-limit:{}", channel);
    let existing: i8 = conn.exists(&[&user_key, &channel_key]).await?;
    if existing != 0 {
        return Ok(false);
    }

    redis::pipe()
        .cmd("SETEX")
        .arg(&user_key)
        .arg(5)
        .arg(1)
        .cmd("SETEX")
        .arg(&channel_key)
        .arg(1)
        .arg(1)
        .query_async::<()>(conn)
        .await?;
    Ok(true)
}

async fn try_handle_command(
    mut msg: ExecuteCommandMessage,
    db: PgPool,
    redis: RedisPool,
    app_access_token: Arc<RwLock<AppAccessToken>>,
) -> anyhow::Result<()> {
    let mut conn = redis.get().await?;
    if !check_update_cooldown(
        &mut conn,
        &msg.raw.channel_id,
        &msg.raw.sender.id,
    )
    .await?
    {
        // nothing went wrong we're just on cooldown
        return Ok(());
    }
    if !msg
        .executor
        .check_permission(&msg.raw, &db, &mut conn)
        .await
    {
        send_chat_message(
            &msg.raw.channel_id,
            &format!(
                "@{}, ⛔ You don't have permission to run this command!",
                msg.raw.sender.login
            ),
            &twitch::get_token(),
        )
        .await?;
        return Ok(());
    }

    let channel_id = msg.raw.channel_id.clone();
    let sender = msg.raw.sender.login.clone();
    match msg
        .executor
        .execute(msg.raw, &db, redis, app_access_token)
        .await
    {
        Ok(res) => {
            send_chat_message(&channel_id, &res, &twitch::get_token()).await
        }
        Err(e) => {
            send_chat_message(
                &channel_id,
                &format!("@{}, ⚠ {}", sender, e),
                &twitch::get_token(),
            )
            .await
        }
    }
}
