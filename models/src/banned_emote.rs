use crate::emote::SlotPlatform;
use errors::sql::SqlResult;
use sqlx::PgPool;

pub async fn is_banned(
    channel_id: &str,
    emote_id: &str,
    platform: SlotPlatform,
    pool: &PgPool,
) -> SqlResult<bool> {
    // language=PostgreSQL
    let result = sqlx::query_scalar!(
        "SELECT 1 FROM banned_emotes WHERE channel_id = $1 AND emote_id = $2 and platform = $3",
        channel_id,
        emote_id,
        platform as _
    )
    .fetch_optional(pool)
    .await?;
    Ok(result.is_some())
}

pub async fn add(
    channel_id: &str,
    emote_id: &str,
    platform: SlotPlatform,
    pool: &PgPool,
) -> SqlResult<()> {
    // language=PostgreSQL
    sqlx::query!(
        "INSERT INTO banned_emotes (channel_id, emote_id, platform) VALUES ($1, $2, $3)",
        channel_id,
        emote_id,
        platform as _
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove(
    channel_id: &str,
    emote_id: &str,
    platform: SlotPlatform,
    pool: &PgPool,
) -> SqlResult<()> {
    // language=PostgreSQL
    sqlx::query_scalar!(
        "DELETE FROM banned_emotes WHERE channel_id = $1 AND emote_id = $2 and platform = $3",
        channel_id,
        emote_id,
        platform as _
    )
    .execute(pool)
    .await?;
    Ok(())
}
