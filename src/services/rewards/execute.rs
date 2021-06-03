use std::sync::Arc;

use actix::Addr;
use anyhow::Result as AnyResult;
use sqlx::PgPool;
use twitch_api2::eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1;
use twitch_api2::eventsub::NotificationPayload;

use crate::actors::irc_actor::IrcActor;
use crate::actors::messages::irc_messages::{TimedMode, TimedModeMessage, TimeoutMessage};
use crate::models::reward::{Reward, RewardData};
use crate::models::user::User;
use crate::services::bttv::{slots, swap};
use crate::services::rewards::reply;
use crate::services::rewards::reply::SpotifyAction;
use crate::services::spotify::rewards as spotify;
use crate::services::{ffz, rewards};
use futures::TryFutureExt;

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
            reply::send_emote_reply(
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
            reply::send_emote_reply(
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
                &redemption.event.user_login,
                pool,
            )
            .await;
            reply::send_slot_reply(
                data,
                &irc,
                redemption.event.broadcaster_user_login,
                redemption.event.user_login,
            )
            .await?;
        }
        RewardData::SpotifySkip(_) => {
            let res = spotify::skip_track(&redemption.event.broadcaster_user_id, pool).await;
            reply::send_spotify_reply(
                SpotifyAction::Skip,
                res,
                &irc,
                redemption.event.broadcaster_user_login,
                redemption.event.user_login,
            )
            .await?;
        }
        RewardData::SpotifyPlay(opts) => {
            let res = spotify::get_track_uri_from_input(
                &redemption.event.user_input,
                &redemption.event.broadcaster_user_id,
                &opts,
                pool,
            )
            .and_then(|track| async {
                spotify::play_track(&redemption.event.broadcaster_user_id, track, pool).await
            })
            .await;
            reply::send_spotify_reply(
                SpotifyAction::Play,
                res,
                &irc,
                redemption.event.broadcaster_user_login,
                redemption.event.user_login,
            )
            .await?;
        }
        RewardData::SpotifyQueue(opts) => {
            let res = spotify::get_track_uri_from_input(
                &redemption.event.user_input,
                &redemption.event.broadcaster_user_id,
                &opts,
                pool,
            )
            .and_then(|track| async {
                spotify::queue_track(&redemption.event.broadcaster_user_id, track, pool).await
            })
            .await;
            reply::send_spotify_reply(
                SpotifyAction::Queue,
                res,
                &irc,
                redemption.event.broadcaster_user_login,
                redemption.event.user_login,
            )
            .await?;
        }
    }
    Ok(())
}
