use crate::actors::irc_actor::IrcActor;
use crate::actors::messages::irc_messages::{
    SayMessage, TimedMode, TimedModeMessage, TimeoutMessage,
};
use crate::models::reward::{Reward, RewardData};
use crate::models::user::User;
use crate::services::bttv::{self, fetch_save_bttv_id, get_user_limits};
use crate::services::ffz::{self, is_editor_in};
use crate::services::twitch::requests::get_user;
use actix::Addr;
use anyhow::{Error as AnyError, Result as AnyResult};
use lazy_static::lazy_static;
use regex::Regex;
use sqlx::PgPool;
use std::sync::Arc;
use twitch_api2::eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1;
use twitch_api2::eventsub::NotificationPayload;
use twitch_api2::twitch_oauth2::UserToken;

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
            let emote_id = extract_id(
                extract_bttv_id,
                &redemption.event.user_input,
                &irc,
                redemption.event.broadcaster_user_login.clone(),
                redemption.event.user_login.clone(),
            )
            .await?;
            log::info!("Adding BTTV emote {} in {}", emote_id, broadcaster.name);
            let data = bttv::swap_or_add_emote(&broadcaster.id, emote_id, pool).await;
            send_emote_reply(
                data,
                &irc,
                redemption.event.broadcaster_user_login,
                redemption.event.user_login,
            )
            .await?;
        }
        RewardData::FfzSwap(_) => {
            let emote_id = extract_id(
                extract_ffz_id,
                &redemption.event.user_input,
                &irc,
                redemption.event.broadcaster_user_login.clone(),
                redemption.event.user_login.clone(),
            )
            .await?;
            log::info!("Adding FFZ emote {} in {}", emote_id, broadcaster.name);
            let data = ffz::swap_or_add_emote(&broadcaster.id, emote_id, pool).await;
            send_emote_reply(
                data,
                &irc,
                redemption.event.broadcaster_user_login,
                redemption.event.user_login,
            )
            .await?;
        }
    }
    Ok(())
}

async fn extract_id<'a, F>(
    extractor: F,
    input: &'a str,
    irc: &Arc<Addr<IrcActor>>,
    broadcaster: String,
    user: String,
) -> AnyResult<&'a str>
where
    F: FnOnce(&'a str) -> AnyResult<&'a str>,
{
    match extractor(input) {
        Ok(id) => Ok(id),
        Err(e) => {
            irc.send(SayMessage(broadcaster, format!("@{} ⚠ {}", user, e)))
                .await??;

            Err(e)
        }
    }
}

async fn send_emote_reply(
    data: AnyResult<(Option<String>, String)>,
    irc: &Arc<Addr<IrcActor>>,
    broadcaster: String,
    user: String,
) -> AnyResult<()> {
    match data {
        Ok((Some(removed), added)) => {
            irc.send(SayMessage(
                broadcaster,
                format!("@{} ☑ Added {} - 🗑 Removed {}", user, added, removed),
            ))
            .await??;
        }
        Ok((None, added)) => {
            irc.send(SayMessage(
                broadcaster,
                format!("@{} ☑ Added {}", user, added),
            ))
            .await??;
        }
        Err(e) => {
            irc.send(SayMessage(broadcaster, format!("@{} ⚠ {}", user, e)))
                .await??;

            return Err(e);
        }
    };
    Ok(())
}

fn extract_username(str: &str) -> AnyResult<String> {
    lazy_static! {
        static ref USERNAME_REGEX: Regex = Regex::new("@([\\w_]+)").expect("must compile");
    }

    let str = str.trim();

    if !str.contains(' ') {
        return Ok(str.replace("@", ""));
    }

    USERNAME_REGEX
        .captures(str)
        .map(|m| m.get(0))
        .flatten()
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| AnyError::msg("No user submitted"))
}

fn extract_bttv_id(str: &str) -> AnyResult<&str> {
    lazy_static! {
        static ref BTTV_REGEX: Regex = Regex::new(
            "(?:^| )(?:https?://)?(?:betterttv\\.com/)?(?:emotes/)?([a-f0-9]{24})(?:$| )"
        )
        .expect("must compile");
    }
    BTTV_REGEX
        .captures(str)
        .map(|c| c.iter().nth(1).flatten().map(|m| m.as_str()))
        .flatten()
        .ok_or_else(|| AnyError::msg("Could not find an emote code there!"))
}

fn extract_ffz_id(str: &str) -> AnyResult<&str> {
    lazy_static! {
        static ref FFZ_REGEX: Regex = Regex::new(
            "(?:^| )(?:https?://)?(?:www\\.)?(?:frankerfacez\\.com/)?(?:emoticon/)(\\d+)(?:-[\\w_!]+)?(?:$| )"
        )
        .expect("must compile");
    }
    FFZ_REGEX
        .captures(str)
        .map(|c| c.iter().nth(1).flatten().map(|m| m.as_str()))
        .flatten()
        .ok_or_else(|| AnyError::msg("Could not find an emote there!"))
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
            get_duration(duration)?;
        }

        // verify editor
        RewardData::BttvSwap(_) => {
            let this_user = User::get_bttv_data(broadcaster_id, pool).await?;
            let bttv_id = if let Some(id) = &this_user.bttv_id {
                id.clone()
            } else {
                fetch_save_bttv_id(broadcaster_id, pool)
                    .await
                    .map_err(|_| AnyError::msg("The user hasn't registered on bttv yet"))?
            };
            get_user_limits(&bttv_id)
                .await
                .map_err(|_| AnyError::msg("RewardMore isn't an editor for the user"))?;
        }
        RewardData::FfzSwap(_) => {
            let user = get_user(broadcaster_id.to_string(), token).await?;
            if !is_editor_in(&user.login).await {
                return Err(AnyError::msg("RewardMore isn't an editor for the user"));
            }
        }
    };
    Ok(())
}
