use crate::{actors::irc::SayMessage, log_err, RedisConn, RedisPool};
use actix::{Actor, Context, ContextFutureSpawner, Handler, WrapFuture};
use deadpool_redis::{redis, redis::AsyncCommands};
use sqlx::PgPool;

mod messages;
pub use messages::*;

pub struct ChatActor {
    pool: PgPool,
    redis: RedisPool,
}

impl ChatActor {
    pub fn new(pool: PgPool, redis: RedisPool) -> Self {
        Self { pool, redis }
    }
}

impl Actor for ChatActor {
    type Context = Context<Self>;
}

impl Handler<ExecuteCommandMessage> for ChatActor {
    type Result = ();

    fn handle(&mut self, msg: ExecuteCommandMessage, ctx: &mut Self::Context) -> Self::Result {
        let pool = self.pool.clone();
        let redis = self.redis.clone();
        async move {
            log_err!(
                try_handle_command(msg, pool, redis).await,
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
        .query_async::<_, ()>(conn)
        .await?;
    Ok(true)
}

async fn try_handle_command(
    mut msg: ExecuteCommandMessage,
    db: PgPool,
    redis: RedisPool,
) -> anyhow::Result<()> {
    let mut conn = redis.get().await?;
    if !check_update_cooldown(&mut conn, &msg.raw.channel_id, &msg.raw.sender.id).await? {
        // nothing went wrong we're just on cooldown
        return Ok(());
    }

    let broadcaster = msg.raw.channel_login.clone();
    let sender = msg.raw.sender.login.clone();
    let message = match msg.executor.execute(msg.raw, &db, &mut conn).await {
        Ok(res) => SayMessage(broadcaster, res),
        Err(e) => SayMessage(broadcaster, format!("@{}, ⚠ {}", sender, e)),
    };

    msg.addr.send(message).await??;
    Ok(())
}
