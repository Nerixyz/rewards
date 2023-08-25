use crate::{
    actors::slot::Recheck,
    services::{
        emotes::{
            bttv::BttvEmotes, ffz::FfzEmotes, search::search_by_id,
            seven_tv::SevenTvEmotes, EmoteRW,
        },
        twitch::requests::update_reward,
    },
    AnyError, RedisPool, SlotActor,
};
use actix::SystemService;
use anyhow::{anyhow, Result as AnyResult};
use chrono::{TimeZone, Utc};
use either::Either;
use futures_util::{future, TryFutureExt};
use models::{
    emote::SlotPlatform, slot::Slot, swap_emote::SwapEmote, user::User,
};
use sqlx::PgPool;
use std::str::FromStr;
use twitch_api2::{
    helix::points::UpdateCustomRewardBody, twitch_oauth2::UserToken,
};

/// Untracks and removes the emote both from the db and the platform.
pub async fn remove_emote(
    channel_id: &str,
    emote_id: &str,
    slot_platform: SlotPlatform,
    pool: &PgPool,
    redis_pool: &RedisPool,
) -> AnyResult<()> {
    match search_by_id(channel_id, emote_id, slot_platform, pool)
        .await?
        .ok_or_else(|| anyhow!("Couldn't find emote"))?
    {
        Either::Left(mut slot) => {
            slot.expires = Some(Utc.timestamp_nanos(0));
            slot.update(pool).await?;
            SlotActor::from_registry().do_send(Recheck);
        }
        Either::Right(swap) => match slot_platform {
            SlotPlatform::Bttv => {
                remove_swap_emote::<BttvEmotes, _, _, _>(
                    channel_id, emote_id, &swap, pool, redis_pool,
                )
                .await?
            }
            SlotPlatform::Ffz => {
                remove_swap_emote::<FfzEmotes, _, _, _>(
                    channel_id, emote_id, &swap, pool, redis_pool,
                )
                .await?
            }
            SlotPlatform::SevenTv => {
                remove_swap_emote::<SevenTvEmotes, _, _, _>(
                    channel_id, emote_id, &swap, pool, redis_pool,
                )
                .await?
            }
        },
    }
    Ok(())
}

/// Only untracks the emote in the database. Doesn't remove the emote from the platform.
pub async fn untrack_emote(
    channel_id: &str,
    emote_id: &str,
    slot_platform: SlotPlatform,
    pool: &PgPool,
) -> AnyResult<String> {
    let name = match search_by_id(channel_id, emote_id, slot_platform, pool)
        .await?
        .ok_or_else(|| anyhow!("Couldn't find emote"))?
    {
        Either::Left(slot) => {
            future::try_join(
                Slot::clear(slot.id, pool).map_err(AnyError::from),
                enable_reward(&slot, pool),
            )
            .await?;
            slot.name.unwrap_or_else(|| "<empty slot>".to_string())
        }
        Either::Right(swap) => {
            SwapEmote::remove(swap.id, pool).await?;
            swap.name
        }
    };
    Ok(name)
}

pub async fn enable_reward(slot: &Slot, pool: &PgPool) -> AnyResult<()> {
    let user = User::get_by_id(&slot.user_id, pool).await?;
    let token: UserToken = user.into();
    update_reward(
        token.user_id.clone(),
        slot.reward_id.clone(),
        UpdateCustomRewardBody::builder()
            .is_paused(Some(false))
            .build(),
        &token,
    )
    .await?;
    Ok(())
}

async fn remove_swap_emote<RW, I, E, EI>(
    channel_id: &str,
    emote_id: &str,
    swap: &SwapEmote,
    pool: &PgPool,
    redis_pool: &RedisPool,
) -> AnyResult<()>
where
    RW: EmoteRW<PlatformId = I, Emote = E, EmoteId = EI>,
    EI: FromStr + Default,
{
    let platform_id = RW::get_platform_id(channel_id, pool).await?;
    RW::remove_emote(
        &platform_id,
        &EI::from_str(emote_id).unwrap_or_default(),
        redis_pool,
    )
    .await?;
    SwapEmote::remove(swap.id, pool).await?;
    Ok(())
}
