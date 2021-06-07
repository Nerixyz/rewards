use crate::models::reward::RewardData;
use crate::services::bttv;
use crate::services::emotes::bttv::BttvEmotes;
use crate::services::emotes::ffz::FfzEmotes;
use crate::services::emotes::slots;
use crate::services::ffz;
use anyhow::Result as AnyResult;
use sqlx::PgPool;

pub async fn save_reward(
    reward: &RewardData,
    reward_id: &str,
    broadcaster_id: &str,
    pool: &PgPool,
) -> AnyResult<()> {
    match reward {
        RewardData::BttvSlot(slot) => {
            let bttv_id = bttv::get_or_fetch_id(broadcaster_id, pool).await?;

            slots::adjust_size::<BttvEmotes, _, _, _>(
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
            slots::adjust_size::<FfzEmotes, _, _, _>(
                broadcaster_id,
                &ffz_id,
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
