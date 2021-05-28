use crate::actors::irc_actor::IrcActor;
use crate::actors::messages::irc_messages::{TimedMode, TimedModeMessage, TimeoutMessage};
use crate::models::reward::{Reward, RewardData};
use crate::models::user::User;
use actix::Addr;
use anyhow::{Error as AnyError, Result as AnyResult};
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
) -> AnyResult<()> {
    match reward.data.0 {
        RewardData::Timeout(timeout) => {
            irc.send(TimeoutMessage {
                user: extract_username(&redemption.event.user_input)?,
                duration: get_duration(&timeout)?,
                broadcaster: broadcaster.name,
            })
            .await??
        }
        RewardData::EmoteOnly(duration) => {
            irc.send(TimedModeMessage {
                duration: get_duration(&duration)?,
                broadcaster: broadcaster.name,
                mode: TimedMode::Emote,
            })
            .await?
        }
        RewardData::SubOnly(duration) => {
            irc.send(TimedModeMessage {
                duration: get_duration(&duration)?,
                broadcaster: broadcaster.name,
                mode: TimedMode::Sub,
            })
            .await?
        }
    }
    Ok(())
}

fn extract_username(str: &str) -> AnyResult<String> {
    let str = str.trim();

    if !str.contains(' ') {
        return Ok(str.replace("@", ""));
    }

    Regex::new("@([\\w_]+)")
        .expect("must compile")
        .captures(str)
        .map(|m| m.get(0))
        .flatten()
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| AnyError::msg("No user submitted"))
}

fn get_duration(duration: &str) -> AnyResult<u64> {
    let duration = duration.trim();

    if let Some(captures) = Regex::new("^rand\\(([^;]+);([^)]+)\\)$")
        .expect("must compile")
        .captures(duration)
    {
        let mut iter = captures
            .iter()
            .skip(1)
            .take(2)
            .flatten()
            .map(|m| humantime::parse_duration(m.as_str().trim()).map(|d| d.as_secs()));
        let (first, second) = (iter.next(), iter.next());

        let (first, second) = match (first, second) {
            (Some(Ok(first)), Some(Ok(second))) => (first, second),
            tuple => {
                return Err(AnyError::msg(format!(
                    "Could not parse duration: {:?}",
                    tuple
                )))
            }
        };

        let (start, diff) = if first < second {
            (first, second - first)
        } else {
            (second, first - second)
        };

        Ok((start as f64 + rand::random::<f64>() * (diff as f64)).floor() as u64)
    } else {
        Ok(humantime::parse_duration(duration)?.as_secs())
    }
}

pub fn verify_reward(reward: &RewardData) -> AnyResult<()> {
    match reward {
        RewardData::Timeout(duration) => get_duration(duration)?,
        RewardData::SubOnly(duration) => get_duration(duration)?,
        RewardData::EmoteOnly(duration) => get_duration(duration)?,
    };
    Ok(())
}
