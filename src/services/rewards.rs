use twitch_api2::eventsub::NotificationPayload;
use twitch_api2::eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1;
use crate::models::reward::{Reward, RewardData};
use crate::models::user::User;
use sqlx::PgPool;
use actix::Addr;
use crate::actors::irc_actor::IrcActor;
use anyhow::Error as AnyError;
use regex::Regex;
use std::sync::Arc;
use crate::actors::messages::irc_messages::{TimeoutMessage, SubOnlyMessage, EmoteOnlyMessage};

/// This doesn't update the reward-redemption on twitch!
pub async fn execute_reward(
    redemption: NotificationPayload<ChannelPointsCustomRewardRedemptionAddV1>,
    reward: Reward,
    broadcaster: User,
    _pool: &PgPool,
    irc: Arc<Addr<IrcActor>>,
) -> Result<(), AnyError> {
    match reward.data.0 {
        RewardData::Timeout(timeout) => {
            let duration = humantime::parse_duration(&timeout)?;
            let user = extract_username(&redemption.event.user_input)?;

            irc.send(TimeoutMessage {
                user,
                duration,
                broadcaster: broadcaster.name
            }).await??;
        },
        RewardData::EmoteOnly(duration) => {
            let duration = humantime::parse_duration(&duration)?;
            irc.send(EmoteOnlyMessage {
                duration,
                broadcaster: broadcaster.name
            }).await?;
        },
        RewardData::SubOnly(duration) => {
            let duration = humantime::parse_duration(&duration)?;
            irc.send(SubOnlyMessage {
                duration,
                broadcaster: broadcaster.name
            }).await?;
        }
    }
    Ok(())
}

pub fn extract_username(str: &str) -> Result<String, AnyError> {
    if !str.contains(" ") {
        return Ok(str.replace("@", ""));
    }

    Regex::new("@([\\w_]+)")
        .expect("must compile")
        .captures(str)
        .map(|m| m.get(0))
        .flatten()
        .map(|m| m.as_str().to_string())
        .ok_or(AnyError::msg("No user submitted"))
}

pub fn verify_reward(reward: &RewardData) -> Result<(), AnyError> {
    Ok(match reward {
        RewardData::Timeout(duration) => humantime::parse_duration(duration).map(|_| ())?,
        RewardData::SubOnly(duration) => humantime::parse_duration(duration).map(|_| ())?,
        RewardData::EmoteOnly(duration) => humantime::parse_duration(duration).map(|_| ())?,
    })
}