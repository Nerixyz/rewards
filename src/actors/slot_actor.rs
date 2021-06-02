use crate::log_err;
use crate::models::bttv_slot::BttvSlot;
use crate::models::log_entry::LogEntry;
use crate::models::user::User;
use crate::services::bttv::get_or_fetch_id;
use crate::services::bttv::requests::{delete_shared_emote, get_emote};
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

            let (bttv_response, sql_response, internal_user, emote_data) = futures::future::join4(
                delete_shared_emote(emote_id, &bttv_id),
                BttvSlot::clear(p.id, &pool),
                User::get_by_id(&p.user_id, &pool),
                get_emote(emote_id),
            )
            .await;
            log_err!(bttv_response, "Could not delete {:?} error={}", p);
            log_err!(
                sql_response,
                "Could not clear pending slot {:?} error={}",
                p
            );
            match emote_data {
                Ok(emote) => {
                    log_err!(
                        LogEntry::create(
                            &p.user_id,
                            &format!("[bttv::slots] Deleted {}", emote.code),
                            &pool
                        )
                        .await,
                        "Could not save logs"
                    );
                }
                Err(e) => {
                    log::warn!("Error requesting bttv-emote data: {}", e);
                    log_err!(
                        LogEntry::create(
                            &p.user_id,
                            &format!("bttv::slots Deleted {} - no emote data available", emote_id),
                            &pool
                        )
                        .await,
                        "Could not save logs"
                    );
                }
            }

            let token: UserToken = match internal_user {
                Ok(t) => t.into(),
                Err(e) => {
                    log::warn!("Could not get user: {:?} error={}", p, e);
                    continue;
                }
            };

            match update_reward(
                &token.user_id,
                p.reward_id.clone(),
                UpdateCustomRewardBody::builder()
                    .is_paused(Some(false))
                    .build(),
                &token,
            )
            .await
            {
                Ok(_) => log::info!("Enabled {:?}", p),
                Err(e) => log::warn!("Could not enable: reward={:?} error={}", p, e),
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
