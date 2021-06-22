use crate::log_err;
use crate::models::log_entry::LogEntry;
use crate::models::slot::{Slot, SlotPlatform};
use crate::models::user::User;
use crate::services::emotes::bttv::BttvEmotes;
use crate::services::emotes::ffz::FfzEmotes;
use crate::services::emotes::seven_tv::SevenTvEmotes;
use crate::services::emotes::EmoteRW;
use crate::services::twitch::requests::update_reward;
use actix::{Actor, AsyncContext, Context, WrapFuture};
use anyhow::Result as AnyResult;
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

    async fn delete_emote(
        platform: &SlotPlatform,
        broadcaster_id: &str,
        id: &str,
        pool: &PgPool,
    ) -> AnyResult<String> {
        match platform {
            SlotPlatform::Bttv => {
                BttvEmotes::remove_emote_from_broadcaster(broadcaster_id, id, pool).await
            }
            SlotPlatform::Ffz => {
                FfzEmotes::remove_emote_from_broadcaster(broadcaster_id, id, pool).await
            }
            SlotPlatform::SevenTv => {
                SevenTvEmotes::remove_emote_from_broadcaster(broadcaster_id, id, pool).await
            }
        }
    }

    async fn queue_rewards(pool: PgPool) {
        let pending = Slot::get_pending(&pool).await;
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

            let (sql_response, internal_user, emote) = futures::future::join3(
                Slot::clear(p.id, &pool),
                User::get_by_id(&p.user_id, &pool),
                Self::delete_emote(&p.platform, &p.user_id, emote_id, &pool),
            )
            .await;
            log_err!(
                sql_response,
                "Could not clear pending slot {:?} error={}",
                p
            );
            match emote {
                Ok(emote) => {
                    log_err!(
                        LogEntry::create(
                            &p.user_id,
                            &format!("[slots::{:?}] Deleted {}", p.platform, emote),
                            &pool
                        )
                        .await,
                        "Could not save logs"
                    );
                }
                Err(e) => {
                    log::warn!(
                        "Could not delete emote or emote data is gone: saved={:?} error={}",
                        p,
                        e
                    );
                    log_err!(
                        LogEntry::create(
                            &p.user_id,
                            &format!("[slots::{:?}] Deleted {} - no emote data available or could not delete on platform", p.platform, emote_id),
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
