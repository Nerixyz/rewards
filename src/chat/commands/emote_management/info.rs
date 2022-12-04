use crate::{
    services::{
        emotes::search::search_emote_by_name, formatting::human_format_duration,
    },
    PgPool, RedisConn,
};
use anyhow::{Error as AnyError, Result as AnyResult};
use chrono::Utc;
use deadpool_redis::redis::AsyncCommands;
use either::Either;
use models::{slot::Slot, swap_emote::SwapEmote};
use twitch_irc::message::PrivmsgMessage;

pub async fn execute_info(
    msg: &PrivmsgMessage,
    emote: &str,
    pool: &PgPool,
    redis: &mut RedisConn,
) -> AnyResult<String> {
    match search_emote_by_name(emote, &msg.channel_id, pool).await? {
        Some(Either::Left(slot)) => format_slot(&msg.sender.login, slot),
        Some(Either::Right(swap)) => format_swap(&msg.sender.login, swap),
        None => {
            if let Some(slot) = redis
                .get::<_, String>(format!(
                    "rewards:exp-slots:{}:{}",
                    msg.channel_id,
                    emote.to_lowercase()
                ))
                .await
                .ok()
                .and_then(|s| serde_json::from_str::<Slot>(&s).ok())
            {
                format_slot(&msg.sender.login, slot)
            } else {
                Err(AnyError::msg("This emote is unknown to me."))
            }
        }
    }
}

fn format_slot(sender: &str, slot: Slot) -> AnyResult<String> {
    match (slot.name, slot.added_at, slot.added_by, slot.expires) {
        (Some(name), Some(added_at), Some(added_by), Some(expired)) => {
            let now = Utc::now();
            let added_duration = now - added_at;
            let expired_duration = now - expired;
            let in_past = now > expired;
            Ok(format!(
                "@{}, {} was added {} by @{} and {} removed {}",
                sender,
                name,
                human_format_duration(&added_duration),
                added_by,
                if in_past { "was" } else { "will be" },
                human_format_duration(&expired_duration)
            ))
        }
        _ => Err(AnyError::msg("Not enough information")),
    }
}

fn format_swap(sender: &str, emote: SwapEmote) -> AnyResult<String> {
    let now = Utc::now();
    let added_duration = now - emote.added_at;
    Ok(format!(
        "@{}, {} was added {} by @{} [{}]",
        sender,
        emote.name,
        human_format_duration(&added_duration),
        emote.added_by,
        emote.platform
    ))
}
