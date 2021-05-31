use crate::services::sql::SqlError;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

#[derive(FromRow, Debug)]
pub struct BttvSlot {
    pub id: i32,
    pub user_id: String,
    pub reward_id: String,
    pub emote_id: Option<String>,
    pub expires: Option<DateTime<Utc>>,
}

impl BttvSlot {
    pub async fn create(user_id: &str, reward_id: &str, pool: &PgPool) -> Result<(), SqlError> {
        // language=PostgreSQL
        sqlx::query!(
            r#"INSERT INTO bttv_slots (user_id, reward_id) VALUES ($1, $2)"#,
            user_id,
            reward_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_available_slots(
        user_id: &str,
        reward_id: &str,
        pool: &PgPool,
    ) -> Result<Vec<Self>, SqlError> {
        // language=PostgreSQL
        let available = sqlx::query_as!(
            Self,
            r#"
            SELECT * FROM bttv_slots
            WHERE reward_id = $1 and user_id = $2 and emote_id is null and expires is null
        "#,
            reward_id,
            user_id
        )
        .fetch_all(pool)
        .await?;
        Ok(available)
    }

    pub async fn get_all_slots(
        user_id: &str,
        reward_id: &str,
        pool: &PgPool,
    ) -> Result<Vec<Self>, SqlError> {
        // language=PostgreSQL
        let all = sqlx::query_as!(
            Self,
            r#"
            SELECT * FROM bttv_slots
            WHERE reward_id = $1 and user_id = $2
        "#,
            reward_id,
            user_id
        )
        .fetch_all(pool)
        .await?;
        Ok(all)
    }

    pub async fn get_pending(pool: &PgPool) -> Result<Vec<Self>, SqlError> {
        // language=PostgreSQL
        let pending = sqlx::query_as!(Self, r#"
            SELECT * FROM bttv_slots
            WHERE emote_id is not null AND expires is not null AND expires < (now() +  '5 minutes'::interval)
        "#).fetch_all(pool).await?;

        Ok(pending)
    }

    pub async fn update(&self, pool: &PgPool) -> Result<(), SqlError> {
        // language=PostgreSQL
        sqlx::query!(
            r#"
            UPDATE bttv_slots SET emote_id = $2, expires = $3 WHERE id = $1
        "#,
            self.id,
            self.emote_id,
            self.expires
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn clear(id: i32, pool: &PgPool) -> Result<(), SqlError> {
        // language=PostgreSQL
        sqlx::query!(
            r#"
            UPDATE bttv_slots SET emote_id = null, expires = null WHERE id = $1
        "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn remove(id: i32, pool: &PgPool) -> Result<(), SqlError> {
        // language=PostgreSQL
        sqlx::query!(
            r#"
            DELETE FROM bttv_slots WHERE id = $1
        "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
