use chrono::{DateTime, Utc};
use errors::{
    json_error::JsonError,
    sql::{SqlReason, SqlResult},
};
use http::StatusCode;
use sqlx::PgPool;

#[derive(Debug, sqlx::FromRow)]
pub struct TimedMode {
    pub id: i32,
    pub user_name: String,
    pub user_id: String,
    pub mode: Mode,
    pub end_ts: DateTime<Utc>,
}

#[derive(sqlx::Type, Debug, derive_more::Display, Copy, Clone)]
#[sqlx(type_name = "timed_mode", rename_all = "snake_case")]
pub enum Mode {
    // the display values are for IRC
    #[display(fmt = "emoteonly")]
    Emoteonly,
    #[display(fmt = "subscribers")]
    Subonly,
}

impl TimedMode {
    pub async fn create_mode(
        user_id: &str,
        mode: Mode,
        duration: std::time::Duration,
        pool: &PgPool,
    ) -> SqlResult<i32> {
        let end = Utc::now()
            + (chrono::Duration::from_std(duration).map_err(|_| {
                JsonError::new(SqlReason::Internal, StatusCode::INTERNAL_SERVER_ERROR)
            })?);

        // language=PostgreSQL
        let id = sqlx::query_scalar!(
            "INSERT INTO timed_modes (user_id, mode, end_ts) VALUES ($1, $2, $3) RETURNING id",
            user_id,
            mode as _,
            end
        )
        .fetch_one(pool)
        .await?;

        Ok(id)
    }

    pub async fn get_all(pool: &PgPool) -> SqlResult<Vec<Self>> {
        // language=PostgreSQL
        let modes = sqlx::query_as!(
            Self,
            r#"
            SELECT timed_modes.id as "id!", user_id as "user_id!", end_ts as "end_ts!", mode as "mode!: Mode", name as "user_name!"
            FROM timed_modes
                LEFT JOIN users u on u.id = timed_modes.user_id
        "#
        )
        .fetch_all(pool)
        .await?;

        Ok(modes)
    }

    pub async fn delete_mode(id: i32, pool: &PgPool) -> SqlResult<()> {
        // language=PostgreSQL
        sqlx::query!("DELETE FROM timed_modes WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
