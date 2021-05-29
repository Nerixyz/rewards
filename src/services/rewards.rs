use crate::actors::irc_actor::IrcActor;
use crate::actors::messages::irc_messages::{TimedMode, TimedModeMessage, TimeoutMessage, SayMessage};
use crate::models::reward::{Reward, RewardData};
use crate::models::user::User;
use actix::Addr;
use anyhow::{Error as AnyError, Result as AnyResult};
use regex::Regex;
use sqlx::PgPool;
use std::sync::Arc;
use twitch_api2::eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1;
use twitch_api2::eventsub::NotificationPayload;
use crate::services::bttv::{swap_or_add_emote, fetch_save_bttv_id, get_user_limits};
use crate::services::bttv::requests::get_emote;

/// This doesn't update the reward-redemption on twitch!
pub async fn execute_reward(
    redemption: NotificationPayload<ChannelPointsCustomRewardRedemptionAddV1>,
    reward: Reward,
    broadcaster: User,
    pool: &PgPool,
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
        RewardData::BttvSwap(_) => {
            let emote_id = match extract_bttv_id(&redemption.event.user_input) {
                Ok(id) => id,
                Err(e) => {
                    irc.send(SayMessage(
                        redemption.event.broadcaster_user_login,
                        format!("@{} ⚠ {}", redemption.event.user_login, e))
                    ).await??;

                    return Err(e);
                }
            };
            log::info!("Adding BTTV emote {} in {}", emote_id, broadcaster.name);
            match swap_or_add_emote(&broadcaster.id, emote_id, pool).await {
                Ok((Some(removed), added)) => {
                    let removed_name = get_emote(&removed).await.map(|e| e.code)
                        .unwrap_or_else(|e| {
                            log::warn!("Emote {} was added in {} but isn't there anymore error={}", removed, broadcaster.name, e);
                            "[?]".to_string()
                        });

                    irc.send(SayMessage(
                        redemption.event.broadcaster_user_login,
                        format!("@{} ☑ Added {} - 🗑 Removed {}", redemption.event.user_login, added, removed_name))
                    ).await??;
                },
                Ok((None, added)) => {
                    irc.send(SayMessage(
                        redemption.event.broadcaster_user_login,
                        format!("@{} ☑ Added {}", redemption.event.user_login, added))
                    ).await??;
                },
                Err(e) => {
                    irc.send(SayMessage(
                        redemption.event.broadcaster_user_login,
                        format!("@{} ⚠ {}", redemption.event.user_login, e))
                    ).await??;

                    return Err(e);
                }
            }
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

fn extract_bttv_id(str: &str) -> AnyResult<&str> {
    Regex::new("(?:^| )(?:https?://)?(?:betterttv\\.com/)?(?:emotes/)?([a-f0-9]{24})(?:$| )")
        .expect("must compile")
        .captures(str)
        .map(|c| {
            c
                .iter()
                .skip(1)
                .next()
                .flatten()
                .map(|m| m.as_str())
        }
        )
        .flatten()
        .ok_or_else(|| AnyError::msg("Could not find an emote code there!"))
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

pub async fn verify_reward(reward: &RewardData, broadcaster_id: &str, pool: &PgPool) -> AnyResult<()> {
    match reward {
        RewardData::EmoteOnly(duration) |
        RewardData::Timeout(duration) |
        RewardData::SubOnly(duration) => {
            get_duration(duration)?;
        },

        // verify editor
        RewardData::BttvSwap(_) => {
            let this_user = User::get_bttv_data(broadcaster_id, pool).await?;
            let bttv_id = if let Some(id) = &this_user.bttv_id {
                id.clone()
            } else {
                fetch_save_bttv_id(broadcaster_id, pool).await.map_err(|_| AnyError::msg("The user hasn't registered on bttv yet"))?
            };
            get_user_limits(&bttv_id).await.map_err(|_| AnyError::msg("RewardMore isn't an editor for the user"))?;
        },
    };
    Ok(())
}
