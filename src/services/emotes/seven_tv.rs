use crate::services::emotes::EmoteRW;
use sqlx::PgPool;
use crate::models::slot::SlotPlatform;
use anyhow::{Result as AnyResult, Error as AnyError};

pub struct SevenTvEmotes {
    _private: ()
}

impl EmoteRW for SevenTvEmotes {
    type PlatformId = String;
    type Emote = ();
    type EmoteId = String;

    fn platform() -> SlotPlatform {
        todo!()
    }

    async fn get_check_initial_data(broadcaster_id: &str, emote_id: &str, pool: &PgPool) -> AnyResult<EmoteInitialData<Self::PlatformId, Self::Emote, Self::EmoteId>> {
        todo!()
    }

    async fn get_emote_env_data(broadcaster_id: &str, platform_id: &Self::PlatformId) -> AnyResult<EmoteEnvData> {
        todo!()
    }

    async fn get_emote_by_id(emote_id: &Self::EmoteId) -> AnyResult<Self::Emote> {
        todo!()
    }

    async fn remove_emote(platform_id: &Self::PlatformId, emote_id: &Self::EmoteId) -> AnyResult<()> {
        todo!()
    }

    async fn add_emote(platform_id: &Self::PlatformId, emote_id: &Self::EmoteId) -> AnyResult<()> {
        todo!()
    }

    async fn save_history(broadcaster_id: &str, history: Vec<Self::EmoteId>, pool: &PgPool) -> AnyResult<()> {
        todo!()
    }

    async fn remove_emote_from_broadcaster(broadcaster_id: &str, emote_id: &str, pool: &PgPool) -> AnyResult<String> {
        todo!()
    }
}