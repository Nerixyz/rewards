pub mod bttv;
pub mod execute;
pub mod ffz;
pub mod seven_tv;
pub mod slots;
pub mod swap;

use anyhow::Result as AnyResult;
use async_trait::async_trait;
use models::emote::SlotPlatform;
use sqlx::PgPool;

pub struct EmoteInitialData<I, E> {
    pub max_emotes: usize,
    pub current_emotes: usize,
    pub platform_id: I,
    pub history_len: usize,
    pub emote: E,
    pub emotes: Vec<E>,
}

pub struct EmoteEnvData {
    pub max_emotes: usize,
    pub current_emotes: usize,
}

pub trait Emote<I> {
    fn id(&self) -> &I;
    fn name(&self) -> &str;
    fn into_name(self) -> String;
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
    ) -> AnyResult<EmoteInitialData<Self::PlatformId, Self::Emote>>;
    async fn get_emote_env_data(
        broadcaster_id: &str,
        platform_id: &Self::PlatformId,
    ) -> AnyResult<EmoteEnvData>;
    async fn get_platform_id(broadcaster_id: &str, pool: &PgPool) -> AnyResult<Self::PlatformId>;

    async fn get_emote_by_id(emote_id: &Self::EmoteId) -> AnyResult<Self::Emote>;
    async fn remove_emote(
        platform_id: &Self::PlatformId,
        emote_id: &Self::EmoteId,
    ) -> AnyResult<()>;
    async fn add_emote(platform_id: &Self::PlatformId, emote_id: &Self::EmoteId) -> AnyResult<()>;

    async fn remove_emote_from_broadcaster(
        broadcaster_id: &str,
        emote_id: &str,
        pool: &PgPool,
    ) -> AnyResult<String>;

    fn format_emote_url(emote_id: &str) -> String;
    fn format_emote_page(emote_id: &str) -> String;
}
