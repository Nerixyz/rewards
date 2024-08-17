use crate::emote::SlotPlatform;
use chrono::{DateTime, Utc};
use config::CONFIG;
use errors::sql::SqlResult;
use futures::Stream;
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow, PgPool};
use std::{pin::Pin, time::Duration};
use twitch_api::{
    helix::points::CreateCustomRewardResponse,
    twitch_oauth2::{
        AccessToken, ClientId, ClientSecret, RefreshToken, UserToken,
    },
    types::{Nickname, UserId},
};

#[derive(Serialize, Deserialize, FromRow)]
pub struct Reward {
    pub id: String,
    pub user_id: String,
    pub data: Json<RewardData>,
    pub live_delay: Option<String>,
    pub auto_accept: bool,
}

#[derive(FromRow)]
pub struct LiveReward {
    pub id: String,
    pub user_id: String,
    pub live_delay: Option<String>,
}

#[derive(FromRow)]
pub struct LiveRewardAT {
    pub id: String,
    pub user_id: String,
    pub live_delay: String,
    pub access_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, derive_more::Display)]
// the tags are used in the debug command.
#[serde(tag = "type", content = "data")]
pub enum RewardData {
    #[display("timeout")]
    Timeout(TimeoutRewardData),
    #[display("mode::sub")]
    SubOnly(String),
    #[display("mode::emote")]
    EmoteOnly(String),
    #[display("swap::bttv")]
    BttvSwap(#[serde(default)] SwapRewardData),
    #[display("swap::ffz")]
    FfzSwap(#[serde(default)] SwapRewardData),
    #[display("swap::7tv")]
    SevenTvSwap(#[serde(default)] SwapRewardData),
    #[display("slot::bttv")]
    BttvSlot(SlotRewardData),
    #[display("slot::ffz")]
    FfzSlot(SlotRewardData),
    #[display("slot::7tv")]
    SevenTvSlot(SlotRewardData),
    #[display("spotify::skip")]
    SpotifySkip(()),
    #[display("spotify::queue")]
    SpotifyQueue(SpotifyPlayOptions),
    #[display("spotify::play")]
    SpotifyPlay(SpotifyPlayOptions),
    #[display("rem-emote")]
    RemEmote(RemEmoteRewardData),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeoutRewardData {
    pub duration: String,
    #[serde(default)]
    pub vip: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlotRewardData {
    pub slots: usize,
    pub expiration: String,
    #[serde(default = "always_true")]
    pub allow_unlisted: bool,
    /// Only controls the "ok" case
    /// Errors are always printed
    #[serde(default = "always_true")]
    pub reply: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SwapRewardData {
    pub limit: Option<u16>,
    #[serde(default = "always_true")]
    pub allow_unlisted: bool,
    /// Only controls the "ok" case
    /// Errors are always printed
    #[serde(default = "always_true")]
    pub reply: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemEmoteRewardData {
    pub platform: SlotPlatform,
    /// Only controls the "ok" case
    /// Errors are always printed
    #[serde(default = "always_true")]
    pub reply: bool,
}

impl Default for SwapRewardData {
    fn default() -> Self {
        Self {
            limit: Default::default(),
            allow_unlisted: true,
            reply: true,
        }
    }
}

const fn always_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpotifyPlayOptions {
    pub allow_explicit: bool,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct RewardDataOnly {
    pub data: Json<RewardData>,
}

impl Reward {
    pub fn from_response(
        res: &CreateCustomRewardResponse,
        data: RewardData,
        live_delay: Option<String>,
        auto_accept: bool,
    ) -> Self {
        Self {
            user_id: res.broadcaster_id.clone().take(),
            data: Json(data),
            id: res.id.clone().take(),
            live_delay,
            auto_accept,
        }
    }

    pub async fn get_by_id(id: &str, pool: &PgPool) -> SqlResult<Reward> {
        // language=PostgreSQL
        let reward: Self = sqlx::query_as!(
            Reward,
            r#"
            SELECT id, user_id, data as "data: Json<RewardData>", live_delay, auto_accept
            FROM rewards
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(reward)
    }

    pub async fn get_all_for_user(
        user_id: impl AsRef<str>,
        pool: &PgPool,
    ) -> SqlResult<Vec<Reward>> {
        // language=PostgreSQL
        let rewards: Vec<Self> = sqlx::query_as!(
            Reward,
            r#"
            SELECT id, user_id, data as "data: Json<RewardData>", live_delay, auto_accept
            FROM rewards
            WHERE user_id = $1
            "#,
            user_id.as_ref()
        )
        .fetch_all(pool)
        .await?;

        Ok(rewards)
    }

    pub async fn get_swap_limit_for_user(
        user_id: &str,
        platform: SlotPlatform,
        pool: &PgPool,
    ) -> SqlResult<Option<usize>> {
        let reward_type = platform.swap_reward_name();
        // language=PostgreSQL
        let data: Vec<RewardDataOnly> = sqlx::query_as!(
            RewardDataOnly,
            r#"
            SELECT data as "data: Json<RewardData>"
            FROM rewards
            WHERE user_id = $1 AND data ->> 'type' = $2
            "#,
            user_id,
            reward_type
        )
        .fetch_all(pool)
        .await?;

        let data = data
            .into_iter()
            .filter_map(|r| match r.data.0 {
                RewardData::FfzSwap(d) => Some(d),
                RewardData::BttvSwap(d) => Some(d),
                RewardData::SevenTvSwap(d) => Some(d),
                _ => None,
            })
            .try_fold(0, |acc, swap| match (acc, &swap.limit) {
                (acc, Some(lim)) => Some(acc + *lim as usize),
                _ => None,
            });
        Ok(data)
    }

    pub async fn get_all_live_for_user(
        user_id: &str,
        pool: &PgPool,
    ) -> SqlResult<Vec<LiveReward>> {
        // language=PostgreSQL
        let rewards = sqlx::query_as!(
            LiveReward,
            "
            SELECT id, user_id, live_delay
            FROM rewards
            WHERE user_id = $1 AND live_delay is not null
            ",
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(rewards)
    }

    pub async fn get_all_pending_live_for_user(
        user_id: &str,
        pool: &PgPool,
    ) -> SqlResult<Vec<LiveReward>> {
        // language=PostgreSQL
        let rewards = sqlx::query_as!(
            LiveReward,
            "
            SELECT id, user_id, live_delay
            FROM rewards
            WHERE user_id = $1 AND live_delay is not null AND unpause_at is not null
            ",
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(rewards)
    }

    pub async fn get_all_pending_live(
        pool: &PgPool,
    ) -> SqlResult<Vec<LiveRewardAT>> {
        // language=PostgreSQL
        let rewards = sqlx::query_as!(
            LiveRewardAT,
            r#"
            SELECT rewards.id as "id!", user_id as "user_id!", live_delay as "live_delay!", access_token as "access_token!"
            FROM rewards
            LEFT JOIN users u on u.id = rewards.user_id
            WHERE live_delay is not null AND unpause_at is not null
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(rewards)
    }

    pub async fn create(&self, pool: &PgPool) -> SqlResult<()> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!(
            "INSERT INTO rewards (id, user_id, data, live_delay, auto_accept) VALUES ($1, $2, $3, $4, $5)",
            self.id,
            self.user_id,
            Json(&self.data) as _,
            self.live_delay,
            self.auto_accept,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn update(&self, pool: &PgPool) -> SqlResult<()> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!(
            "UPDATE rewards SET data=$2, live_delay = $3, auto_accept = $4 WHERE id=$1",
            self.id,
            Json(self.data.clone()) as _,
            self.live_delay,
            self.auto_accept,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn set_unpause_at(
        id: &str,
        unpause_at: Option<DateTime<Utc>>,
        pool: &PgPool,
    ) -> SqlResult<()> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!(
            "UPDATE rewards SET unpause_at = $2 WHERE id=$1",
            id,
            unpause_at
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn delete(id: &str, pool: &PgPool) -> SqlResult<()> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!("DELETE FROM rewards WHERE id = $1", id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }
}

#[derive(FromRow)]
pub struct RewardToUpdate {
    broadcaster_id: String,
    access_token: String,
    refresh_token: String,
    reward_id: String,
}

impl RewardToUpdate {
    pub fn get_all<'a>(
        pool: &'a PgPool,
    ) -> Pin<Box<dyn Stream<Item = Result<Self, sqlx::Error>> + 'a>> {
        // language=PostgreSQL
        sqlx::query_as!(
            RewardToUpdate,
            r#"
            SELECT 
                u.id as "broadcaster_id!",
                access_token as "access_token!",
                refresh_token as "refresh_token!",
                rewards.id as "reward_id!"
            FROM rewards
                LEFT JOIN users u on u.id = rewards.user_id
            WHERE rewards.auto_accept
        "#
        )
        .fetch(pool)
    }
}

impl From<RewardToUpdate> for (String, UserToken) {
    fn from(r: RewardToUpdate) -> Self {
        let (access_token, refresh_token, broadcaster_id) =
            (r.access_token, r.refresh_token, r.broadcaster_id);
        (
            r.reward_id,
            UserToken::from_existing_unchecked(
                AccessToken::new(access_token),
                RefreshToken::new(refresh_token),
                ClientId::new(CONFIG.twitch.client_id.to_string()),
                ClientSecret::new(CONFIG.twitch.client_secret.to_string()),
                Nickname::from(""),
                UserId::from(broadcaster_id),
                None,
                Some(Duration::from_secs(1000)),
            ),
        )
    }
}

impl From<LiveRewardAT> for (LiveReward, UserToken) {
    fn from(reward: LiveRewardAT) -> Self {
        (
            LiveReward {
                id: reward.id,
                user_id: reward.user_id.clone(),
                live_delay: Some(reward.live_delay),
            },
            UserToken::from_existing_unchecked(
                AccessToken::new(reward.access_token),
                None,
                ClientId::new(CONFIG.twitch.client_id.to_string()),
                None,
                Nickname::from(""),
                UserId::from(reward.user_id),
                None,
                None,
            ),
        )
    }
}
