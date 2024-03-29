use crate::emote::SlotPlatform;
use chrono::{DateTime, Utc};
use errors::sql::SqlResult;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct Slot {
    pub id: i32,
    pub user_id: String,
    pub reward_id: String,
    pub emote_id: Option<String>,
    pub expires: Option<DateTime<Utc>>,
    pub platform: SlotPlatform,
    pub name: Option<String>,
    pub added_by: Option<String>,
    pub added_at: Option<DateTime<Utc>>,
}

#[derive(FromRow)]
pub struct SlotOccupation {
    pub total: Option<i64>,
    pub available: Option<i64>,
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
            "INSERT INTO slots (user_id, reward_id, platform) VALUES ($1, $2, $3)",
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
            SELECT id, user_id, emote_id, expires, reward_id, platform as "platform: _", name, added_at, added_by FROM slots
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
            "
            SELECT count(*) FROM slots
            WHERE reward_id = $1 and user_id = $2 and emote_id is null and expires is null
        ",
            reward_id,
            user_id
        )
        .fetch_one(pool)
        .await?;
        Ok(n_available.unwrap_or(0))
    }

    pub async fn get_n_available_slots_for_platform(
        user_id: &str,
        platform: SlotPlatform,
        pool: &PgPool,
    ) -> SqlResult<i64> {
        // language=PostgreSQL
        let n_available = sqlx::query_scalar!(
            "
            SELECT count(*) FROM slots
            WHERE platform = $1 and user_id = $2 and emote_id is null and expires is null
        ",
            platform as _,
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
            SELECT id, user_id, emote_id, expires, reward_id, platform as "platform: _", name, added_at, added_by FROM slots
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
            SELECT id, user_id, emote_id, expires, reward_id, platform as "platform: _", name, added_at, added_by FROM slots
            WHERE emote_id is not null AND expires is not null AND expires < (now() + '1 minute'::interval)
        "#).fetch_all(pool).await?;

        Ok(pending)
    }

    pub async fn get_slot_by_emote_name(
        user_id: &str,
        name: &str,
        pool: &PgPool,
    ) -> SqlResult<Option<Self>> {
        // language=PostgreSQL
        let slot = sqlx::query_as!(Self, r#"
            SELECT id, user_id, emote_id, expires, reward_id, platform as "platform: _", name, added_at, added_by FROM slots
            WHERE user_id = $1 AND lower(name) = lower($2)
        "#, user_id, name).fetch_optional(pool).await?;

        Ok(slot)
    }

    pub async fn get_slot_by_emote_id(
        user_id: &str,
        emote_id: &str,
        platform: SlotPlatform,
        pool: &PgPool,
    ) -> SqlResult<Option<Self>> {
        // language=PostgreSQL
        let slot = sqlx::query_as!(Self, r#"
            SELECT id, user_id, emote_id, expires, reward_id, platform as "platform: _", name, added_at, added_by FROM slots
            WHERE user_id = $1 AND emote_id = $2 and platform = $3
        "#, user_id, emote_id, platform as _).fetch_optional(pool).await?;

        Ok(slot)
    }

    pub async fn get_occupation(
        user_id: &str,
        pool: &PgPool,
    ) -> SqlResult<SlotOccupation> {
        // language=PostgreSQL
        let occupation = sqlx::query_as!(SlotOccupation,
            "
            SELECT count(*) as available, (SELECT count(*) FROM slots WHERE user_id = $1) as total FROM slots
            WHERE user_id = $1 and emote_id is null and expires is null
        ",
            user_id
        )
            .fetch_one(pool)
            .await?;
        Ok(occupation)
    }

    pub async fn get_occupied_emotes(
        user_id: &str,
        pool: &PgPool,
    ) -> SqlResult<Vec<String>> {
        // language=PostgreSQL
        let emotes = sqlx::query_scalar!(
            "
            SELECT name FROM slots
            WHERE user_id = $1 and emote_id is not null and name is not null
        ",
            user_id
        )
        .fetch_all(pool)
        .await?;
        Ok(emotes.into_iter().flatten().collect())
    }

    pub async fn get_occupied(
        user_id: &str,
        pool: &PgPool,
    ) -> SqlResult<Vec<Self>> {
        // language=PostgreSQL
        let emotes = sqlx::query_as!(
            Self,
            r#"
            SELECT id, user_id, emote_id, expires, reward_id, platform as "platform: _", name, added_at, added_by FROM slots
            WHERE user_id = $1 and emote_id is not null and name is not null
        "#,
            user_id
        )
        .fetch_all(pool)
        .await?;
        Ok(emotes)
    }

    pub async fn update(&self, pool: &PgPool) -> SqlResult<()> {
        // language=PostgreSQL
        sqlx::query!(
            "UPDATE slots SET emote_id = $2, expires = $3, name = $4, added_by = $5, added_at = $6 WHERE id = $1",
            self.id,
            self.emote_id,
            self.expires,
            self.name,
            self.added_by,
            self.added_at,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn clear(id: i32, pool: &PgPool) -> SqlResult<()> {
        // language=PostgreSQL
        sqlx::query!(
            "UPDATE slots SET emote_id = null, expires = null, name = null, added_by = null, added_at = null WHERE id = $1",
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn remove(id: i32, pool: &PgPool) -> SqlResult<()> {
        // language=PostgreSQL
        sqlx::query!("DELETE FROM slots WHERE id = $1", id)
            .execute(pool)
            .await?;

        Ok(())
    }
}
