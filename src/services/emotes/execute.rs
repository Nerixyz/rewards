use crate::actors::irc_actor::IrcActor;
use crate::actors::messages::irc_messages::SayMessage;
use crate::models::reward::SlotRewardData;
use crate::services::emotes::{slots, swap, Emote, EmoteRW};
use actix::Addr;
use anyhow::Result as AnyResult;
use sqlx::PgPool;
use std::fmt::Display;
use std::sync::Arc;
use twitch_api2::eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1;
use twitch_api2::eventsub::NotificationPayload;

pub async fn execute_swap<RW, F, I, E, EI>(
    extractor: F,
    redemption: NotificationPayload<ChannelPointsCustomRewardRedemptionAddV1>,
    pool: &PgPool,
    irc: &Arc<Addr<IrcActor>>,
) -> AnyResult<()>
where
    RW: EmoteRW<PlatformId = I, Emote = E, EmoteId = EI>,
    F: FnOnce(&str) -> AnyResult<&str>,
    I: Display,
    EI: Display + Clone,
    E: Emote<EI>,
{
    let platform_id = extract_id(
        extractor,
        &redemption.event.user_input,
        irc,
        redemption.event.broadcaster_user_login.clone(),
        &redemption.event.user_login,
    )
    .await?;

    log::info!(
        "Adding {:?} emote {} in {}",
        RW::platform(),
        platform_id,
        redemption.event.broadcaster_user_login
    );

    let broadcaster = redemption.event.broadcaster_user_login;
    let user = redemption.event.user_login;
    match swap::swap_or_add_emote::<RW, I, E, EI>(
        &redemption.event.broadcaster_user_id,
        platform_id,
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
    irc: &Arc<Addr<IrcActor>>,
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
        irc,
        redemption.event.broadcaster_user_login.clone(),
        &redemption.event.user_login,
    )
    .await?;
    let broadcaster = redemption.event.broadcaster_user_login;
    let user = redemption.event.user_login;

    log::info!(
        "Adding {:?} emote {} in {}",
        RW::platform(),
        platform_id,
        broadcaster
    );

    match slots::add_slot_emote::<RW, I, E, EI>(
        &redemption.event.broadcaster_user_id,
        &redemption.event.reward.id,
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
    irc: &Arc<Addr<IrcActor>>,
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
