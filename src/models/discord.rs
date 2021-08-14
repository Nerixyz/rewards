use errors::sql::SqlResult;
use sqlx::PgPool;

pub async fn get_discord_webhook_url(user_id: &str, pool: &PgPool) -> SqlResult<Option<String>> {
    // language=PostgreSQL
    let url = sqlx::query_scalar!(
        "SELECT url FROM discord_settings WHERE user_id = $1 AND log_emotes = true",
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(url)
}
