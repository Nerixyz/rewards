use crate::services::{
    bttv,
    emotes::{
        bttv::BttvEmotes, ffz::FfzEmotes, seven_tv::SevenTvEmotes, slots, swap,
    },
    ffz, seven_tv,
};
use anyhow::Result as AnyResult;
use models::reward::RewardData;
use sqlx::PgPool;

pub async fn save_reward(
    reward: &RewardData,
    reward_id: &str,
    broadcaster_id: &str,
    pool: &PgPool,
) -> AnyResult<()> {
    match reward {
        RewardData::BttvSwap(swap) => {
            if let Some(limit) = &swap.limit {
                swap::update_swap_limit::<BttvEmotes>(
                    broadcaster_id,
                    *limit,
                    pool,
                )
                .await?;
            }
        }
        RewardData::FfzSwap(swap) => {
            if let Some(limit) = &swap.limit {
                swap::update_swap_limit::<FfzEmotes>(
                    broadcaster_id,
                    *limit,
                    pool,
                )
                .await?;
            }
        }
        RewardData::SevenTvSwap(swap) => {
            if let Some(limit) = &swap.limit {
                swap::update_swap_limit::<SevenTvEmotes>(
                    broadcaster_id,
                    *limit,
                    pool,
                )
                .await?;
            }
        }
        RewardData::BttvSlot(slot) => {
            let bttv_id = bttv::get_or_fetch_id(broadcaster_id, pool).await?;

            slots::adjust_size::<BttvEmotes>(
                broadcaster_id,
                &bttv_id,
                reward_id,
                slot.slots,
                pool,
            )
            .await?;
        }
        RewardData::FfzSlot(slot) => {
            let ffz_id = ffz::requests::get_user(broadcaster_id).await?.id;
            slots::adjust_size::<FfzEmotes>(
                broadcaster_id,
                &ffz_id,
                reward_id,
                slot.slots,
                pool,
            )
            .await?;
        }
        RewardData::SevenTvSlot(slot) => {
            let sid = seven_tv::requests::get_user(broadcaster_id)
                .await?
                .emote_set
                .id;
            slots::adjust_size::<SevenTvEmotes>(
                broadcaster_id,
                &sid,
                reward_id,
                slot.slots,
                pool,
            )
            .await?;
        }
        _ => (),
    }
    Ok(())
}
