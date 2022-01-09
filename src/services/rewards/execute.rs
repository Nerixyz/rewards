use std::sync::Arc;

use actix::Addr;
use anyhow::{Error as AnyError, Result as AnyResult};
use futures::TryFutureExt;
use sqlx::PgPool;
use tokio::sync::RwLock;
use twitch_api2::twitch_oauth2::AppAccessToken;

use crate::{
    actors::{
        discord::DiscordActor,
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
        rewards::{
            extract,
            reply::{format_spotify_result, get_reply_data, reply_to_redemption, SpotifyAction},
            Redemption,
        },
        spotify::rewards as spotify,
        twitch::requests::get_user_by_login,
    },
};
use std::{fmt::Display, str::FromStr};

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
    let username = extract::username(&redemption.user_input)?.to_lowercase();
    let user = get_user_by_login(username.clone(), &*app_token.read().await)
        .await
        .map_err(|e| AnyError::msg(format!("Could not get user: {}", e)))?;

    let ok_timeout = timeout_handler
        .send(CheckValidTimeoutMessage {
            channel_id: redemption.broadcaster_user_id.clone().into_string(),
            user_id: user.id.clone().into_string(),
        })
        .await
        .map_err(|_| AnyError::msg("Too much traffic"))?
        .map_err(|_| AnyError::msg("Internal error"))?;

    if !ok_timeout {
        return Err(AnyError::msg("Can't timeout this user"));
    }

    irc.send(TimeoutMessage {
        user: extract::username(&redemption.user_input)?,
        user_id: user.id.into_string(),
        duration: extract::duration(&timeout)?,
        broadcaster: broadcaster.name,
        broadcaster_id: redemption.broadcaster_user_id.into_string(),
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
    (db, irc, discord): (PgPool, Addr<IrcActor>, Addr<DiscordActor>),
) -> AnyResult<()>
where
    RW: EmoteRW<PlatformId = I, Emote = E, EmoteId = EI>,
    F: FnOnce(&str) -> AnyResult<&str>,
    I: Display,
    EI: Display + Clone + FromStr + Default,
    E: Emote<EI>,
{
    let (broadcaster, user) = get_reply_data(&redemption);
    let res = execute_swap::<RW, F, I, E, EI>(extractor, redemption, data, &db, discord).await;
    reply_to_redemption(res, &irc, broadcaster, user).await
}

pub async fn slot<RW, F, I, E, EI>(
    extractor: F,
    redemption: Redemption,
    slot: SlotRewardData,
    (db, irc, discord): (PgPool, Addr<IrcActor>, Addr<DiscordActor>),
) -> AnyResult<()>
where
    RW: EmoteRW<PlatformId = I, Emote = E, EmoteId = EI>,
    F: FnOnce(&str) -> AnyResult<&str>,
    E: Emote<EI>,
    EI: Display,
{
    let (broadcaster, user) = get_reply_data(&redemption);
    let res = execute_slot::<RW, F, I, E, EI>(extractor, redemption, slot, &db, discord).await;
    reply_to_redemption(res, &irc, broadcaster, user).await
}

pub async fn spotify_skip(
    redemption: Redemption,
    (db, irc): (PgPool, Addr<IrcActor>),
) -> AnyResult<()> {
    let (broadcaster, user) = get_reply_data(&redemption);
    let res = spotify::skip_track(redemption.broadcaster_user_id.as_ref(), &db).await;
    reply_to_redemption(
        format_spotify_result(res, SpotifyAction::Skip),
        &irc,
        broadcaster,
        user,
    )
    .await
}

pub async fn spotify_play(
    opts: SpotifyPlayOptions,
    redemption: Redemption,
    (db, irc): (PgPool, Addr<IrcActor>),
) -> AnyResult<()> {
    let (broadcaster, user) = get_reply_data(&redemption);
    let res = spotify::get_track_uri_from_input(
        &redemption.user_input,
        redemption.broadcaster_user_id.as_ref(),
        &opts,
        &db,
    )
    .and_then(|track| async {
        spotify::play_track(redemption.broadcaster_user_id.as_ref(), track, &db).await
    })
    .await;
    reply_to_redemption(
        format_spotify_result(res, SpotifyAction::Play),
        &irc,
        broadcaster,
        user,
    )
    .await
}

pub async fn spotify_queue(
    opts: SpotifyPlayOptions,
    redemption: Redemption,
    (db, irc): (PgPool, Addr<IrcActor>),
) -> AnyResult<()> {
    let (broadcaster, user) = get_reply_data(&redemption);
    let res = spotify::get_track_uri_from_input(
        &redemption.user_input,
        redemption.broadcaster_user_id.as_ref(),
        &opts,
        &db,
    )
    .and_then(|track| async {
        spotify::queue_track(redemption.broadcaster_user_id.as_ref(), track, &db).await
    })
    .await;
    reply_to_redemption(
        format_spotify_result(res, SpotifyAction::Queue),
        &irc,
        broadcaster,
        user,
    )
    .await
}
