use sqlx::{types::Json, FromRow, PgPool};
use serde::{Deserialize, Serialize};
use twitch_irc::login::UserAccessToken;
use crate::services::sql::SqlError;

#[derive(FromRow)]
pub struct ConfigEntry {
    key: String,
    value: Json<ConfigValue>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum ConfigValue {
    UserToken(UserAccessToken)
}

impl ConfigEntry {
    pub async fn insert(&self, pool: &PgPool) -> Result<(), SqlError> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!("INSERT INTO config (key, value) VALUES ($1, $2)", self.key, Json(&self.value) as _)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn update(&self, pool: &PgPool) -> Result<(), SqlError> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!("UPDATE config SET value = $2 WHERE key = $1", self.key, Json(&self.value) as _)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn update_token(pool: &PgPool, token: &UserAccessToken) -> Result<(), SqlError> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!("UPDATE config SET value = $1 WHERE key = 'user_token'", Json(token) as _)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn get(key: &str, pool: &PgPool) -> Result<ConfigEntry, SqlError> {
        // language=PostgreSQL
        let entry: Self = sqlx::query_as!(ConfigEntry, r#"
            SELECT key, value as "value: Json<ConfigValue>"
            FROM config
            WHERE key = $1
            "#, key)
            .fetch_one(pool)
            .await?;

        Ok(entry)
    }

    pub async fn get_user_token(pool: &PgPool) -> Result<UserAccessToken, SqlError> {
        let entry = Self::get("user_token", pool).await?;
        let value = entry.value.0;

        match value {
            ConfigValue::UserToken(token) => Ok(token),
            _ => Err(SqlError::Internal) // TODO
        }
    }
}