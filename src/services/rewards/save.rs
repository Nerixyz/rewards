use crate::models::reward::RewardData;
use crate::services::bttv;
use anyhow::Result as AnyResult;
use sqlx::PgPool;

pub async fn save_reward(
    reward: &RewardData,
    reward_id: &str,
    broadcaster_id: &str,
    pool: &PgPool,
) -> AnyResult<()> {
    if let RewardData::BttvSlot(slot) = reward {
        let bttv_id = bttv::get_or_fetch_id(broadcaster_id, pool).await?;

        bttv::slots::adjust_size(broadcaster_id, &bttv_id, reward_id, slot.slots, pool).await?;
    };
    Ok(())
}
