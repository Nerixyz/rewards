use std::sync::Arc;

use actix::Addr;
use anyhow::{anyhow, bail, Error as AnyError, Result as AnyResult};
use futures::TryFutureExt;
use sqlx::PgPool;
use tokio::sync::RwLock;
use twitch_api2::twitch_oauth2::AppAccessToken;

use crate::{
    actors::{
        discord::DiscordActor,
        irc::IrcActor,
        timeout::{ChannelTimeoutMessage, CheckValidTimeoutMessage},
    },
    log_err,
    services::{
        emotes::{
            execute::{execute_remove_emote, execute_slot, execute_swap},
            Emote, EmoteRW,
        },
        ivr,
        rewards::{
            extract,
            reply::{
                format_spotify_result, get_reply_data, reply_to_redemption,
                SpotifyAction,
            },
            Redemption,
        },
        spotify::rewards as spotify,
        twitch::requests::get_user_by_login,
    },
    twitch,
    twitch::requests::{get_chat_settings, timeout_user, update_chat_settings},
    RedisPool, TimeoutActor,
};
use config::CONFIG;
use models::{
    reward::{
        RemEmoteRewardData, SlotRewardData, SpotifyPlayOptions, SwapRewardData,
        TimeoutRewardData,
    },
    timed_mode,
    user::User,
};
use std::{fmt::Display, str::FromStr};

use super::extract::EmoteSpec;

pub async fn timeout(
    timeout: TimeoutRewardData,
    redemption: Redemption,
    broadcaster: User,
    (irc, app_token, timeout_handler): (
        Addr<IrcActor>,
        Arc<RwLock<AppAccessToken>>,
        Addr<TimeoutActor>,
    ),
) -> AnyResult<()> {
    let reply_data = get_reply_data(&redemption);
    let reply_irc_addr = irc.clone();
    let result = async move {
        // check timeout
        let username =
            extract::username(&redemption.user_input)?.to_lowercase();
        let duration = extract::duration(&timeout.duration)?;

        let user =
            get_user_by_login(username.clone(), &*app_token.read().await)
                .await
                .map_err(|e| {
                    AnyError::msg(format!(
                        "This user doesn't seem to exist: {}",
                        e
                    ))
                })?;

        if timeout.vip {
            let vips = ivr::modvips(redemption.broadcaster_user_login.as_str())
                .await
                .map_err(|e| anyhow!("Attempt to read VIPs failed: {}", e))?
                .vips;
            if vips.iter().any(|v| v.id == user.id.as_str()) {
                return Err(anyhow!("I won't timeout VIPs."));
            }
        }

        let token = twitch::get_token();

        if !timeout_handler
            .send(CheckValidTimeoutMessage {
                channel_id: broadcaster.id.clone(),
                user_id: user.id.clone().take(),
            })
            .await
            .map_err(|_| anyhow!("Too much traffic"))?
            .map_err(|e| {
                anyhow!(
                    "Cannot check {}'s timeout/ban status: {}",
                    user.login,
                    e
                )
            })?
        {
            return Err(anyhow!(
                "This user was timed out by another moderator."
            ));
        }

        timeout_handler
            .send(ChannelTimeoutMessage {
                channel_id: broadcaster.id.clone(),
                user_id: user.id.clone().take(),
                duration: std::time::Duration::from_secs(duration),
                is_self: true,
            })
            .await
            .map_err(|_| anyhow!("Too much traffic"))?;

        timeout_user(
            &broadcaster.id,
            &CONFIG.twitch.user_id,
            user.id,
            std::time::Duration::from_secs(duration),
            format!("Redemption from {}", redemption.user_login).as_str(),
            &token,
        )
        .await
        .map_err(|e| anyhow!("Cannot timeout user: {e}"))?;

        Ok(())
    }
    .await;
    if result.is_err() {
        reply_to_redemption(
            result.map(|_| unreachable!("only errors are printed")),
            &reply_irc_addr,
            reply_data.0,
            reply_data.1,
        )
        .await?;
    }
    Ok(())
}

