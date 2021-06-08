use crate::services::sql::SqlResult;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

#[derive(FromRow, Debug)]
pub struct Slot {
    pub id: i32,
    pub user_id: String,
    pub reward_id: String,
    pub emote_id: Option<String>,
    pub expires: Option<DateTime<Utc>>,
    pub platform: SlotPlatform,
}

#[derive(sqlx::Type, Debug)]
#[sqlx(type_name = "slot_platform", rename_all = "snake_case")]
pub enum SlotPlatform {
    Bttv,
    Ffz,
    #[sqlx(rename = "7tv")]
    SevenTv,
}

impl Slot {
    pub async fn create(
        user_id: &str,
        reward_id: &str,
        platform: SlotPlatform,
        pool: &PgPool,
    ) -> SqlResult<()> {
        // language=PostgreSQL
        sqlx::query!(
            r#"INSERT INTO slots (user_id, reward_id, platform) VALUES ($1, $2, $3)"#,
            user_id,
            reward_id,
            platform as _,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_available_slots(
        user_id: &str,
        reward_id: &str,
        pool: &PgPool,
    ) -> SqlResult<Vec<Self>> {
        // language=PostgreSQL
        let available = sqlx::query_as!(
            Self,
            r#"
            SELECT id, user_id, emote_id, expires, reward_id, platform as "platform: _" FROM slots
            WHERE reward_id = $1 and user_id = $2 and emote_id is null and expires is null
        "#,
            reward_id,
            user_id
        )
        .fetch_all(pool)
        .await?;
        Ok(available)
    }

    pub async fn get_n_available_slots(
        user_id: &str,
        reward_id: &str,
        pool: &PgPool,
    ) -> SqlResult<i64> {
        // language=PostgreSQL
        let n_available = sqlx::query_scalar!(
            r#"
            SELECT count(*) FROM slots
            WHERE reward_id = $1 and user_id = $2 and emote_id is null and expires is null
        "#,
            reward_id,
            user_id
        )
        .fetch_one(pool)
        .await?;
        Ok(n_available.unwrap_or(0))
    }

    pub async fn get_all_slots(
        user_id: &str,
        reward_id: &str,
        pool: &PgPool,
    ) -> SqlResult<Vec<Self>> {
        // language=PostgreSQL
        let all = sqlx::query_as!(
            Self,
            r#"
            SELECT id, user_id, emote_id, expires, reward_id, platform as "platform: _" FROM slots
            WHERE reward_id = $1 and user_id = $2
        "#,
            reward_id,
            user_id
        )
        .fetch_all(pool)
        .await?;
        Ok(all)
    }

    pub async fn get_pending(pool: &PgPool) -> SqlResult<Vec<Self>> {
        // language=PostgreSQL
        let pending = sqlx::query_as!(Self, r#"
            SELECT id, user_id, emote_id, expires, reward_id, platform as "platform: _" FROM slots
            WHERE emote_id is not null AND expires is not null AND expires < (now() + '1 minute'::interval)
        "#).fetch_all(pool).await?;

        Ok(pending)
    }

    pub async fn update(&self, pool: &PgPool) -> SqlResult<()> {
        // language=PostgreSQL
        sqlx::query!(
            r#"
            UPDATE slots SET emote_id = $2, expires = $3 WHERE id = $1
        "#,
            self.id,
            self.emote_id,
            self.expires
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn clear(id: i32, pool: &PgPool) -> SqlResult<()> {
        // language=PostgreSQL
        sqlx::query!(
            r#"
            UPDATE slots SET emote_id = null, expires = null WHERE id = $1
        "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn remove(id: i32, pool: &PgPool) -> SqlResult<()> {
        // language=PostgreSQL
        sqlx::query!(
            r#"
            DELETE FROM slots WHERE id = $1
        "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
