use actix::{
    Actor, ActorFutureExt, AsyncContext, Context, ContextFutureSpawner,
    Handler, WrapFuture,
};
use anyhow::{Error as AnyError, Result as AnyResult};
use chrono::Utc;
use futures::future::TryFutureExt;
use sqlx::PgPool;
use twitch_api::{
    helix::points::UpdateCustomRewardBody, twitch_oauth2::UserToken,
};

use crate::{
    log_discord, log_err,
    services::twitch::{
        self,
        requests::{
            get_reward_for_broadcaster_by_id, send_chat_message, update_reward,
        },
    },
};
use models::{reward::Reward, user::User};

mod messages;
pub use messages::*;

struct UnpauseInfo {
    run_in: std::time::Duration,
    reward_id: String,
    user_token: UserToken,
}

pub struct LiveActor {
    pool: PgPool,
}

impl LiveActor {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn on_live(
        user_id: &str,
        pool: &PgPool,
    ) -> AnyResult<Vec<UnpauseInfo>> {
        let user_token: UserToken =
            User::get_by_id(user_id, pool).await?.into();

        log_discord!(format!("🔴 {} is now live", user_token.login));

        let live = Reward::get_all_live_for_user(user_id, pool).await?;
        if live.is_empty() {
            return Ok(vec![]);
        }

        let mut pending = vec![];
        for reward in live {
            if let Some(duration) = reward.live_delay.and_then(|delay| {
                humantime::parse_duration(&delay)
                    .map(|d| chrono::Duration::from_std(d).ok())
                    .ok()
                    .flatten()
            }) {
                if let Ok(res) = get_reward_for_broadcaster_by_id(
                    user_id,
                    &[reward.id.as_str().into()],
                    &user_token,
                )
                .await
                {
                    if !res.is_paused {
                        pending.push((duration, res.id.take()));
                    }
                }
            }
        }

        log::info!("Pausing {} rewards", pending.len());
        log_err!(
            send_chat_message(
                user_token.user_id.as_str(),
                &format!(
                    "🔴 Live, pausing {} reward{} at the start.",
                    pending.len(),
                    if pending.len() != 1 { "s" } else { "" }
                ),
                &twitch::get_token()
            )
            .await,
            "Could not send chat"
        );

        for (duration, id) in pending.iter() {
            log_err!(
                Reward::set_unpause_at(id, Some(Utc::now() + *duration), pool)
                    .await,
                "Could not set unpause_at"
            );
            log_err!(
                update_reward(
                    user_id,
                    id.clone(),
                    UpdateCustomRewardBody::builder().is_paused(true).build(),
                    &user_token
                )
                .await,
                "Could not pause reward"
            );
        }

        Ok(pending
            .into_iter()
            .map(|(duration, reward_id)| UnpauseInfo {
                run_in: duration.to_std().unwrap_or_default(),
                reward_id,
                user_token: user_token.clone(),
            })
            .collect())
    }

    async fn on_offline(user_id: &str, pool: &PgPool) -> AnyResult<()> {
        let user_token: UserToken =
            User::get_by_id(user_id, pool).await?.into();
        let pending =
            Reward::get_all_pending_live_for_user(user_id, pool).await?;

        log_discord!(format!("📴 {} is now offline", user_token.login));

        log::info!("Unpausing {} rewards", pending.len());

        for reward in pending {
            log_err!(
                futures::future::try_join(
                    Reward::set_unpause_at(&reward.id, None, pool)
                        .map_err(AnyError::from),
                    update_reward(
                        user_id,
                        reward.id.clone(),
                        UpdateCustomRewardBody::builder()
                            .is_paused(false)
                            .build(),
                        &user_token
                    )
                    .map_err(AnyError::from)
                )
                .await,
                "Could not update reward"
            );
        }

        Ok(())
    }

    async fn clear_all(pool: &PgPool) -> AnyResult<()> {
        let rewards = Reward::get_all_pending_live(pool).await?;

        log::info!("Clearing {} live-delay rewards", rewards.len());

        for reward in rewards {
            let (reward, user_token) = reward.into();
            log_err!(
                futures::future::try_join(
                    Reward::set_unpause_at(&reward.id, None, pool)
                        .map_err(AnyError::from),
                    update_reward(
                        reward.user_id.clone(),
                        reward.id.clone(),
                        UpdateCustomRewardBody::builder()
                            .is_paused(false)
                            .build(),
                        &user_token
                    )
                    .map_err(AnyError::from)
                )
                .await,
                "Could not update reward"
            );
        }

        Ok(())
    }
}

impl Actor for LiveActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let pool = self.pool.clone();
        ctx.spawn(
            async move {
                log_err!(Self::clear_all(&pool).await, "Could not clear all");
            }
            .into_actor(self),
        );
    }
}

impl Handler<LiveMessage> for LiveActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: LiveMessage,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let pool = self.pool.clone();
        async move { Self::on_live(&msg.0, &pool).await }
            .into_actor(self)
            .map(|res, _this, ctx| match res {
                Ok(to_queue) => {
                    for info in to_queue {
                        ctx.run_later(info.run_in, |this, ctx| {
                            let pool = this.pool.clone();
                            log::info!("Unpausing id={}", info.reward_id);
                            ctx.spawn(
                                async move {
                                    log_err!(
                                        Reward::set_unpause_at(
                                            &info.reward_id,
                                            None,
                                            &pool
                                        )
                                        .await,
                                        "Could not set unpause_at"
                                    );
                                    log_err!(
                                        update_reward(
                                            info.user_token.user_id.as_str(),
                                            info.reward_id,
                                            UpdateCustomRewardBody::builder()
                                                .is_paused(false)
                                                .build(),
                                            &info.user_token
                                        )
                                        .await,
                                        "Could not unpause reward"
                                    );
                                }
                                .into_actor(this),
                            );
                        });
                    }
                }
                Err(e) => log::warn!("Some update error: {}", e),
            })
            .spawn(ctx);
    }
}

impl Handler<OfflineMessage> for LiveActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: OfflineMessage,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let pool = self.pool.clone();
        ctx.spawn(
            async move {
                log_err!(
                    Self::on_offline(&msg.0, &pool).await,
                    "Could not re-enable rewards"
                );
            }
            .into_actor(self),
        );
    }
}
