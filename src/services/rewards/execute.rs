use std::sync::Arc;

use actix::Addr;
use anyhow::Result as AnyResult;
use sqlx::PgPool;
use twitch_api2::eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1;
use twitch_api2::eventsub::NotificationPayload;

use crate::actors::irc_actor::IrcActor;
use crate::actors::messages::irc_messages::{
    SayMessage, TimedMode, TimedModeMessage, TimeoutMessage,
};
use crate::models::reward::{Reward, RewardData};
use crate::models::user::User;
use crate::services::bttv::{slots, swap};
use crate::services::{ffz, rewards};

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
                user: rewards::extract_username(&redemption.event.user_input)?,
                duration: rewards::get_duration(&timeout)?,
                broadcaster: broadcaster.name,
            })
            .await??
        }
        RewardData::EmoteOnly(duration) => {
            irc.send(TimedModeMessage {
                duration: rewards::get_duration(&duration)?,
                broadcaster: broadcaster.name,
                mode: TimedMode::Emote,
            })
            .await?
        }
        RewardData::SubOnly(duration) => {
            irc.send(TimedModeMessage {
                duration: rewards::get_duration(&duration)?,
                broadcaster: broadcaster.name,
                mode: TimedMode::Sub,
            })
            .await?
        }
        RewardData::BttvSwap(_) => {
            let emote_id = rewards::extract_id(
                rewards::extract_bttv_id,
                &redemption.event.user_input,
                &irc,
                redemption.event.broadcaster_user_login.clone(),
                redemption.event.user_login.clone(),
            )
            .await?;
            log::info!("Adding BTTV emote {} in {}", emote_id, broadcaster.name);
            let data = swap::swap_or_add_emote(&broadcaster.id, emote_id, pool).await;
            send_emote_reply(
                data,
                &irc,
                redemption.event.broadcaster_user_login,
                redemption.event.user_login,
            )
            .await?;
        }
        RewardData::FfzSwap(_) => {
            let emote_id = rewards::extract_id(
                rewards::extract_ffz_id,
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
        RewardData::BttvSlot(slot) => {
            let emote_id = rewards::extract_id(
                rewards::extract_bttv_id,
                &redemption.event.user_input,
                &irc,
                redemption.event.broadcaster_user_login.clone(),
                redemption.event.user_login.clone(),
            )
            .await?;
            log::info!("Adding BTTV emote {} in {}", emote_id, broadcaster.name);
            let data = slots::add_emote(
                &broadcaster.id,
                &redemption.event.reward.id,
                slot,
                emote_id,
                pool,
            )
            .await;
            send_slot_reply(
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

async fn send_slot_reply(
    data: AnyResult<(String, usize)>,
    irc: &Arc<Addr<IrcActor>>,
    broadcaster: String,
    user: String,
) -> AnyResult<()> {
    match data {
        Ok((added, remaining)) if remaining > 1 => {
            irc.send(SayMessage(
                broadcaster,
                format!("@{} ☑ Added {} - 🔳 {} slots open", user, added, remaining),
            ))
            .await??;
        }
        Ok((added, remaining)) if remaining == 1 => {
            irc.send(SayMessage(
                broadcaster,
                format!("@{} ☑ Added {} - 🔳 {} slot open", user, added, remaining),
            ))
            .await??;
        }
        Ok((added, _)) => {
            irc.send(SayMessage(
                broadcaster,
                format!("@{} ☑ Added {} - 0 slots open - 🔒 closing", user, added),
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
