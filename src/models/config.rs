use crate::services::sql::SqlError;
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow, PgPool};
use twitch_irc::login::UserAccessToken;

#[derive(FromRow)]
pub struct ConfigEntry {
    key: String,
    value: Json<ConfigValue>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum ConfigValue {
    UserToken(UserAccessToken),
}

impl ConfigEntry {
    pub async fn update(&self, pool: &PgPool) -> Result<(), SqlError> {
        let mut tx = pool.begin().await?;
        // language=PostgreSQL
        let _ = sqlx::query!(
            "UPDATE config SET value = $2 WHERE key = $1",
            self.key,
            Json(&self.value) as _
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn update_user_token(pool: &PgPool, token: UserAccessToken) -> Result<(), SqlError> {
        Self {
            key: "user_token".to_string(),
            value: Json(ConfigValue::UserToken(token)),
        }
        .update(pool)
        .await
    }

    async fn get(key: &str, pool: &PgPool) -> Result<ConfigEntry, SqlError> {
        // language=PostgreSQL
        let entry: Self = sqlx::query_as!(
            ConfigEntry,
            r#"
            SELECT key, value as "value: Json<ConfigValue>"
            FROM config
            WHERE key = $1
            "#,
            key
        )
        .fetch_one(pool)
        .await?;

        Ok(entry)
    }

    pub async fn get_user_token(pool: &PgPool) -> Result<UserAccessToken, SqlError> {
        let entry = Self::get("user_token", pool).await?;

        match entry.value.0 {
            ConfigValue::UserToken(token) => Ok(token),
        }
    }
}
