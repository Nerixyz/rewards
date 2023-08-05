use std::fmt::Display;

use anyhow::Result as AnyResult;
use sqlx::PgPool;

use crate::{
    actors::discord::DiscordActor,
    embed_builder, send_discord,
    services::{
        emotes::{slots, swap, Emote, EmoteRW},
        rewards::{extract::EmoteSpec, Redemption},
    },
};
use actix::Addr;
use models::reward::{SlotRewardData, SwapRewardData};
use std::str::FromStr;

pub async fn execute_swap<RW>(
    extractor: impl FnOnce(&str) -> AnyResult<EmoteSpec>,
    redemption: Redemption,
    reward_data: SwapRewardData,
    pool: &PgPool,
    discord: Addr<DiscordActor>,
) -> AnyResult<String>
where
    RW: EmoteRW,
    RW::PlatformId: Display,
    RW::Emote: Emote<RW::EmoteId>,
    RW::EmoteId: Display + Clone + FromStr + Default,
{
    let EmoteSpec {
        id: platform_id,
        override_name,
    } = extractor(&redemption.user_input)?;

    log::info!(
        "Adding {:?} emote {} as {override_name:?} in {}",
        RW::platform(),
        platform_id,
        redemption.broadcaster_user_login
    );

    let user: String = redemption.user_login.take();

    Ok(
        match swap::swap_or_add_emote::<RW>(
            redemption.broadcaster_user_id.as_ref(),
            platform_id,
            override_name,
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
                    redemption.broadcaster_user_id.take(),
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
                    redemption.broadcaster_user_id.take(),
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

pub async fn execute_slot<RW>(
    extractor: impl FnOnce(&str) -> AnyResult<EmoteSpec>,
    redemption: Redemption,
    slot_data: SlotRewardData,
    pool: &PgPool,
    discord: Addr<DiscordActor>,
) -> AnyResult<String>
where
    RW: EmoteRW,
    RW::Emote: Emote<RW::EmoteId>,
    RW::EmoteId: Display,
{
    let EmoteSpec {
        id: platform_id,
        override_name,
    } = extractor(&redemption.user_input)?;

    let broadcaster: String = redemption.broadcaster_user_login.take();
    let user: String = redemption.user_login.take();

    log::info!(
        "Adding {:?} emote {} as {override_name:?} in {}",
        RW::platform(),
        platform_id,
        broadcaster
    );

    let res = slots::add_slot_emote::<RW>(
        redemption.broadcaster_user_id.as_ref(),
        redemption.reward.id.as_ref(),
        slot_data,
        platform_id,
        override_name,
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
        redemption.broadcaster_user_id.take(),
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
