use crate::{
    services::{
        bttv::{get_or_fetch_id, requests as bttv},
        emotes::{Emote, EmoteEnvData, EmoteId, EmoteInitialData, EmoteRW},
    },
    RedisPool,
};
use anyhow::{anyhow, Error as AnyError, Result as AnyResult};
use async_trait::async_trait;
use futures::TryFutureExt;
use models::{emote::SlotPlatform, swap_emote::SwapEmote};
use sqlx::PgPool;

pub struct BttvEmotes {
    _private: usize,
}

impl Emote<String> for bttv::BttvEmote {
    fn id(&self) -> &String {
        &self.id
    }

    fn name(&self) -> &str {
        &self.code
    }

    fn into_name(self) -> String {
        self.code
    }
}

impl EmoteId for String {
    fn from_db(db: &str) -> AnyResult<Self> {
        Ok(db.to_owned())
    }
}

#[async_trait]
impl EmoteRW for BttvEmotes {
    type PlatformId = String;
    type Emote = bttv::BttvEmote;
    type EmoteId = String;

    fn platform() -> SlotPlatform {
        SlotPlatform::Bttv
    }

    async fn get_check_initial_data(
        broadcaster_id: &str,
        emote_id: &str,
        overwritten_name: Option<&str>,
        _allow_unlisted: bool,
        pool: &PgPool,
        _redis_pool: &RedisPool,
    ) -> AnyResult<EmoteInitialData<String, bttv::BttvEmote>> {
        if overwritten_name.is_some() {
            return Err(anyhow!("BTTV doesn't support renaming emotes"));
        }

        let (bttv_id, history_len) = futures::future::try_join(
            get_or_fetch_id(broadcaster_id, pool),
            SwapEmote::emote_count(broadcaster_id, Self::platform(), pool)
                .map_err(|_| AnyError::msg("Could not get past emotes")),
        )
        .await?;

        // get the data in parallel
        let (bttv_user, user_limits, emote_data) = futures::future::try_join3(
            bttv::get_user(&bttv_id)
                .map_err(|_| AnyError::msg("No such user.")),
            get_user_limits(&bttv_id)
                .map_err(|_| AnyError::msg("I'm not added as an editor.")),
            bttv::get_emote(emote_id)
                .map_err(|_| AnyError::msg("This emote doesn't exist.")),
        )
        .await?;

        // check if there's already an emote with the same name or id
        // If the added emote will replace an emote with the same name it will never work!
        if bttv_user
            .shared_emotes
            .iter()
            .any(|e| e.id == emote_id || e.code == emote_data.code)
        {
            return Err(AnyError::msg(
                "The emote already exists as a shared emote",
            ));
        }
        if bttv_user
            .channel_emotes
            .iter()
            .any(|e| e.id == emote_id || e.code == emote_data.code)
        {
            return Err(AnyError::msg(
                "The emote already exists as a channel emote",
            ));
        }
        Ok(EmoteInitialData {
            max_emotes: user_limits.channel_emotes,
            current_emotes: bttv_user.shared_emotes.len(),
            history_len: history_len as usize,
            platform_id: bttv_id,
            emote: emote_data,
            emotes: bttv_user.shared_emotes,
        })
    }

    async fn get_emote_env_data(
        _broadcaster_id: &str,
        platform_id: &String,
    ) -> AnyResult<EmoteEnvData> {
        let (bttv_limits, shared_emotes) = futures::future::try_join(
            get_user_limits(platform_id),
            bttv::get_user(platform_id),
        )
        .await?;

        Ok(EmoteEnvData {
            max_emotes: bttv_limits.channel_emotes,
            current_emotes: shared_emotes.shared_emotes.len(),
        })
    }

    async fn get_emotes(
        broadcaster_id: &str,
        pool: &PgPool,
    ) -> AnyResult<Vec<Self::Emote>> {
        let bttv_id = get_or_fetch_id(broadcaster_id, pool).await?;

        let bttv_user = bttv::get_user(&bttv_id)
            .map_err(|_| AnyError::msg("No such user."))
            .await?;
        Ok(bttv_user.shared_emotes)
    }

    async fn get_emote_by_id(emote_id: &String) -> AnyResult<bttv::BttvEmote> {
        bttv::get_emote(emote_id).await
    }

    async fn remove_emote(
        platform_id: &String,
        emote_id: &String,
        _: &RedisPool,
    ) -> AnyResult<()> {
        bttv::delete_shared_emote(emote_id, platform_id).await?;
        Ok(())
    }

    async fn add_emote(
        platform_id: &String,
        emote_id: &String,
        _overwritten_name: Option<&str>,
        _redis_pool: &RedisPool,
    ) -> AnyResult<()> {
        bttv::add_shared_emote(emote_id, platform_id).await?;
        Ok(())
    }

    async fn remove_emote_from_broadcaster(
        broadcaster_id: &str,
        emote_id: &str,
        pool: &PgPool,
        _: &RedisPool,
    ) -> AnyResult<String> {
        let bttv_id = match get_or_fetch_id(broadcaster_id, pool).await {
            Ok(id) => id,
            Err(e) => {
                log::warn!(
                    "No user-id? broadcaster={} error={}",
                    broadcaster_id,
                    e
                );
                return Err(e);
            }
        };

        let (_, emote) = futures::future::try_join(
            bttv::delete_shared_emote(emote_id, &bttv_id),
            bttv::get_emote(emote_id),
        )
        .await?;

        Ok(emote.code)
    }

    async fn get_platform_id(
        broadcaster_id: &str,
        pool: &PgPool,
    ) -> AnyResult<Self::PlatformId> {
        get_or_fetch_id(broadcaster_id, pool).await
    }

    fn format_emote_url(emote_id: &str) -> String {
        format!("https://cdn.betterttv.net/emote/{}/3x", emote_id)
    }

    fn format_emote_page(emote_id: &str) -> String {
        format!("https://betterttv.com/emotes/{}", emote_id)
    }
}

async fn get_user_limits(bttv_id: &str) -> AnyResult<bttv::BttvLimits> {
    bttv::get_dashboards()
        .await?
        .into_iter()
        .find(|d| d.id == bttv_id)
        .map(|u| u.limits)
        .ok_or_else(|| AnyError::msg("User isn't an editor"))
}
