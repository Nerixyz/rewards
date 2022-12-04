use crate::{
    chat::commands::emote_management::extract::extract_emote_by_url,
    services::emotes::{add::track_emote, search::EmoteCache},
    PgPool, RedisConn,
};
use anyhow::{anyhow, Result as AnyResult};
use models::emote::SlotPlatform;
use twitch_irc::message::PrivmsgMessage;

struct FoundEmote {
    id: String,
    name: String,
    platform: SlotPlatform,
}

pub async fn execute_inject(
    msg: &PrivmsgMessage,
    emote: &str,
    redis: &mut RedisConn,
    pool: &PgPool,
) -> AnyResult<String> {
    let emote = find_emote(emote, &msg.channel_id, redis, pool)
        .await
        .ok_or_else(|| {
            anyhow!("Could not find emote. Try to specify the emote url!")
        })?;
    track_emote(
        &msg.channel_id,
        &msg.sender.login,
        &emote.id,
        &emote.name,
        emote.platform,
        pool,
    )
    .await?;
    Ok(format!(
        "@{}, injected {} from {}.",
        msg.sender.login, emote.name, emote.platform
    ))
}

async fn find_emote(
    emote: &str,
    channel_id: &str,
    redis: &mut RedisConn,
    pg: &PgPool,
) -> Option<FoundEmote> {
    let cache = EmoteCache::fetch_or_load(channel_id, redis, pg)
        .await
        .ok()?;

    if let Some((id, platform)) = extract_emote_by_url(emote) {
        return cache.find_name_by_id(&id, platform).map(|name| FoundEmote {
            id: id.into_owned(),
            name: name.to_owned(),
            platform,
        });
    }

    if let Some(emote) = cache.seventv.into_iter().find(|e| e.name == emote) {
        return Some(FoundEmote {
            id: emote.id,
            name: emote.name,
            platform: SlotPlatform::SevenTv,
        });
    }
    if let Some(emote) = cache.bttv.into_iter().find(|e| e.code == emote) {
        return Some(FoundEmote {
            id: emote.id,
            name: emote.code,
            platform: SlotPlatform::Bttv,
        });
    }
    if let Some(emote) = cache.ffz.into_iter().find(|e| e.name == emote) {
        return Some(FoundEmote {
            id: emote.id.to_string(),
            name: emote.name,
            platform: SlotPlatform::Ffz,
        });
    }

    None
}
