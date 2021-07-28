use crate::models::emote::SlotPlatform;
use chrono::{DateTime, Utc};
use errors::sql::SqlResult;
use sqlx::{FromRow, PgPool};

#[derive(FromRow, Debug)]
pub struct SwapEmote {
    pub id: i64,
    pub user_id: String,
    pub emote_id: String,
    pub platform: SlotPlatform,
    pub name: String,
    pub added_by: String,
    pub added_at: DateTime<Utc>,
}

impl SwapEmote {
    pub async fn oldest(
        user_id: &str,
        platform: SlotPlatform,
        pool: &PgPool,
    ) -> SqlResult<Option<Self>> {
        // language=PostgreSQL
        let emote = sqlx::query_as!(
            Self,
            r#"
            SELECT id, user_id, emote_id, platform as "platform: _", name, added_by, added_at
            FROM swap_emotes
            WHERE user_id = $1 AND platform = $2
            ORDER BY added_at
            LIMIT 1"#,
            user_id,
            platform as _
        )
        .fetch_optional(pool)
        .await?;
        Ok(emote)
    }

    pub async fn add(
        user_id: &str,
        emote_id: &str,
        platform: SlotPlatform,
        name: &str,
        added_by: &str,
        pool: &PgPool,
    ) -> SqlResult<()> {
        // language=PostgreSQL
        sqlx::query!("INSERT INTO swap_emotes (user_id, emote_id, platform, name, added_by, added_at) VALUES ($1, $2, $3, $4, $5, now())", user_id, emote_id, platform as _, name, added_by).execute(pool).await?;
        Ok(())
    }

    pub async fn remove(id: i64, pool: &PgPool) -> SqlResult<()> {
        // language=PostgreSQL
        sqlx::query!("DELETE FROM swap_emotes WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn emote_count(
        user_id: &str,
        platform: SlotPlatform,
        pool: &PgPool,
    ) -> SqlResult<i64> {
        // language=PostgreSQL
        let count = sqlx::query_scalar!(
            "
            SELECT count(*)
            FROM swap_emotes
            WHERE user_id = $1 AND platform = $2",
            user_id,
            platform as _
        )
        .fetch_one(pool)
        .await?;
        Ok(count.unwrap_or_default())
    }

    pub async fn by_name(user_id: &str, name: &str, pool: &PgPool) -> SqlResult<Option<Self>> {
        // language=PostgreSQL
        let emote = sqlx::query_as!(
            Self,
            r#"
            SELECT id, user_id, emote_id, platform as "platform: _", name, added_by, added_at
            FROM swap_emotes
            WHERE user_id = $1 AND lower(name) = lower($2)"#,
            user_id,
            name
        )
        .fetch_optional(pool)
        .await?;
        Ok(emote)
    }
}
