use crate::models::slot::SlotPlatform;
use crate::models::user::User;
use crate::services::emotes::{Emote, EmoteEnvData, EmoteInitialData, EmoteRW};
use crate::services::seven_tv::{fetch_save_seventv_id, get_or_fetch_id, requests as seven_tv};
use anyhow::{Error as AnyError, Result as AnyResult};
use async_trait::async_trait;
use futures::TryFutureExt;
use sqlx::PgPool;

pub struct SevenTvEmotes {
    _private: (),
}

impl Emote<String> for seven_tv::SevenEmote {
    fn id(&self) -> &String {
        &self.id
    }

    fn name(self) -> String {
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
        pool: &PgPool,
    ) -> AnyResult<EmoteInitialData<Self::PlatformId, Self::Emote, Self::EmoteId>> {
        let this_user = User::get_seventv_data(broadcaster_id, pool)
            .await
            .map_err(|_| AnyError::msg("No internal user"))?;
        let stv_id = if let Some(id) = &this_user.seventv_id {
            id.to_string()
        } else {
            fetch_save_seventv_id(broadcaster_id, &this_user.name, pool)
                .await
                .map_err(|_| AnyError::msg("No such user"))?
        };

        let (emote, stv_user) = futures::future::try_join(
            seven_tv::get_emote(emote_id).map_err(|_| AnyError::msg("This emote doesn't exist.")),
            seven_tv::get_user(&stv_id).map_err(|_| AnyError::msg("No such user?!")),
        )
        .await?;

        if stv_user
            .emotes
            .iter()
            .any(|e| e.id == emote.id || e.name == emote.name)
        {
            return Err(AnyError::msg(
                "The emote or an emote with the same name already exists.",
            ));
        }

        Ok(EmoteInitialData {
            max_emotes: stv_user.emote_slots,
            current_emotes: stv_user.emotes.len(),
            history: this_user.seventv_history.0,
            platform_id: stv_id,
            emote,
            emotes: stv_user.emotes,
        })
    }

    async fn get_emote_env_data(
        _broadcaster_id: &str,
        platform_id: &Self::PlatformId,
    ) -> AnyResult<EmoteEnvData> {
        let user = seven_tv::get_user(platform_id).await?;

        Ok(EmoteEnvData {
            max_emotes: user.emote_slots,
            current_emotes: user.emotes.len(),
        })
    }

    async fn get_emote_by_id(emote_id: &Self::EmoteId) -> AnyResult<Self::Emote> {
        seven_tv::get_emote(emote_id).await
    }

    async fn remove_emote(
        platform_id: &Self::PlatformId,
        emote_id: &Self::EmoteId,
    ) -> AnyResult<()> {
        seven_tv::remove_emote(platform_id, emote_id).await
    }

    async fn add_emote(platform_id: &Self::PlatformId, emote_id: &Self::EmoteId) -> AnyResult<()> {
        seven_tv::add_emote(platform_id, emote_id).await
    }

    async fn save_history(
        broadcaster_id: &str,
        history: Vec<Self::EmoteId>,
        pool: &PgPool,
    ) -> AnyResult<()> {
        Ok(User::set_seventv_history(broadcaster_id, history, pool).await?)
    }

    async fn remove_emote_from_broadcaster(
        broadcaster_id: &str,
        emote_id: &str,
        pool: &PgPool,
    ) -> AnyResult<String> {
        let platform_id = get_or_fetch_id(broadcaster_id, pool).await.map_err(|e| {
            log::warn!("No user-id broadcaster={} error={}", broadcaster_id, e);
            e
        })?;

        let (_, emote) = futures::future::try_join(
            seven_tv::remove_emote(&platform_id, emote_id),
            seven_tv::get_emote(emote_id),
        )
        .await?;

        Ok(emote.name)
    }
}
