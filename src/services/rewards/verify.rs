use anyhow::{Error as AnyError, Result as AnyResult};
use sqlx::PgPool;
use twitch_api::twitch_oauth2::UserToken;

use crate::services::{
    bttv, ffz::is_editor_in, rewards::extract, seven_tv,
    spotify::rewards as spotify, twitch::requests::get_user,
};
use models::reward::RewardData;

pub async fn verify_reward(
    reward: &RewardData,
    broadcaster_id: &str,
    pool: &PgPool,
    token: &UserToken,
) -> AnyResult<()> {
    match reward {
        RewardData::EmoteOnly(duration) | RewardData::SubOnly(duration) => {
            extract::duration(duration)?;
        }
        RewardData::Timeout(data) => {
            extract::duration(&data.duration)?;
        }

        // verify editor
        RewardData::BttvSwap(_) => {
            bttv::verify_user(broadcaster_id, pool).await?;
        }
        RewardData::FfzSwap(_) => {
            let user = get_user(broadcaster_id.to_string(), token).await?;
            if !is_editor_in(user.login.as_ref()).await {
                return Err(AnyError::msg(
                    "RewardMore isn't an editor for the user",
                ));
            }
        }
        RewardData::SevenTvSwap(_) => {
            seven_tv::verify_user(broadcaster_id).await?;
        }
        RewardData::BttvSlot(slot) => {
            bttv::verify_user(broadcaster_id, pool).await?;

            if slot.slots > 50 {
                return Err(AnyError::msg("50 slots is the max"));
            }

            extract::duration(&slot.expiration)?;
        }
        RewardData::FfzSlot(slot) => {
            let user = get_user(broadcaster_id.to_string(), token).await?;
            if !is_editor_in(user.login.as_ref()).await {
                return Err(AnyError::msg(
                    "RewardMore isn't an editor for the user",
                ));
            }

            if slot.slots > 50 {
                return Err(AnyError::msg("50 slots is the max"));
            }

            extract::duration(&slot.expiration)?;
        }
        RewardData::SevenTvSlot(slot) => {
            seven_tv::verify_user(broadcaster_id).await?;

            if slot.slots > 100 {
                return Err(AnyError::msg("100 slots is the max"));
            }

            extract::duration(&slot.expiration)?;
        }
        RewardData::SpotifySkip(_)
        | RewardData::SpotifyQueue(_)
        | RewardData::SpotifyPlay(_) => {
            spotify::get_spotify_token(broadcaster_id, pool).await?;
        }
        RewardData::RemEmote(d) => match d.platform {
            models::emote::SlotPlatform::Bttv => {
                bttv::verify_user(broadcaster_id, pool).await?;
            }
            models::emote::SlotPlatform::Ffz => {
                let user = get_user(broadcaster_id.to_string(), token).await?;
                if !is_editor_in(user.login.as_ref()).await {
                    return Err(AnyError::msg(
                        "RewardMore isn't an editor for the user",
                    ));
                }
            }
            models::emote::SlotPlatform::SevenTv => {
                seven_tv::verify_user(broadcaster_id).await?;
            }
        },
    };
    Ok(())
}

pub fn verify_live_delay(delay: &Option<String>) -> AnyResult<()> {
    if let Some(delay) = delay {
        humantime::parse_duration(delay).map_err(|e| {
            AnyError::msg(format!("Could not parse duration: {}", e))
        })?;
    }
    Ok(())
}
