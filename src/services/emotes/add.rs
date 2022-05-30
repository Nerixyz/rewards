use crate::PgPool;
use anyhow::{anyhow, Result as AnyResult};
use config::CONFIG;
use futures_util::future;
use models::{emote::SlotPlatform, reward::Reward, swap_emote::SwapEmote};

pub async fn track_emote(
    channel_id: &str,
    executing_user_login: &str,
    emote_id: &str,
    emote_name: &str,
    slot_platform: SlotPlatform,
    pool: &PgPool,
) -> AnyResult<()> {
    let (limit, count) = future::try_join(
        Reward::get_swap_limit_for_user(channel_id, slot_platform, pool),
        SwapEmote::emote_count(channel_id, slot_platform, pool),
    )
    .await?;
    if limit.map(|lim| count >= lim as i64).unwrap_or(true) {
        return Err(anyhow!(
            "No swap capacity (limit is {:?}), try {}emote reload",
            limit,
            CONFIG.bot.prefix
        ));
    }
    SwapEmote::add(
        channel_id,
        emote_id,
        slot_platform,
        emote_name,
        executing_user_login,
        pool,
    )
    .await?;

    Ok(())
}
