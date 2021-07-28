use std::time::Duration;

use actix::{Actor, AsyncContext, Context, Handler, ResponseFuture, WrapFuture};
use chrono::Utc;
use sqlx::PgPool;

use crate::{log_err, models::timeout::Timeout};

mod messages;
pub use messages::*;

pub struct TimeoutActor {
    pool: PgPool,
}

impl TimeoutActor {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Actor for TimeoutActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(60), |this, ctx| {
            let pool = this.pool.clone();
            ctx.spawn(
                async move {
                    log_err!(
                        Timeout::delete_inactive(&pool).await,
                        "Could not clear timeouts"
                    );
                }
                .into_actor(this),
            );
        });
    }
}

impl Handler<ChannelTimeoutMessage> for TimeoutActor {
    type Result = ();

    fn handle(&mut self, msg: ChannelTimeoutMessage, ctx: &mut Self::Context) -> Self::Result {
        let pool = self.pool.clone();
        ctx.spawn(
            async move {
                let chrono_duration = chrono::Duration::from_std(msg.duration)
                    .unwrap_or_else(|_| chrono::Duration::minutes(0));

                log::info!(
                    "Timeout in {} for {} for {:?}",
                    msg.channel_id,
                    msg.user_id,
                    msg.duration
                );

                if chrono_duration > chrono::Duration::seconds(30) {
                    // only save timeouts longer than 30s
                    log_err!(
                        Timeout::create(
                            &msg.channel_id,
                            &msg.user_id,
                            Utc::now() + chrono_duration,
                            &pool
                        )
                        .await,
                        "Couldn't save timeout"
                    );
                } else {
                    log_err!(
                        Timeout::delete_specific(&msg.channel_id, &msg.user_id, &pool).await,
                        "Couldn't remove timeout"
                    );
                }
            }
            .into_actor(self),
        );
    }
}

impl Handler<CheckValidTimeoutMessage> for TimeoutActor {
    type Result = ResponseFuture<anyhow::Result<bool>>;

    fn handle(&mut self, msg: CheckValidTimeoutMessage, _ctx: &mut Self::Context) -> Self::Result {
        let pool = self.pool.clone();
        Box::pin(async move {
            let timeout = Timeout::get_timeout(&msg.channel_id, &msg.user_id, &pool).await?;

            Ok(if let Some(timeout) = timeout {
                timeout < Utc::now()
            } else {
                true
            })
        })
    }
}

impl Handler<RemoveTimeoutMessage> for TimeoutActor {
    type Result = ();

    fn handle(&mut self, msg: RemoveTimeoutMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.run_later(msg.later, |this, ctx| {
            let pool = this.pool.clone();
            ctx.spawn(
                async move {
                    log::info!("Clear in {} for {}", msg.channel_id, msg.user_id);
                    log_err!(
                        Timeout::delete_specific(&msg.channel_id, &msg.user_id, &pool).await,
                        "Could not clear specific"
                    );
                }
                .into_actor(this),
            );
        });
    }
}
