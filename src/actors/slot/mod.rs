mod messages;

use crate::{
    actors::discord::DiscordActor,
    embed_builder, log_discord, log_err, send_discord,
    services::{
        emotes::{bttv::BttvEmotes, ffz::FfzEmotes, seven_tv::SevenTvEmotes, EmoteRW},
        twitch::requests::update_reward,
    },
    RedisPool,
};
use actix::{Actor, Addr, AsyncContext, Context, Handler, Supervised, SystemService, WrapFuture};
use anyhow::Result as AnyResult;
use deadpool_redis::redis::AsyncCommands;
pub use messages::*;
use models::{emote::SlotPlatform, log_entry::LogEntry, slot::Slot, user::User};
use sqlx::PgPool;
use std::time::Duration;
use twitch_api2::{helix::points::UpdateCustomRewardBody, twitch_oauth2::UserToken};

pub struct SlotActor {
    pool: PgPool,
    redis: RedisPool,
    discord: Addr<DiscordActor>,
}

impl SlotActor {
    pub fn new(pool: PgPool, redis: RedisPool, discord: Addr<DiscordActor>) -> Self {
        Self {
            pool,
            redis,
            discord,
        }
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

    async fn queue_rewards(pool: PgPool, redis: RedisPool, discord: Addr<DiscordActor>) {
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
                    log_discord!(
                        "Slots",
                        "🗑 Cleared slot",
                        0x00e676,
                        "UserId" = p.user_id.clone(),
                        "Platform" = format!("{:?}", p.platform),
                        "Emote" = emote.clone()
                    );
                    let discord = discord.clone();
                    send_discord!(
                        discord,
                        p.user_id.clone(),
                        embed_builder!("Emotes", format!("Removed {}", emote), 0xff5370,)
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
                    log_discord!(
                        "Slots",
                        "🗑 Cleared slot but there's no emote data",
                        0x00e676,
                        "UserId" = p.user_id.clone(),
                        "Platform" = format!("{:?}", p.platform),
                        "EmoteId" = emote_id,
                        "Error" = e.to_string()
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

            // also called when untracking emotes
            match update_reward(
                token.user_id.clone(),
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

            if let (Some(name), Ok(json), Ok(mut conn)) = (
                p.name.as_ref(),
                serde_json::to_string(&p),
                redis.get().await,
            ) {
                log_err!(
                    conn.set_ex::<_, _, ()>(
                        format!("rewards:exp-slots:{}:{}", p.user_id, name.to_lowercase()),
                        json,
                        5 * 60 * 60
                    )
                    .await,
                    "Could not set slot on redis"
                );
            }
        }
    }
}

impl Actor for SlotActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(2 * 60), |this, ctx| {
            ctx.spawn(
                Self::queue_rewards(this.pool.clone(), this.redis.clone(), this.discord.clone())
                    .into_actor(this),
            );
        });
    }
}

impl Handler<Recheck> for SlotActor {
    type Result = ();

    fn handle(&mut self, _: Recheck, ctx: &mut Self::Context) -> Self::Result {
        ctx.spawn(
            Self::queue_rewards(self.pool.clone(), self.redis.clone(), self.discord.clone())
                .into_actor(self),
        );
    }
}

impl SystemService for SlotActor {}
impl Supervised for SlotActor {}

impl Default for SlotActor {
    fn default() -> Self {
        unreachable!();
    }
}
