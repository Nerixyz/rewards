use std::sync::Arc;

use actix::Addr;
use anyhow::Result as AnyResult;
use sqlx::PgPool;
use twitch_api2::eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1;
use twitch_api2::eventsub::NotificationPayload;

use crate::actors::irc_actor::IrcActor;
use crate::actors::messages::irc_messages::{TimedModeMessage, TimeoutMessage};
use crate::models::reward::{Reward, RewardData};
use crate::models::timed_mode;
use crate::models::user::User;
use crate::services::emotes::bttv::BttvEmotes;
use crate::services::emotes::execute::{execute_slot, execute_swap};
use crate::services::emotes::ffz::FfzEmotes;
use crate::services::rewards;
use crate::services::rewards::reply::SpotifyAction;
use crate::services::rewards::{extract_bttv_id, extract_ffz_id, reply};
use crate::services::spotify::rewards as spotify;
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
        RewardData::BttvSwap(_) => {
            execute_swap::<BttvEmotes, _, _, _, _>(extract_bttv_id, redemption, pool, &irc).await?;
        }
        RewardData::FfzSwap(_) => {
            execute_swap::<FfzEmotes, _, _, _, _>(extract_ffz_id, redemption, pool, &irc).await?;
        }
        RewardData::BttvSlot(slot) => {
            execute_slot::<BttvEmotes, _, _, _, _>(extract_bttv_id, redemption, slot, pool, &irc)
                .await?;
        }
        RewardData::FfzSlot(slot) => {
            execute_slot::<FfzEmotes, _, _, _, _>(extract_ffz_id, redemption, slot, pool, &irc)
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
