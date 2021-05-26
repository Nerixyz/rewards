use crate::actors::irc_actor::IrcActor;
use crate::actors::messages::irc_messages::{TimeoutMessage, TimedModeMessage, TimedMode};
use crate::models::reward::{Reward, RewardData};
use crate::models::user::User;
use actix::Addr;
use anyhow::Error as AnyError;
use regex::Regex;
use sqlx::PgPool;
use std::sync::Arc;
use twitch_api2::eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1;
use twitch_api2::eventsub::NotificationPayload;

/// This doesn't update the reward-redemption on twitch!
pub async fn execute_reward(
    redemption: NotificationPayload<ChannelPointsCustomRewardRedemptionAddV1>,
    reward: Reward,
    broadcaster: User,
    _pool: &PgPool,
    irc: Arc<Addr<IrcActor>>,
) -> Result<(), AnyError> {
    match reward.data.0 {
        RewardData::Timeout(timeout) =>
            irc.send(TimeoutMessage {
                user: extract_username(&redemption.event.user_input)?,
                duration: humantime::parse_duration(&timeout)?,
                broadcaster: broadcaster.name,
            })
            .await??,
        RewardData::EmoteOnly(duration) =>
            irc.send(TimedModeMessage {
                duration: humantime::parse_duration(&duration)?,
                broadcaster: broadcaster.name,
                mode: TimedMode::Emote
            })
            .await?,
        RewardData::SubOnly(duration) =>
            irc.send(TimedModeMessage {
                duration: humantime::parse_duration(&duration)?,
                broadcaster: broadcaster.name,
                mode: TimedMode::Sub
            })
            .await?
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
