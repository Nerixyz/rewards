use crate::models::bttv_slot::BttvSlot;
use crate::models::user::User;
use crate::services::bttv::get_or_fetch_id;
use crate::services::bttv::requests::delete_shared_emote;
use crate::services::twitch::requests::update_reward;
use actix::{Actor, AsyncContext, Context, WrapFuture};
use sqlx::PgPool;
use std::time::Duration;
use twitch_api2::helix::points::UpdateCustomRewardBody;
use twitch_api2::twitch_oauth2::UserToken;

pub struct SlotActor {
    pool: PgPool,
}

impl SlotActor {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn queue_rewards(pool: PgPool) {
        let pending = BttvSlot::get_pending(&pool).await;
        let pending = match pending {
            Ok(p) => p,
            Err(e) => {
                log::warn!("Could not get pending: {}", e);
                return;
            }
        };

        if pending.is_empty() {
            return;
        }

        log::info!("Clearing {} pending slots", pending.len());

        for p in pending {
            let emote_id = match p.emote_id {
                Some(ref e) => e,
                _ => {
                    log::warn!("Invalid pending: {:?}", p);
                    continue;
                }
            };

            let bttv_id = match get_or_fetch_id(&p.user_id, &pool).await {
                Ok(id) => id,
                Err(e) => {
                    log::warn!("No user-id? pending={:?} error={}", p, e);
                    continue;
                }
            };
            if let Err(e) = delete_shared_emote(emote_id, &bttv_id).await {
                log::warn!("Could not delete {:?} error={}", p, e);
            }
            if let Err(e) = BttvSlot::clear(p.id, &pool).await {
                log::warn!("Could not clear pending slot {:?} error={}", p, e);
            }
            let token: UserToken = match User::get_by_id(&p.user_id, &pool).await {
                Ok(t) => t.into(),
                Err(e) => {
                    log::warn!("Could not get user: {:?} error={}", p, e);
                    continue;
                }
            };

            if let Err(e) = update_reward(
                &token.user_id,
                p.reward_id.clone(),
                UpdateCustomRewardBody::builder()
                    .is_paused(Some(false))
                    .build(),
                &token,
            )
            .await
            {
                log::warn!("Could not enable: reward={:?} error={}", p, e);
            } else {
                log::info!("Enabled {:?}", p);
            }
        }
    }
}

impl Actor for SlotActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(2 * 60), |this, ctx| {
            ctx.spawn(Self::queue_rewards(this.pool.clone()).into_actor(this));
        });
    }
}
