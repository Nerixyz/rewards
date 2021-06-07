use crate::services::sql::SqlResult;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool};

#[derive(FromRow, Serialize)]
pub struct LogEntry {
    pub date: DateTime<Utc>,
    pub content: String,
}

impl LogEntry {
    pub async fn get_for_user(id: &str, pool: &PgPool) -> SqlResult<Vec<Self>> {
        // language=PostgreSQL
        let logs = sqlx::query_as!(
            Self,
            r#"SELECT date, content FROM logs WHERE user_id = $1 ORDER BY date desc"#,
            id
        )
        .fetch_all(pool)
        .await?;

        Ok(logs)
    }

    pub async fn create(id: &str, content: &str, pool: &PgPool) -> SqlResult<()> {
        // language=PostgreSQL
        sqlx::query!(
            r#"INSERT INTO logs (user_id, date, content) VALUES ($1, $2, $3)"#,
            id,
            Utc::now(),
            content
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}
