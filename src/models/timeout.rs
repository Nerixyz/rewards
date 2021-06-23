use crate::services::sql::SqlResult;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

#[derive(FromRow)]
pub struct Timeout {
    pub channel_id: String,
    pub user_id: String,
    pub expires_at: DateTime<Utc>,
}

impl Timeout {
    pub async fn create(
        channel: &str,
        user: &str,
        expires_at: DateTime<Utc>,
        pool: &PgPool,
    ) -> SqlResult<()> {
        //language=PostgreSQL
        sqlx::query!(
            r#"INSERT INTO timeouts (channel_id, user_id, expires_at) VALUES ($1, $2, $3)"#,
            channel,
            user,
            expires_at as _
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn delete_inactive(pool: &PgPool) -> SqlResult<()> {
        //language=PostgreSQL
        sqlx::query!(r#"DELETE FROM timeouts WHERE expires_at < now()"#)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn delete_specific(channel_id: &str, user_id: &str, pool: &PgPool) -> SqlResult<()> {
        //language=PostgreSQL
        sqlx::query!(
            r#"DELETE FROM timeouts WHERE channel_id = $1 AND user_id = $2"#,
            channel_id,
            user_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get_timeout(
        channel_id: &str,
        user_id: &str,
        pool: &PgPool,
    ) -> SqlResult<Option<DateTime<Utc>>> {
        //language=PostgreSQL
        let dt = sqlx::query_scalar!(
            r#"
            SELECT expires_at
            FROM timeouts
            WHERE channel_id = $1 AND user_id = $2
            "#,
            channel_id,
            user_id,
        )
        .fetch_optional(pool)
        .await?;
        Ok(dt)
    }
}