pub async fn timed_mode(
    mode: timed_mode::Mode,
    duration: String,
    broadcaster: User,
    redemption: Redemption,
    irc: Addr<IrcActor>,
) -> AnyResult<()> {
    let reply_data = get_reply_data(&redemption);
    let duration =
        extract::duration(&duration).map(std::time::Duration::from_secs)?;
    let broadcaster_id = broadcaster.id.clone();
    let token = twitch::get_token();

    let res = async move {
        let chat_settings = get_chat_settings(&broadcaster.id, &token)
            .await
            .map_err(|e| anyhow!("Cannot get chat settings {e}"))?;
        match mode {
            timed_mode::Mode::Subonly if chat_settings.subscriber_mode => {
                bail!("This chat is already in subscriber mode.");
            }
            timed_mode::Mode::Emoteonly if chat_settings.emote_mode => {
                bail!("This chat is already in emote only mode.");
            }
            _ => (),
        }

        update_chat_settings(
            &broadcaster.id,
            CONFIG.twitch.user_id.clone(),
            mode,
            true,
            &token,
        )
        .await
        .map_err(|e| anyhow!("Cannot update chat settings: {e}"))?;

        tokio::spawn(async move {
            tokio::time::sleep(duration).await;
            let res = update_chat_settings(
                &broadcaster_id,
                CONFIG.twitch.user_id.clone(),
                mode,
                false,
                &token,
            )
            .await;
            log_err!(res, "cannot update chat settings");
        });
        Ok(())
    }
    .await;

    if let Err(e) = res {
        reply_to_redemption(Err(e), &irc, reply_data.0, reply_data.1).await?;
    }
    Ok(())
}

pub async fn swap<RW>(
    extractor: impl FnOnce(&str) -> AnyResult<EmoteSpec>,
    redemption: Redemption,
    data: SwapRewardData,
    (db, redis_pool, irc, discord): (
        PgPool,
        RedisPool,
        Addr<IrcActor>,
        Addr<DiscordActor>,
    ),
) -> AnyResult<()>
where
    RW: EmoteRW,
    RW::PlatformId: Display,
    RW::Emote: Emote<RW::EmoteId>,
    RW::EmoteId: Display + Clone + FromStr + Default,
{
    let (broadcaster, user) = get_reply_data(&redemption);
    let should_reply = data.reply;
    let res = execute_swap::<RW>(
        extractor,
        redemption,
        data,
        &db,
        &redis_pool,
        discord,
    )
    .await;
    reply_to_redemption(
        res.map(|r| should_reply.then_some(r)),
        &irc,
        broadcaster,
        user,
    )
    .await
}

pub async fn slot<RW>(
    extractor: impl FnOnce(&str) -> AnyResult<EmoteSpec>,
    redemption: Redemption,
    slot: SlotRewardData,
    (db, redis, irc, discord): (
        PgPool,
        RedisPool,
        Addr<IrcActor>,
        Addr<DiscordActor>,
    ),
) -> AnyResult<()>
where
    RW: EmoteRW,
    RW::Emote: Emote<RW::EmoteId>,
    RW::EmoteId: Display,
{
    let (broadcaster, user) = get_reply_data(&redemption);
    let should_reply = slot.reply;
    let res =
        execute_slot::<RW>(extractor, redemption, slot, &db, &redis, discord)
            .await;
    reply_to_redemption(
        res.map(|r| should_reply.then_some(r)),
        &irc,
        broadcaster,
        user,
    )
    .await
}

pub async fn rem_emote<RW>(
    redemption: Redemption,
    data: RemEmoteRewardData,
    (db, redis, irc, discord): (
        PgPool,
        RedisPool,
        Addr<IrcActor>,
        Addr<DiscordActor>,
    ),
) -> AnyResult<()>
where
    RW: EmoteRW,
    RW::Emote: Emote<RW::EmoteId>,
    RW::EmoteId: Display,
{
    let (broadcaster, user) = get_reply_data(&redemption);
    let should_reply = data.reply;
    let res =
        execute_remove_emote::<RW>(redemption, &db, &redis, discord).await;
    reply_to_redemption(
        res.map(|r| should_reply.then_some(r)),
        &irc,
        broadcaster,
        user,
    )
    .await
}

pub async fn spotify_skip(
    redemption: Redemption,
    (db, irc): (PgPool, Addr<IrcActor>),
) -> AnyResult<()> {
    let (broadcaster, user) = get_reply_data(&redemption);
    let res =
        spotify::skip_track(redemption.broadcaster_user_id.as_ref(), &db).await;
    reply_to_redemption(
        format_spotify_result(res, SpotifyAction::Skip).map(Some),
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
        spotify::play_track(redemption.broadcaster_user_id.as_ref(), track, &db)
            .await
    })
    .await;
    reply_to_redemption(
        format_spotify_result(res, SpotifyAction::Play).map(Some),
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
        spotify::queue_track(
            redemption.broadcaster_user_id.as_ref(),
            track,
            &db,
        )
        .await
    })
    .await;
    reply_to_redemption(
        format_spotify_result(res, SpotifyAction::Queue).map(Some),
        &irc,
        broadcaster,
        user,
    )
    .await
}
