use std::sync::Arc;

use actix::Addr;
use anyhow::{Error as AnyError, Result as AnyResult};
use sqlx::PgPool;
use twitch_api2::eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1;
use twitch_api2::eventsub::NotificationPayload;

use crate::actors::irc_actor::IrcActor;
use crate::actors::messages::irc_messages::{TimedModeMessage, TimeoutMessage};
use crate::actors::messages::timeout_messages::CheckValidTimeoutMessage;
use crate::actors::timeout_actor::TimeoutActor;
use crate::models::reward::{Reward, RewardData};
use crate::models::timed_mode;
use crate::models::user::User;
use crate::services::emotes::bttv::BttvEmotes;
use crate::services::emotes::execute::{execute_slot, execute_swap};
use crate::services::emotes::ffz::FfzEmotes;
use crate::services::emotes::seven_tv::SevenTvEmotes;
use crate::services::rewards;
use crate::services::rewards::reply::SpotifyAction;
use crate::services::rewards::{extract_bttv_id, extract_ffz_id, extract_seventv_id, reply};
use crate::services::spotify::rewards as spotify;
use crate::services::twitch::requests::get_user_by_login;
use futures::TryFutureExt;
use tokio::sync::RwLock;
use twitch_api2::twitch_oauth2::AppAccessToken;

/// This doesn't update the reward-redemption on twitch!
pub async fn execute_reward(
    redemption: NotificationPayload<ChannelPointsCustomRewardRedemptionAddV1>,
    reward: Reward,
    broadcaster: User,
    pool: &PgPool,
    irc: Arc<Addr<IrcActor>>,
    timeout_handler: Arc<Addr<TimeoutActor>>,
    app_token: Arc<RwLock<AppAccessToken>>,
) -> AnyResult<()> {
    match reward.data.0 {
        RewardData::Timeout(timeout) => {
            // check timeout
            let username = rewards::extract_username(&redemption.event.user_input)?.to_lowercase();
            let user = get_user_by_login(username.clone(), &*app_token.read().await)
                .await
                .map_err(|_| AnyError::msg("Could not get user"))?;

            let ok_timeout = timeout_handler
                .send(CheckValidTimeoutMessage {
                    channel_id: redemption.event.broadcaster_user_id.clone().into_string(),
                    user_id: user.id.clone().into_string(),
                })
                .await
                .map_err(|_| AnyError::msg("Too much traffic"))?
                .map_err(|_| AnyError::msg("Internal error"))?;

            if !ok_timeout {
                return Err(AnyError::msg("Can't timeout this user"));
            }

            irc.send(TimeoutMessage {
                user: rewards::extract_username(&redemption.event.user_input)?,
                user_id: user.id.into_string(),
                duration: rewards::get_duration(&timeout)?,
                broadcaster: broadcaster.name,
                broadcaster_id: redemption.event.broadcaster_user_id.into_string(),
            })
            .await??
        }
        RewardData::EmoteOnly(duration) => {
            irc.send(TimedModeMessage {
                duration: rewards::get_duration(&duration)?,
                broadcaster: broadcaster.name,
                broadcaster_id: broadcaster.id,
                mode: timed_mode::Mode::Emoteonly,
            })
            .await?
        }
        RewardData::SubOnly(duration) => {
            irc.send(TimedModeMessage {
                duration: rewards::get_duration(&duration)?,
                broadcaster: broadcaster.name,
                broadcaster_id: broadcaster.id,
                mode: timed_mode::Mode::Subonly,
            })
            .await?
        }
        RewardData::BttvSwap(data) => {
            execute_swap::<BttvEmotes, _, _, _, _>(extract_bttv_id, redemption, data, pool, &irc)
                .await?;
        }
        RewardData::FfzSwap(data) => {
            execute_swap::<FfzEmotes, _, _, _, _>(extract_ffz_id, redemption, data, pool, &irc)
                .await?;
        }
        RewardData::SevenTvSwap(data) => {
            execute_swap::<SevenTvEmotes, _, _, _, _>(
                extract_seventv_id,
                redemption,
                data,
                pool,
                &irc,
            )
            .await?;
        }
        RewardData::BttvSlot(slot) => {
            execute_slot::<BttvEmotes, _, _, _, _>(extract_bttv_id, redemption, slot, pool, &irc)
                .await?;
        }
        RewardData::FfzSlot(slot) => {
            execute_slot::<FfzEmotes, _, _, _, _>(extract_ffz_id, redemption, slot, pool, &irc)
                .await?;
        }
        RewardData::SevenTvSlot(slot) => {
            execute_slot::<SevenTvEmotes, _, _, _, _>(
                extract_seventv_id,
                redemption,
                slot,
                pool,
                &irc,
            )
            .await?;
        }
        RewardData::SpotifySkip(_) => {
            let res =
                spotify::skip_track(redemption.event.broadcaster_user_id.as_ref(), pool).await;
            reply::send_spotify_reply(
                SpotifyAction::Skip,
                res,
                &irc,
                redemption.event.broadcaster_user_login.into_string(),
                redemption.event.user_login.into_string(),
            )
            .await?;
        }
        RewardData::SpotifyPlay(opts) => {
            let res = spotify::get_track_uri_from_input(
                &redemption.event.user_input,
                redemption.event.broadcaster_user_id.as_ref(),
                &opts,
                pool,
            )
            .and_then(|track| async {
                spotify::play_track(redemption.event.broadcaster_user_id.as_ref(), track, pool)
                    .await
            })
            .await;
            reply::send_spotify_reply(
                SpotifyAction::Play,
                res,
                &irc,
                redemption.event.broadcaster_user_login.into_string(),
                redemption.event.user_login.into_string(),
            )
            .await?;
        }
        RewardData::SpotifyQueue(opts) => {
            let res = spotify::get_track_uri_from_input(
                &redemption.event.user_input,
                redemption.event.broadcaster_user_id.as_ref(),
                &opts,
                pool,
            )
            .and_then(|track| async {
                spotify::queue_track(redemption.event.broadcaster_user_id.as_ref(), track, pool)
                    .await
            })
            .await;
            reply::send_spotify_reply(
                SpotifyAction::Queue,
                res,
                &irc,
                redemption.event.broadcaster_user_login.into_string(),
                redemption.event.user_login.into_string(),
            )
            .await?;
        }
    }
    Ok(())
}
