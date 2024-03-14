use std::fmt::Display;

use anyhow::{anyhow, bail, Result as AnyResult};
use sqlx::PgPool;

use crate::{
    actors::discord::DiscordActor,
    chat::parse::opt_next_space,
    embed_builder, send_discord,
    services::{
        emotes::{slots, swap, Emote, EmoteRW},
        rewards::{extract::EmoteSpec, Redemption},
    },
    RedisPool,
};
use actix::Addr;
use models::reward::{SlotRewardData, SwapRewardData};
use std::str::FromStr;

pub async fn execute_swap<RW>(
    extractor: impl FnOnce(&str) -> AnyResult<EmoteSpec>,
    redemption: Redemption,
    reward_data: SwapRewardData,
    pool: &PgPool,
    redis_pool: &RedisPool,
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
            redis_pool,
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
    redis_pool: &RedisPool,
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

    let res = slots::add_slot_emote::<RW>(slots::AddEmoteSlot {
        broadcaster_id: redemption.broadcaster_user_id.as_ref(),
        reward_id: redemption.reward.id.as_ref(),
        slot_data,
        emote_id: platform_id,
        override_name,
        redeemed_user_login: &user,
        pool,
        redis_pool,
    })
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

enum IdOrName<'a> {
    Id(&'a str),
    Name(&'a str),
}

impl<'a> IdOrName<'a> {
    fn parse(s: &'a str, ex: impl FnOnce(&str) -> AnyResult<&str>) -> Self {
        ex(s)
            .map(IdOrName::Id)
            .unwrap_or_else(|_| IdOrName::Name(opt_next_space(s).0))
    }
}

impl Display for IdOrName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdOrName::Id(i) => write!(f, "id:{i}"),
            IdOrName::Name(n) => write!(f, "name:{n}"),
        }
    }
}

pub async fn execute_remove_emote<RW>(
    extract_id: impl FnOnce(&str) -> AnyResult<&str>,
    redemption: Redemption,
    pool: &PgPool,
    redis_pool: &RedisPool,
    discord: Addr<DiscordActor>,
) -> AnyResult<String>
where
    RW: EmoteRW,
    RW::Emote: Emote<RW::EmoteId>,
    RW::EmoteId: Display + FromStr + PartialEq,
{
    let spec = IdOrName::parse(&redemption.user_input, extract_id);
    let broadcaster: String = redemption.broadcaster_user_login.take();
    let user: String = redemption.user_login.take();

    log::info!(
        "Removing {:?} emote {} from {}",
        RW::platform(),
        spec,
        broadcaster
    );

    let emotes = RW::get_emotes(redemption.broadcaster_user_id.as_ref(), pool)
        .await
        .map_err(|e| anyhow!("Failed to list emotes in channel ({e})"))?;
    let emote = match spec {
        IdOrName::Id(id) => {
            // ugh, this should be &str or usize for FFZ
            let Ok(id) = RW::EmoteId::from_str(id) else {
                bail!("Invalid emote ID");
            };
            emotes.iter().find(|e| e.id() == &id)
        }
        IdOrName::Name(name) => emotes.iter().find(|e| e.name() == name),
    };
    let Some(emote) = emote else {
        return Err(anyhow!("This {} emote isn't added!", RW::platform()));
    };
    // TODO: ugh
    let emote_id = emote.id().to_string();
    RW::remove_emote_from_broadcaster(
        redemption.broadcaster_user_id.as_str(),
        &emote_id,
        pool,
        redis_pool,
    )
    .await
    .map_err(|e| {
        anyhow!("Failed to remove emote from {} ({e})", RW::platform())
    })?;

    send_discord!(
        discord,
        redemption.broadcaster_user_id.take(),
        embed_builder!(
            "Emotes",
            "Removed an emote",
            0x00c8af,
            "User" = user.clone(),
            "Removed" = emote.name().to_owned();
            image = Some(RW::format_emote_url(&emote_id)),
            url = Some(RW::format_emote_page(&emote_id)),
        )
    );

    Ok(format!("🗑 Removed {}", emote.name()))
}
