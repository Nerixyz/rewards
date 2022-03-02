use super::extract::extract_emote_data;
use crate::{services::emotes::remove::untrack_emote, PgPool};
use anyhow::{anyhow, Result as AnyResult};
use twitch_irc::message::PrivmsgMessage;

pub async fn execute_eject(msg: &PrivmsgMessage, emote: &str, pool: &PgPool) -> AnyResult<String> {
    let (emote_id, platform) = extract_emote_data(emote, &msg.channel_id, pool)
        .await
        .ok_or_else(|| anyhow!("Could not find emote. Try to specify the emote url!"))?;
    let name = untrack_emote(&msg.channel_id, &emote_id, platform, pool).await?;
    Ok(format!(
        "@{}, ejected {} from {} - the emote remains in this channel but I won't manage the emote.",
        msg.sender.login, name, platform
    ))
}
