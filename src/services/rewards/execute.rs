use std::sync::Arc;

use actix::Addr;
use anyhow::{Error as AnyError, Result as AnyResult};
use futures::TryFutureExt;
use sqlx::PgPool;
use tokio::sync::RwLock;
use twitch_api2::{
    eventsub::{channel::ChannelPointsCustomRewardRedemptionAddV1, NotificationPayload},
    twitch_oauth2::AppAccessToken,
};

use crate::{
    actors::{
        irc::{IrcActor, TimedModeMessage, TimeoutMessage},
        timeout::{CheckValidTimeoutMessage, TimeoutActor},
    },
    models::{
        reward::{SlotRewardData, SpotifyPlayOptions, SwapRewardData},
        timed_mode,
        user::User,
    },
    services::{
        emotes::{
            execute::{execute_slot, execute_swap},
            Emote, EmoteRW,
        },
        rewards::{extract, reply, reply::SpotifyAction},
        spotify::rewards as spotify,
        twitch::requests::get_user_by_login,
    },
};
use std::{fmt::Display, str::FromStr};

type Redemption = NotificationPayload<ChannelPointsCustomRewardRedemptionAddV1>;

pub async fn timeout(
    timeout: String,
    redemption: Redemption,
    broadcaster: User,
    (irc, app_token, timeout_handler): (
        Addr<IrcActor>,
        Arc<RwLock<AppAccessToken>>,
        Addr<TimeoutActor>,
    ),
) -> AnyResult<()> {
    // check timeout
    let username = extract::username(&redemption.event.user_input)?.to_lowercase();
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
        user: extract::username(&redemption.event.user_input)?,
        user_id: user.id.into_string(),
        duration: extract::duration(&timeout)?,
        broadcaster: broadcaster.name,
        broadcaster_id: redemption.event.broadcaster_user_id.into_string(),
    })
    .await??;
    Ok(())
}

pub async fn timed_mode(
    mode: timed_mode::Mode,
    duration: String,
    broadcaster: User,
    irc: Addr<IrcActor>,
) -> AnyResult<()> {
    irc.send(TimedModeMessage {
        duration: extract::duration(&duration)?,
        broadcaster: broadcaster.name,
        broadcaster_id: broadcaster.id,
        mode,
    })
    .await?;
    Ok(())
}

pub async fn swap<RW, F, I, E, EI>(
    extractor: F,
    redemption: Redemption,
    data: SwapRewardData,
    (db, irc): (PgPool, Addr<IrcActor>),
) -> AnyResult<()>
where
    RW: EmoteRW<PlatformId = I, Emote = E, EmoteId = EI>,
    F: FnOnce(&str) -> AnyResult<&str>,
    I: Display,
    EI: Display + Clone + FromStr + Default,
    E: Emote<EI>,
{
    execute_swap::<RW, F, I, E, EI>(extractor, redemption, data, &db, irc).await?;
    Ok(())
}

pub async fn slot<RW, F, I, E, EI>(
    extractor: F,
    redemption: Redemption,
    slot: SlotRewardData,
    (db, irc): (PgPool, Addr<IrcActor>),
) -> AnyResult<()>
where
    RW: EmoteRW<PlatformId = I, Emote = E, EmoteId = EI>,
    F: FnOnce(&str) -> AnyResult<&str>,
    E: Emote<EI>,
    EI: Display,
{
    execute_slot::<RW, F, I, E, EI>(extractor, redemption, slot, &db, irc).await?;
    Ok(())
}

pub async fn spotify_skip(
    redemption: Redemption,
    (db, irc): (PgPool, Addr<IrcActor>),
) -> AnyResult<()> {
    let res = spotify::skip_track(redemption.event.broadcaster_user_id.as_ref(), &db).await;
    reply::send_spotify_reply(
        SpotifyAction::Skip,
        res,
        irc,
        redemption.event.broadcaster_user_login.into_string(),
        redemption.event.user_login.into_string(),
    )
    .await?;
    Ok(())
}

pub async fn spotify_play(
    opts: SpotifyPlayOptions,
    redemption: Redemption,
    (db, irc): (PgPool, Addr<IrcActor>),
) -> AnyResult<()> {
    let res = spotify::get_track_uri_from_input(
        &redemption.event.user_input,
        redemption.event.broadcaster_user_id.as_ref(),
        &opts,
        &db,
    )
    .and_then(|track| async {
        spotify::play_track(redemption.event.broadcaster_user_id.as_ref(), track, &db).await
    })
    .await;
    reply::send_spotify_reply(
        SpotifyAction::Play,
        res,
        irc,
        redemption.event.broadcaster_user_login.into_string(),
        redemption.event.user_login.into_string(),
    )
    .await?;
    Ok(())
}

pub async fn spotify_queue(
    opts: SpotifyPlayOptions,
    redemption: Redemption,
    (db, irc): (PgPool, Addr<IrcActor>),
) -> AnyResult<()> {
    let res = spotify::get_track_uri_from_input(
        &redemption.event.user_input,
        redemption.event.broadcaster_user_id.as_ref(),
        &opts,
        &db,
    )
    .and_then(|track| async {
        spotify::queue_track(redemption.event.broadcaster_user_id.as_ref(), track, &db).await
    })
    .await;
    reply::send_spotify_reply(
        SpotifyAction::Queue,
        res,
        irc,
        redemption.event.broadcaster_user_login.into_string(),
        redemption.event.user_login.into_string(),
    )
    .await?;
    Ok(())
}
