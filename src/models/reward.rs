use crate::services::sql::SqlError;
use actix_web::{HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, PgPool};
use twitch_api2::helix::points::CreateCustomRewardResponse;

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
            Json(self.data.clone()) as _
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
