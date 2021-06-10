use crate::constants::{TWITCH_CLIENT_ID, TWITCH_CLIENT_SECRET};
use crate::services::sql::SqlResult;
use actix_web::{HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, PgPool};
use std::convert::TryFrom;
use std::pin::Pin;
use std::time::Duration;
use twitch_api2::helix::points::CreateCustomRewardResponse;
use twitch_api2::twitch_oauth2::{AccessToken, ClientId, ClientSecret, RefreshToken, UserToken};

#[derive(Serialize, Deserialize, FromRow)]
pub struct Reward {
    pub id: String,
    pub user_id: String,
    pub data: Json<RewardData>,
    pub live_delay: Option<String>,
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
    pub live_delay: Option<String>,
    pub access_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum RewardData {
    Timeout(String),
    SubOnly(String),
    EmoteOnly(String),
    BttvSwap(()),
    FfzSwap(()),
    BttvSlot(SlotRewardData),
    FfzSlot(SlotRewardData),
    SpotifySkip(()),
    SpotifyQueue(SpotifyPlayOptions),
    SpotifyPlay(SpotifyPlayOptions),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlotRewardData {
    pub slots: usize,
    pub expiration: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpotifyPlayOptions {
    pub allow_explicit: bool,
}

impl Responder for Reward {
    fn respond_to(self, _req: &HttpRequest) -> HttpResponse {
        HttpResponse::Ok().json(&self)
    }
}

impl Reward {
    pub fn from_response(
        res: &CreateCustomRewardResponse,
        data: RewardData,
        live_delay: Option<String>,
    ) -> Self {
        Self {
            user_id: res.broadcaster_id.clone(),
            data: Json(data),
            id: res.id.clone(),
            live_delay,
        }
    }

    pub async fn get_by_id(id: &str, pool: &PgPool) -> SqlResult<Reward> {
        // language=PostgreSQL
        let reward: Self = sqlx::query_as!(
            Reward,
            r#"
            SELECT id, user_id, data as "data: Json<RewardData>", live_delay
            FROM rewards
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(reward)
    }

    pub async fn get_all_for_user(user_id: &str, pool: &PgPool) -> SqlResult<Vec<Reward>> {
        // language=PostgreSQL
        let rewards: Vec<Self> = sqlx::query_as!(
            Reward,
            r#"
            SELECT id, user_id, data as "data: Json<RewardData>", live_delay
            FROM rewards
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(rewards)
    }

    pub async fn get_all_live_for_user(user_id: &str, pool: &PgPool) -> SqlResult<Vec<LiveReward>> {
        // language=PostgreSQL
        let rewards = sqlx::query_as!(
            LiveReward,
            r#"
            SELECT id, user_id, live_delay
            FROM rewards
            WHERE user_id = $1 AND live_delay is not null
            "#,
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
            r#"
            SELECT id, user_id, live_delay
            FROM rewards
            WHERE user_id = $1 AND live_delay is not null AND unpause_at is not null
            "#,
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(rewards)
    }

    pub async fn get_all_pending_live(pool: &PgPool) -> SqlResult<Vec<LiveRewardAT>> {
        // language=PostgreSQL
        let rewards = sqlx::query_as!(
            LiveRewardAT,
            r#"
            SELECT rewards.id, user_id, live_delay, access_token
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
            "INSERT INTO rewards (id, user_id, data, live_delay) VALUES ($1, $2, $3, $4)",
            self.id,
            self.user_id,
            Json(&self.data) as _,
            self.live_delay
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn update(&self, pool: &PgPool) -> SqlResult<()> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!(
            "UPDATE rewards SET data=$2, live_delay = $3 WHERE id=$1",
            self.id,
            Json(self.data.clone()) as _,
            self.live_delay
        )
        .execute(&mut tx)
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
        .execute(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn delete(id: &str, pool: &PgPool) -> SqlResult<()> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!("DELETE FROM rewards WHERE id = $1", id)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }
}

#[derive(FromRow)]
pub struct RewardToUpdate {
    broadcaster_id: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
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
            SELECT u.id as broadcaster_id, access_token, refresh_token, rewards.id as reward_id
            FROM rewards
                LEFT JOIN users u on u.id = rewards.user_id
        "#
        )
        .fetch(pool)
    }
}

impl TryFrom<RewardToUpdate> for (String, UserToken) {
    type Error = ();
    fn try_from(r: RewardToUpdate) -> Result<Self, Self::Error> {
        if let (Some(access_token), Some(refresh_token), Some(broadcaster_id)) =
            (r.access_token, r.refresh_token, r.broadcaster_id)
        {
            Ok((
                r.reward_id,
                UserToken::from_existing_unchecked(
                    AccessToken::new(access_token),
                    RefreshToken::new(refresh_token),
                    ClientId::new(TWITCH_CLIENT_ID.to_string()),
                    ClientSecret::new(TWITCH_CLIENT_SECRET.to_string()),
                    String::new(),
                    broadcaster_id,
                    None,
                    Some(Duration::from_secs(1000)),
                ),
            ))
        } else {
            Err(())
        }
    }
}

impl TryFrom<LiveRewardAT> for (LiveReward, UserToken) {
    type Error = ();

    fn try_from(reward: LiveRewardAT) -> Result<Self, Self::Error> {
        if let Some(access_token) = reward.access_token {
            Ok((
                LiveReward {
                    id: reward.id,
                    user_id: reward.user_id.clone(),
                    live_delay: reward.live_delay,
                },
                UserToken::from_existing_unchecked(
                    AccessToken::new(access_token),
                    None,
                    ClientId::new(TWITCH_CLIENT_ID.to_string()),
                    None,
                    String::new(),
                    reward.user_id,
                    None,
                    None,
                ),
            ))
        } else {
            Err(())
        }
    }
}
