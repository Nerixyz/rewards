use errors::sql::SqlResult;
use serde::Serialize;
use sqlx::{FromRow, PgPool};

#[derive(Serialize, FromRow)]
pub struct DiscordUserSettings {
    pub user_id: String,
    pub url: String,
    pub log_emotes: bool,
}

pub async fn get_discord_settings(
    user_id: &str,
    pool: &PgPool,
) -> SqlResult<Option<DiscordUserSettings>> {
    // language=PostgreSQL
    let settings = sqlx::query_as!(
        DiscordUserSettings,
        "SELECT * FROM discord_settings WHERE user_id = $1",
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(settings)
}

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

pub async fn set_discord_webhook_url(user_id: &str, url: &str, pool: &PgPool) -> SqlResult<()> {
    // language=PostgreSQL
    sqlx::query_scalar!(
        "INSERT INTO discord_settings (user_id, url) VALUES ($1, $2) ON CONFLICT (user_id) DO UPDATE SET url = excluded.url",
        user_id, url
    )
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_discord_webhook_url(user_id: &str, pool: &PgPool) -> SqlResult<()> {
    // language=PostgreSQL
    sqlx::query_scalar!("DELETE FROM discord_settings WHERE user_id = $1", user_id)
        .execute(pool)
        .await?;

    Ok(())
}
