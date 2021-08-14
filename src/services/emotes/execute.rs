use std::fmt::Display;

use anyhow::Result as AnyResult;
use sqlx::PgPool;
use twitch_api2::eventsub::{
    channel::ChannelPointsCustomRewardRedemptionAddV1, NotificationPayload,
};

use crate::{
    actors::discord::DiscordActor,
    embed_builder,
    models::reward::{SlotRewardData, SwapRewardData},
    send_discord,
    services::emotes::{slots, swap, Emote, EmoteRW},
};
use actix::Addr;
use std::str::FromStr;

pub async fn execute_swap<RW, F, I, E, EI>(
    extractor: F,
    redemption: NotificationPayload<ChannelPointsCustomRewardRedemptionAddV1>,
    reward_data: SwapRewardData,
    pool: &PgPool,
    discord: Addr<DiscordActor>,
) -> AnyResult<String>
where
    RW: EmoteRW<PlatformId = I, Emote = E, EmoteId = EI>,
    F: FnOnce(&str) -> AnyResult<&str>,
    I: Display,
    EI: Display + Clone + FromStr + Default,
    E: Emote<EI>,
{
    let platform_id = extractor(&redemption.event.user_input)?;

    log::info!(
        "Adding {:?} emote {} in {}",
        RW::platform(),
        platform_id,
        redemption.event.broadcaster_user_login
    );

    let user: String = redemption.event.user_login.into_string();

    Ok(
        match swap::swap_or_add_emote::<RW, I, E, EI>(
            redemption.event.broadcaster_user_id.as_ref(),
            platform_id,
            reward_data,
            &user,
            pool,
        )
        .await?
        {
            (Some(removed), added) => {
                let msg = format!("☑ Added {} - 🗑 Removed {}", added, removed);
                send_discord!(
                    discord,
                    redemption.event.user_id.into_string(),
                    embed_builder!(
                        "Emotes",
                        "Added an emote",
                        0x00c8af,
                        "User" = user.clone(),
                        "Emote" = added,
                        "Removed" = removed;
                        image = Some(RW::format_emote_url(platform_id)),
                        url = Some(RW::format_emote_page(platform_id)),
                    )
                );
                msg
            }
            (None, added) => {
                let msg = format!("☑ Added {}", added);
                send_discord!(
                    discord,
                    redemption.event.broadcaster_user_id.into_string(),
                    embed_builder!(
                        "Emotes",
                        "Added an emote",
                        0x00c8af,
                        "User" = user.clone(),
                        "Emote" = added;
                        image = Some(RW::format_emote_url(platform_id)),
                        url = Some(RW::format_emote_page(platform_id)),
                    )
                );
                msg
            }
        },
    )
}

pub async fn execute_slot<RW, F, I, E, EI>(
    extractor: F,
    redemption: NotificationPayload<ChannelPointsCustomRewardRedemptionAddV1>,
    slot_data: SlotRewardData,
    pool: &PgPool,
    discord: Addr<DiscordActor>,
) -> AnyResult<String>
where
    RW: EmoteRW<PlatformId = I, Emote = E, EmoteId = EI>,
    F: FnOnce(&str) -> AnyResult<&str>,
    E: Emote<EI>,
    EI: Display,
{
    let platform_id = extractor(&redemption.event.user_input)?;

    let broadcaster: String = redemption.event.broadcaster_user_login.into_string();
    let user: String = redemption.event.user_login.into_string();

    log::info!(
        "Adding {:?} emote {} in {}",
        RW::platform(),
        platform_id,
        broadcaster
    );

    let res = slots::add_slot_emote::<RW, I, E, EI>(
        redemption.event.broadcaster_user_id.as_ref(),
        redemption.event.reward.id.as_ref(),
        slot_data,
        platform_id,
        &user,
        pool,
    )
    .await?;

    let msg = match &res {
        (added, remaining) if *remaining > 1 => {
            format!("☑ Added {} - 🔳 {} slots open", added, remaining)
        }
        (added, remaining) if *remaining == 1 => {
            format!("☑ Added {} - 🔳 {} slot open", added, remaining)
        }
        (added, _) => format!("☑ Added {} - 0 slots open - 🔒 closing", added),
    };

    send_discord!(
        discord,
        redemption.event.broadcaster_user_id.into_string(),
        embed_builder!(
            "Emotes",
            "Added an emote",
            0x00c8af,
            "User" = user.clone(),
            "Emote" = res.0;
            image = Some(RW::format_emote_url(platform_id)),
            url = Some(RW::format_emote_page(platform_id)),
        )
    );

    Ok(msg)
}
