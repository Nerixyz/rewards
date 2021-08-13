use std::fmt::Display;

use actix::Addr;
use anyhow::Result as AnyResult;
use sqlx::PgPool;
use twitch_api2::eventsub::{
    channel::ChannelPointsCustomRewardRedemptionAddV1, NotificationPayload,
};

use crate::{
    actors::irc::{IrcActor, SayMessage},
    models::reward::{SlotRewardData, SwapRewardData},
    services::emotes::{slots, swap, Emote, EmoteRW},
};
use std::str::FromStr;

pub async fn execute_swap<RW, F, I, E, EI>(
    extractor: F,
    redemption: NotificationPayload<ChannelPointsCustomRewardRedemptionAddV1>,
    reward_data: SwapRewardData,
    pool: &PgPool,
    irc: Addr<IrcActor>,
) -> AnyResult<()>
where
    RW: EmoteRW<PlatformId = I, Emote = E, EmoteId = EI>,
    F: FnOnce(&str) -> AnyResult<&str>,
    I: Display,
    EI: Display + Clone + FromStr + Default,
    E: Emote<EI>,
{
    let platform_id = extract_id(
        extractor,
        &redemption.event.user_input,
        &irc,
        redemption
            .event
            .broadcaster_user_login
            .clone()
            .into_string(),
        redemption.event.user_login.as_ref(),
    )
    .await?;

    log::info!(
        "Adding {:?} emote {} in {}",
        RW::platform(),
        platform_id,
        redemption.event.broadcaster_user_login
    );

    let broadcaster: String = redemption.event.broadcaster_user_login.into_string();
    let user: String = redemption.event.user_login.into_string();
    match swap::swap_or_add_emote::<RW, I, E, EI>(
        redemption.event.broadcaster_user_id.as_ref(),
        platform_id,
        reward_data,
        &user,
        pool,
    )
    .await
    {
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

pub async fn execute_slot<RW, F, I, E, EI>(
    extractor: F,
    redemption: NotificationPayload<ChannelPointsCustomRewardRedemptionAddV1>,
    slot_data: SlotRewardData,
    pool: &PgPool,
    irc: Addr<IrcActor>,
) -> AnyResult<()>
where
    RW: EmoteRW<PlatformId = I, Emote = E, EmoteId = EI>,
    F: FnOnce(&str) -> AnyResult<&str>,
    E: Emote<EI>,
    EI: Display,
{
    let platform_id = extract_id(
        extractor,
        &redemption.event.user_input,
        &irc,
        redemption
            .event
            .broadcaster_user_login
            .clone()
            .into_string(),
        redemption.event.user_login.as_ref(),
    )
    .await?;
    let broadcaster: String = redemption.event.broadcaster_user_login.into_string();
    let user: String = redemption.event.user_login.into_string();

    log::info!(
        "Adding {:?} emote {} in {}",
        RW::platform(),
        platform_id,
        broadcaster
    );

    match slots::add_slot_emote::<RW, I, E, EI>(
        redemption.event.broadcaster_user_id.as_ref(),
        redemption.event.reward.id.as_ref(),
        slot_data,
        platform_id,
        &user,
        pool,
    )
    .await
    {
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

async fn extract_id<'a, F>(
    extractor: F,
    input: &'a str,
    irc: &Addr<IrcActor>,
    broadcaster: String,
    user: &str,
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
