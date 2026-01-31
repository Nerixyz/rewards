use crate::services::spotify::rewards::skip_track;
use anyhow::Result as AnyResult;
use sqlx::PgPool;
use twitch_irc::message::PrivmsgMessage;

pub async fn execute(msg: PrivmsgMessage, pool: &PgPool) -> AnyResult<String> {
    let name = skip_track(&msg.channel_id, true, pool).await?;
    Ok(format!("Skipped {name}"))
}
