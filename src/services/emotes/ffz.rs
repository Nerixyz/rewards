use crate::{
    models::{emote::SlotPlatform, swap_emote::SwapEmote},
    services::{
        emotes::{Emote, EmoteEnvData, EmoteId, EmoteInitialData, EmoteRW},
        ffz::requests as ffz,
    },
};
use anyhow::{Error as AnyError, Result as AnyResult};
use async_trait::async_trait;
use futures::TryFutureExt;
use sqlx::PgPool;

pub struct FfzEmotes {
    _private: usize,
}

impl Emote<usize> for ffz::FfzEmote {
    fn id(&self) -> &usize {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn into_name(self) -> String {
        self.name
    }
}

impl EmoteId for usize {
    fn from_db(db: &str) -> AnyResult<Self> {
        let parsed = db.parse::<Self>()?;
        Ok(parsed)
    }
}

#[async_trait]
impl EmoteRW for FfzEmotes {
    type PlatformId = usize;
    type Emote = ffz::FfzEmote;
    type EmoteId = usize;

    fn platform() -> SlotPlatform {
        SlotPlatform::Ffz
    }

    async fn get_check_initial_data(
        broadcaster_id: &str,
        emote_id: &str,
        pool: &PgPool,
    ) -> AnyResult<EmoteInitialData<usize, ffz::FfzEmote>> {
        let (ffz_user, ffz_emote, ffz_room, ffz_history) = futures::future::try_join4(
            ffz::get_user(broadcaster_id).map_err(|e| {
                log::warn!("err: {}", e);
                AnyError::msg("No such ffz-user")
            }),
            ffz::get_emote(emote_id).map_err(|e| {
                log::warn!("err: {}", e);
                AnyError::msg("No such emote")
            }),
            ffz::get_room(broadcaster_id).map_err(|e| {
                log::warn!("err: {}", e);
                AnyError::msg("No such ffz-room")
            }),
            SwapEmote::emote_count(broadcaster_id, Self::platform(), pool).map_err(|e| {
                log::warn!("err: {}", e);
                AnyError::msg("No emote-count?!")
            }),
        )
        .await?;

        let room_emotes: Vec<ffz::FfzEmote> = ffz_room
            .sets
            .into_iter()
            .map(|s| s.1.emoticons)
            .flatten()
            .collect();

        if room_emotes
            .iter()
            .any(|e| e.id == ffz_emote.id || e.name == ffz_emote.name)
        {
            return Err(AnyError::msg("The emote is already added"));
        }

        Ok(EmoteInitialData {
            max_emotes: ffz_user.max_emoticons,
            current_emotes: room_emotes.len(),
            history_len: ffz_history as usize,
            platform_id: ffz_room.room._id,
            emote: ffz_emote,
            emotes: room_emotes,
        })
    }

    async fn get_emote_env_data(
        broadcaster_id: &str,
        _platform_id: &usize,
    ) -> AnyResult<EmoteEnvData> {
        let (ffz_user, ffz_room) = futures::future::try_join(
            ffz::get_user(broadcaster_id).map_err(|e| {
                log::warn!("err: {}", e);
                AnyError::msg("No such ffz-user")
            }),
            ffz::get_room(broadcaster_id).map_err(|e| {
                log::warn!("err: {}", e);
                AnyError::msg("No such ffz-room")
            }),
        )
        .await?;

        let room_emotes: Vec<ffz::FfzEmote> = ffz_room
            .sets
            .into_iter()
            .map(|s| s.1.emoticons)
            .flatten()
            .collect();

        Ok(EmoteEnvData {
            current_emotes: room_emotes.len(),
            max_emotes: ffz_user.max_emoticons,
        })
    }

    async fn get_platform_id(broadcaster_id: &str, _pool: &PgPool) -> AnyResult<Self::PlatformId> {
        // TODO: save room id in db
        ffz::get_room(broadcaster_id)
            .await
            .map(|room| room.room._id)
    }

    async fn get_emote_by_id(emote_id: &usize) -> AnyResult<ffz::FfzEmote> {
        ffz::get_emote(emote_id).await
    }

    async fn remove_emote(platform_id: &usize, emote_id: &usize) -> AnyResult<()> {
        ffz::delete_emote(*platform_id, *emote_id).await
    }

    async fn add_emote(platform_id: &usize, emote_id: &usize) -> AnyResult<()> {
        ffz::add_emote(*platform_id, *emote_id).await
    }

    async fn remove_emote_from_broadcaster(
        broadcaster_id: &str,
        emote_id: &str,
        _pool: &PgPool,
    ) -> AnyResult<String> {
        let room = ffz::get_room(broadcaster_id).await?;
        let (_, emote) = futures::future::try_join(
            Self::remove_emote(&room.room._id, &emote_id.parse::<Self::EmoteId>()?),
            ffz::get_emote(emote_id),
        )
        .await?;

        Ok(emote.name)
    }
}
