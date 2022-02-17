use crate::{
    actors::slot::Recheck,
    services::emotes::{
        bttv::BttvEmotes, ffz::FfzEmotes, search::search_by_id, seven_tv::SevenTvEmotes, EmoteRW,
    },
    SlotActor,
};
use actix::SystemService;
use anyhow::{anyhow, Result as AnyResult};
use chrono::{TimeZone, Utc};
use either::Either;
use models::{emote::SlotPlatform, swap_emote::SwapEmote};
use sqlx::PgPool;
use std::str::FromStr;

pub async fn remove_emote(
    channel_id: &str,
    emote_id: &str,
    slot_platform: SlotPlatform,
    pool: &PgPool,
) -> AnyResult<()> {
    match search_by_id(channel_id, emote_id, slot_platform, pool)
        .await?
        .ok_or_else(|| anyhow!("Couldn't find emote"))?
    {
        Either::Left(mut slot) => {
            slot.expires = Some(Utc.timestamp(100000, 0));
            slot.update(pool).await?;
            SlotActor::from_registry().do_send(Recheck);
        }
        Either::Right(swap) => match slot_platform {
            SlotPlatform::Bttv => {
                remove_swap_emote::<BttvEmotes, _, _, _>(channel_id, emote_id, &swap, pool).await?
            }
            SlotPlatform::Ffz => {
                remove_swap_emote::<FfzEmotes, _, _, _>(channel_id, emote_id, &swap, pool).await?
            }
            SlotPlatform::SevenTv => {
                remove_swap_emote::<SevenTvEmotes, _, _, _>(channel_id, emote_id, &swap, pool)
                    .await?
            }
        },
    }
    Ok(())
}

async fn remove_swap_emote<RW, I, E, EI>(
    channel_id: &str,
    emote_id: &str,
    swap: &SwapEmote,
    pool: &PgPool,
) -> AnyResult<()>
where
    RW: EmoteRW<PlatformId = I, Emote = E, EmoteId = EI>,
    EI: FromStr + Default,
{
    let platform_id = RW::get_platform_id(channel_id, pool).await?;
    RW::remove_emote(&platform_id, &EI::from_str(emote_id).unwrap_or_default()).await?;
    SwapEmote::remove(swap.id, pool).await?;
    Ok(())
}
