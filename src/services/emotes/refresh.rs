use anyhow::Result as AnyResult;
use futures::future;
use models::{slot::Slot, swap_emote::SwapEmote};
use sqlx::PgPool;

use crate::{
    services::emotes::{remove, search::EmoteCache},
    RedisConn,
};

pub async fn refresh_emotes(
    channel_id: &str,
    redis: &mut RedisConn,
    pg: &PgPool,
) -> AnyResult<usize> {
    let (cache, swaps, slots) = future::join3(
        EmoteCache::fetch_or_load(channel_id, redis, pg),
        SwapEmote::all_for_user(channel_id, pg),
        Slot::get_occupied(channel_id, pg),
    )
    .await;
    let (cache, swaps, slots) = (cache?, swaps?, slots?);

    let mut removed = 0;
    for swap in swaps {
        if cache.non_empty_platform(swap.platform)
            && !cache.contains(&swap.emote_id, swap.platform)
        {
            SwapEmote::remove(swap.id, pg).await?;
            removed += 1;
        }
    }
    for slot in slots {
        if let Some(id) = &slot.emote_id {
            if cache.non_empty_platform(slot.platform)
                && !cache.contains(id, slot.platform)
            {
                let (db, _twitch) = future::join(
                    Slot::clear(slot.id, pg),
                    remove::enable_reward(&slot, pg),
                )
                .await;
                db?;
                removed += 1;
            }
        }
    }

    Ok(removed)
}
