use actix::{
    Actor, AsyncContext, Context, ContextFutureSpawner, Handler,
    ResponseFuture, WrapFuture,
};

use crate::{log_err, RedisPool};

mod messages;
use deadpool_redis::redis::{cmd, AsyncCommands};
pub use messages::*;

const SELF_TIMEOUT: i32 = 2;
const REGULAR_TIMEOUT: i32 = 1;

pub struct TimeoutActor {
    pool: RedisPool,
}

impl TimeoutActor {
    pub fn new(pool: RedisPool) -> Self {
        Self { pool }
    }
}

impl Actor for TimeoutActor {
    type Context = Context<Self>;
}

impl Handler<ChannelTimeoutMessage> for TimeoutActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: ChannelTimeoutMessage,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let pool = self.pool.clone();
        async move {
            if let Ok(mut conn) = pool.get().await {
                log::info!(
                    "Timeout in {} for {} for {:?}",
                    msg.channel_id,
                    msg.user_id,
                    msg.duration
                );

                log_err!(
                    cmd("SET")
                        .arg(format!(
                            "rewards:timeout:{}:{}",
                            msg.channel_id, msg.user_id
                        ))
                        .arg(if msg.is_self {
                            SELF_TIMEOUT
                        } else {
                            REGULAR_TIMEOUT
                        })
                        .arg("NX")
                        .arg("EX")
                        .arg(msg.duration.as_secs() as usize)
                        .query_async::<_, ()>(&mut conn)
                        .await,
                    "Couldn't set timeout"
                );
            }
        }
        .into_actor(self)
        .spawn(ctx);
    }
}

impl Handler<CheckValidTimeoutMessage> for TimeoutActor {
    type Result = ResponseFuture<anyhow::Result<bool>>;

    fn handle(
        &mut self,
        msg: CheckValidTimeoutMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let pool = self.pool.clone();
        Box::pin(async move {
            let mut conn = pool.get().await?;
            let val: Option<i32> = conn
                .get(format!(
                    "rewards:timeout:{}:{}",
                    msg.channel_id, msg.user_id
                ))
                .await?;

            match val {
                Some(SELF_TIMEOUT) => Ok(true),
                Some(_) /* REGULAR_TIMEOUT */ => Ok(false),
                None => Ok(false)
            }
        })
    }
}

impl Handler<RemoveTimeoutMessage> for TimeoutActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: RemoveTimeoutMessage,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        ctx.run_later(msg.later, |this, ctx| {
            let pool = this.pool.clone();
            async move {
                if let Ok(mut conn) = pool.get().await {
                    log::info!(
                        "Clear in {} for {}",
                        msg.channel_id,
                        msg.user_id
                    );

                    log_err!(
                        conn.del::<_, ()>(format!(
                            "rewards:timeout:{}:{}",
                            msg.channel_id, msg.user_id
                        ))
                        .await,
                        "Couldn't delete timeout"
                    );
                }
            }
            .into_actor(this)
            .spawn(ctx);
        });
    }
}
