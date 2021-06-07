pub mod bttv;
pub mod execute;
pub mod ffz;
pub mod slots;
pub mod swap;

use crate::models::slot::SlotPlatform;
use anyhow::Result as AnyResult;
use async_trait::async_trait;
use sqlx::PgPool;

pub struct EmoteInitialData<I, E, EI> {
    pub max_emotes: usize,
    pub current_emotes: usize,
    pub history: Vec<EI>,
    pub platform_id: I,
    pub emote: E,
    pub emotes: Vec<E>,
}

pub struct EmoteEnvData {
    pub max_emotes: usize,
    pub current_emotes: usize,
}

pub trait Emote<I> {
    fn id(&self) -> &I;
    fn name(self) -> String;
}

pub trait EmoteId {
    fn from_db(db: &str) -> AnyResult<Self>
    where
        Self: Sized;
}

#[async_trait]
pub trait EmoteRW {
    type PlatformId;
    type Emote;
    type EmoteId;
    fn platform() -> SlotPlatform;

    /// Here, the input is checked (e.g. if there are already emotes with the same name) and the impl returns the required data
    async fn get_check_initial_data(
        broadcaster_id: &str,
        emote_id: &str,
        pool: &PgPool,
    ) -> AnyResult<EmoteInitialData<Self::PlatformId, Self::Emote, Self::EmoteId>>;
    async fn get_emote_env_data(
        broadcaster_id: &str,
        platform_id: &Self::PlatformId,
    ) -> AnyResult<EmoteEnvData>;

    async fn get_emote_by_id(emote_id: &Self::EmoteId) -> AnyResult<Self::Emote>;
    async fn remove_emote(
        platform_id: &Self::PlatformId,
        emote_id: &Self::EmoteId,
    ) -> AnyResult<()>;
    async fn add_emote(platform_id: &Self::PlatformId, emote_id: &Self::EmoteId) -> AnyResult<()>;

    async fn save_history(
        broadcaster_id: &str,
        history: Vec<Self::EmoteId>,
        pool: &PgPool,
    ) -> AnyResult<()>;

    async fn remove_emote_from_broadcaster(
        broadcaster_id: &str,
        emote_id: &str,
        pool: &PgPool,
    ) -> AnyResult<String>;
}
