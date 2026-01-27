use crate::{services::emotes::refresh::refresh_emotes, PgPool, RedisConn};
use anyhow::Result as AnyResult;
use twitch_irc::message::PrivmsgMessage;

pub async fn execute_reload(
    msg: &PrivmsgMessage,
    redis: &mut RedisConn,
    pg: &PgPool,
) -> AnyResult<String> {
    let removed = refresh_emotes(&msg.channel_id, redis, pg).await?;

    Ok(format!(
        "@{}, removed {} emotes!",
        msg.sender.login, removed
    ))
}
