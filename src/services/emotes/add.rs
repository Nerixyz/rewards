use crate::PgPool;
use anyhow::{anyhow, Result as AnyResult};
use config::CONFIG;
use models::{
    emote::SlotPlatform,
    reward::{self},
    swap_emote::SwapEmote,
};

pub async fn track_emote(
    channel_id: &str,
    executing_user_login: &str,
    emote_id: &str,
    emote_name: &str,
    slot_platform: SlotPlatform,
    pool: &PgPool,
) -> AnyResult<()> {
    let Some(chosen_reward) = reward::Reward::get_swap_stats_for_user(
        channel_id,
        slot_platform,
        pool,
    )
    .await?
    .into_iter()
    .filter(|it| it.limit.is_none_or(|l| it.count < l as usize))
    .next() else {
        return Err(anyhow!(
            "No reward with enough capacity, try {}emote reload",
            CONFIG.bot.prefix
        ));
    };
    SwapEmote::add_or_update(
        channel_id,
        emote_id,
        slot_platform,
        emote_name,
        executing_user_login,
        &chosen_reward.reward_id,
        pool,
    )
    .await?;

    Ok(())
}
