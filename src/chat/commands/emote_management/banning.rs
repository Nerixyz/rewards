use super::extract::extract_emote_data;
use crate::{
    services::emotes::{format::format_emote_url, remove::remove_emote},
    PgPool, RedisPool,
};
use anyhow::{anyhow, Result as AnyResult};
use models::banned_emote;
use twitch_irc::message::PrivmsgMessage;

pub async fn execute_ban(
    msg: &PrivmsgMessage,
    emote: &str,
    pool: &PgPool,
    redis_pool: &RedisPool,
) -> AnyResult<String> {
    let (emote_id, platform) = extract_emote_data(emote, &msg.channel_id, pool)
        .await
        .ok_or_else(|| {
            anyhow!("Could not find emote. Try to specify the emote url!")
        })?;
    banned_emote::add(&msg.channel_id, &emote_id, platform, pool)
        .await
        .map_err(|_| {
            anyhow!("Couldn't add ban, the emote might be banned already")
        })?;
    // .ok because it may not be added
    remove_emote(&msg.channel_id, &emote_id, platform, pool, redis_pool)
        .await
        .ok();
    Ok(format!(
        "@{}, 🚫 Banned {}",
        msg.sender.login,
        format_emote_url(platform, &emote_id)
    ))
}

pub async fn execute_unban(
    msg: &PrivmsgMessage,
    emote: &str,
    pool: &PgPool,
) -> AnyResult<String> {
    let (emote_id, platform) = extract_emote_data(emote, &msg.channel_id, pool)
        .await
        .ok_or_else(|| {
            anyhow!("Could not find emote. Try to specify the emote url!")
        })?;
    banned_emote::remove(&msg.channel_id, &emote_id, platform, pool).await?;
    Ok(format!(
        "@{}, ✅ Unbanned {}",
        msg.sender.login,
        format_emote_url(platform, &emote_id)
    ))
}
