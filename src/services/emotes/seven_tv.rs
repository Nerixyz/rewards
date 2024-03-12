use crate::{
    services::{
        emotes::{Emote, EmoteEnvData, EmoteInitialData, EmoteRW},
        seven_tv::requests as seven_tv,
    },
    RedisPool,
};
use anyhow::{anyhow, bail, Error as AnyError, Result as AnyResult};
use async_trait::async_trait;
use deadpool_redis::redis;
use futures::TryFutureExt;
use models::{emote::SlotPlatform, swap_emote::SwapEmote};
use sqlx::PgPool;

pub struct SevenTvEmotes {
    _private: (),
}

impl Emote<String> for seven_tv::SevenEmote {
    fn id(&self) -> &String {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn into_name(self) -> String {
        self.name
    }
}

#[async_trait]
impl EmoteRW for SevenTvEmotes {
    type PlatformId = String;
    type Emote = seven_tv::SevenEmote;
    type EmoteId = String;

    fn platform() -> SlotPlatform {
        SlotPlatform::SevenTv
    }

    async fn get_check_initial_data(
        broadcaster_id: &str,
        emote_id: &str,
        overwritten_name: Option<&str>,
        allow_unlisted: bool,
        pool: &PgPool,
    ) -> AnyResult<EmoteInitialData<Self::PlatformId, Self::Emote>> {
        let (history_len, emote, stv_user) = futures::future::try_join3(
            SwapEmote::emote_count(broadcaster_id, Self::platform(), pool)
                .map_err(|_| AnyError::msg("Could not get past emotes")),
            seven_tv::get_emote(emote_id)
                .map_err(|_| AnyError::msg("This emote doesn't exist.")),
            seven_tv::get_user(broadcaster_id)
                .map_err(|_| AnyError::msg("No such user?!")),
        )
        .await?;

        let actual_name = overwritten_name.unwrap_or(&emote.name);

        let Some(stv_set) = stv_user.emote_set else {
            bail!("No 7TV emote set selected");
        };

        if stv_set
            .emotes
            .iter()
            .any(|e| e.id == emote.id || e.name == actual_name)
        {
            return Err(AnyError::msg(
                "The emote or an emote with the same name already exists.",
            ));
        }

        if !allow_unlisted && !emote.listed {
            return Err(AnyError::msg(
                "Attempted to add an unlisted emote, but unlisted emotes aren't allowed.",
            ));
        }

        Ok(EmoteInitialData {
            max_emotes: stv_set.capacity,
            current_emotes: stv_set.emotes.len(),
            history_len: history_len as usize,
            platform_id: stv_set.id,
            emote,
            emotes: stv_set.emotes,
        })
    }

    async fn get_emote_env_data(
        broadcaster_id: &str,
        _platform_id: &Self::PlatformId,
    ) -> AnyResult<EmoteEnvData> {
        let Some(set) = seven_tv::get_user(broadcaster_id).await?.emote_set
        else {
            bail!("No 7TV emote set selected");
        };

        Ok(EmoteEnvData {
            max_emotes: set.capacity,
            current_emotes: set.emotes.len(),
        })
    }

    async fn get_platform_id(
        broadcaster_id: &str,
        _pool: &PgPool,
    ) -> AnyResult<Self::PlatformId> {
        // XXX: this technically returns the emote set id
        seven_tv::get_user(broadcaster_id)
            .map_err(|_| AnyError::msg("No 7TV user found"))
            .await?
            .emote_set
            .ok_or_else(|| anyhow!("No 7TV emote set selected"))
            .map(|s: seven_tv::SevenEmoteSet| s.id)
    }

    async fn get_emotes(
        broadcaster_id: &str,
        _pool: &PgPool,
    ) -> AnyResult<Vec<Self::Emote>> {
        seven_tv::get_user(broadcaster_id)
            .map_err(|_| AnyError::msg("No such user?!"))
            .await?
            .emote_set
            .ok_or_else(|| anyhow!("No 7TV emote set selected"))
            .map(|s| s.emotes)
    }

    async fn get_emote_by_id(
        emote_id: &Self::EmoteId,
    ) -> AnyResult<Self::Emote> {
        seven_tv::get_emote(emote_id).await
    }

    async fn remove_emote(
        platform_id: &Self::PlatformId,
        emote_id: &Self::EmoteId,
        redis_pool: &RedisPool,
    ) -> AnyResult<()> {
        // seventv doesn't error if the emote isn't added,
        // so we have to check if the emote is added in the first place.
        // There's no request to check if the emote is added for the user, so we have to either
        // check the channels the emote is added to or check the users emotes.
        let emote_set = seven_tv::get_emote_set(platform_id).await?;
        if !emote_set.emotes.iter().any(|emote| emote.id == *emote_id) {
            match redis::cmd("DEL")
                .arg(format!("rewards:seventv:cache:{platform_id}:{emote_id}"))
                .query_async::<_, usize>(&mut redis_pool.get().await?)
                .await
            {
                Ok(x) if x > 0 => (),
                _ => return Err(AnyError::msg("Emote not added")),
            }
        }

        seven_tv::remove_emote(platform_id, emote_id).await
    }

    async fn add_emote(
        platform_id: &Self::PlatformId,
        emote_id: &Self::EmoteId,
        overwritten_name: Option<&str>,
        redis_pool: &RedisPool,
    ) -> AnyResult<()> {
        if let Ok(mut conn) = redis_pool.get().await {
            redis::cmd("SETEX")
                .arg(format!("rewards:seventv:cache:{platform_id}:{emote_id}"))
                .arg(90)
                .arg(1)
                .query_async::<_, ()>(&mut conn)
                .await
                .ok();
        }

        seven_tv::add_emote(platform_id, emote_id, overwritten_name).await
    }

    async fn remove_emote_from_broadcaster(
        broadcaster_id: &str,
        emote_id: &str,
        _pool: &PgPool,
        _redis: &RedisPool,
    ) -> AnyResult<String> {
        let user = seven_tv::get_user(broadcaster_id).await?;
        let Some(ref emote_set) = user.emote_set else {
            bail!("No 7TV emote set is selected");
        };

        let (_, emote) = futures::future::try_join(
            seven_tv::remove_emote(&emote_set.id, emote_id),
            seven_tv::get_emote(emote_id),
        )
        .await?;

        Ok(emote.name)
    }

    fn format_emote_url(emote_id: &str) -> String {
        format!("https://cdn.7tv.app/emote/{}/4x", emote_id)
    }

    fn format_emote_page(emote_id: &str) -> String {
        format!("https://7tv.app/emotes/{}", emote_id)
    }
}
