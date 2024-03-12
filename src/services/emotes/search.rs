use crate::{
    services::{bttv, ffz, seven_tv},
    RedisConn,
};
use anyhow::{anyhow, Result as AnyResult};
use deadpool_redis::redis::AsyncCommands;
use either::Either;
use futures_util::{future, TryFutureExt};
use models::{emote::SlotPlatform, slot::Slot, swap_emote::SwapEmote};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize, Deserialize)]
pub struct EmoteCache {
    pub seventv: Vec<seven_tv::requests::SevenEmote>,
    pub bttv: Vec<bttv::requests::BttvEmote>,
    pub ffz: Vec<ffz::requests::FfzEmote>,
}

impl EmoteCache {
    pub async fn fetch_or_load(
        user_id: &str,
        redis: &mut RedisConn,
        pg: &PgPool,
    ) -> AnyResult<Self> {
        let cached: Option<String> = redis
            .get(&format!("rewards:emote-cache:{}", user_id))
            .await?;
        match cached {
            Some(c) => serde_json::from_str(&c).map_err(anyhow::Error::from),
            None => {
                let this = Self::fetch(user_id, pg).await;
                redis
                    .set_ex::<_, _, ()>(
                        &format!("rewards:emote-cache:{}", user_id),
                        serde_json::to_string(&this)?,
                        10 * 60,
                    )
                    .await
                    .ok();
                Ok(this)
            }
        }
    }

    pub async fn fetch(user_id: &str, pg: &PgPool) -> Self {
        let (seventv, bttv, ffz) = future::join3(
            async move { seven_tv::requests::get_user(user_id).await },
            async move {
                let id = bttv::get_or_fetch_id(user_id, pg).await?;
                bttv::requests::get_user(&id).await
            },
            ffz::requests::get_room(user_id),
        )
        .await;

        Self {
            seventv: seventv
                .ok()
                .and_then(|s| s.emote_set)
                .map(|s| s.emotes)
                .unwrap_or_else(|| Vec::with_capacity(0)),
            bttv: bttv
                .map(|s| s.shared_emotes)
                .unwrap_or_else(|_| Vec::with_capacity(0)),
            ffz: ffz
                .map(|s| {
                    s.sets
                        .into_iter()
                        .flat_map(|(_, set)| set.emoticons)
                        .collect()
                })
                .unwrap_or_else(|_| Vec::with_capacity(0)),
        }
    }

    pub fn find_name_by_id(
        &self,
        emote_id: &str,
        platform: SlotPlatform,
    ) -> Option<&str> {
        match platform {
            SlotPlatform::Bttv => self
                .bttv
                .iter()
                .find(|emote| emote.id == emote_id)
                .map(|e| e.code.as_str()),
            SlotPlatform::Ffz => {
                let id = emote_id.parse::<usize>().ok()?;
                self.ffz
                    .iter()
                    .find(|emote| emote.id == id)
                    .map(|e| e.name.as_str())
            }
            SlotPlatform::SevenTv => self
                .seventv
                .iter()
                .find(|emote| emote.id == emote_id)
                .map(|e| e.name.as_str()),
        }
    }

    pub fn non_empty_platform(&self, platform: SlotPlatform) -> bool {
        match platform {
            SlotPlatform::Bttv => !self.bttv.is_empty(),
            SlotPlatform::Ffz => !self.ffz.is_empty(),
            SlotPlatform::SevenTv => !self.seventv.is_empty(),
        }
    }

    pub fn contains(&self, emote_id: &str, platform: SlotPlatform) -> bool {
        match platform {
            SlotPlatform::Bttv => {
                self.bttv.iter().any(|emote| emote.id == emote_id)
            }
            SlotPlatform::Ffz => {
                let id = emote_id.parse::<usize>().ok().unwrap_or(usize::MAX);
                self.ffz.iter().any(|emote| emote.id == id)
            }
            SlotPlatform::SevenTv => {
                self.seventv.iter().any(|emote| emote.id == emote_id)
            }
        }
    }
}

pub async fn search_emote_by_name(
    emote: &str,
    channel_id: &str,
    pool: &PgPool,
) -> AnyResult<Option<Either<Slot, SwapEmote>>> {
    Slot::get_slot_by_emote_name(channel_id, emote, pool)
        .and_then(|opt| async {
            match opt {
                Some(v) => Ok(Some(Either::Left(v))),
                None => SwapEmote::by_name(channel_id, emote, pool)
                    .await
                    .map(|e| e.map(Either::Right)),
            }
        })
        .await
        .map_err(|_| anyhow!("Internal Error"))
}

pub async fn search_by_id(
    channel_id: &str,
    emote_id: &str,
    platform: SlotPlatform,
    pool: &PgPool,
) -> AnyResult<Option<Either<Slot, SwapEmote>>> {
    Slot::get_slot_by_emote_id(channel_id, emote_id, platform, pool)
        .and_then(|opt| async {
            match opt {
                Some(v) => Ok(Some(Either::Left(v))),
                None => SwapEmote::by_id(channel_id, emote_id, platform, pool)
                    .await
                    .map(|e| e.map(Either::Right)),
            }
        })
        .await
        .map_err(|_| anyhow!("Internal Error"))
}
