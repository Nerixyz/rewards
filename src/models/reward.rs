use crate::constants::{TWITCH_CLIENT_ID, TWITCH_CLIENT_SECRET};
use crate::services::sql::SqlError;
use actix_web::{HttpRequest, HttpResponse, Responder};
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum RewardData {
    Timeout(String),
    SubOnly(String),
    EmoteOnly(String),
    BttvSwap(()),
    FfzSwap(()),
    BttvSlot(BttvSlotRewardData),
    SpotifySkip(()),
    SpotifyQueue(SpotifyPlayOptions),
    SpotifyPlay(SpotifyPlayOptions),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BttvSlotRewardData {
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
    pub fn from_response(res: &CreateCustomRewardResponse, data: RewardData) -> Self {
        Self {
            user_id: res.broadcaster_id.clone(),
            data: Json(data),
            id: res.id.clone(),
        }
    }

    pub async fn get_by_id(id: &str, pool: &PgPool) -> Result<Reward, SqlError> {
        // language=PostgreSQL
        let reward: Self = sqlx::query_as!(
            Reward,
            r#"
            SELECT id, user_id, data as "data: Json<RewardData>"
            FROM rewards
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(reward)
    }

    pub async fn get_all_for_user(user_id: &str, pool: &PgPool) -> Result<Vec<Reward>, SqlError> {
        // language=PostgreSQL
        let rewards: Vec<Self> = sqlx::query_as!(
            Reward,
            r#"
            SELECT id, user_id, data as "data: Json<RewardData>"
            FROM rewards
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(rewards)
    }

    pub async fn create(&self, pool: &PgPool) -> Result<(), SqlError> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!(
            "INSERT INTO rewards (id, user_id, data) VALUES ($1, $2, $3)",
            self.id,
            self.user_id,
            Json(&self.data) as _
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn update(&self, pool: &PgPool) -> Result<(), SqlError> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!(
            "UPDATE rewards SET data=$2 WHERE id=$1",
            self.id,
            Json(self.data.clone()) as _
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn delete(id: &str, pool: &PgPool) -> Result<(), SqlError> {
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
