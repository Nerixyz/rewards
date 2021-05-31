use anyhow::{Error as AnyError, Result as AnyResult};
use sqlx::PgPool;
use twitch_api2::twitch_oauth2::UserToken;

use crate::models::reward::RewardData;
use crate::services::ffz::is_editor_in;
use crate::services::twitch::requests::get_user;
use crate::services::{bttv, rewards};

pub async fn verify_reward(
    reward: &RewardData,
    broadcaster_id: &str,
    pool: &PgPool,
    token: &UserToken,
) -> AnyResult<()> {
    match reward {
        RewardData::EmoteOnly(duration)
        | RewardData::Timeout(duration)
        | RewardData::SubOnly(duration) => {
            rewards::get_duration(duration)?;
        }

        // verify editor
        RewardData::BttvSwap(_) => {
            bttv::verify_user(broadcaster_id, pool).await?;
        }
        RewardData::FfzSwap(_) => {
            let user = get_user(broadcaster_id.to_string(), token).await?;
            if !is_editor_in(&user.login).await {
                return Err(AnyError::msg("RewardMore isn't an editor for the user"));
            }
        }
        RewardData::BttvSlot(slot) => {
            bttv::verify_user(broadcaster_id, pool).await?;

            if slot.slots > 50 {
                return Err(AnyError::msg("50 slots is the max"));
            }

            rewards::get_duration(&slot.expiration)?;
        }
    };
    Ok(())
}
