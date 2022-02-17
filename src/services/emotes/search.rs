use anyhow::{anyhow, Result as AnyResult};
use either::Either;
use futures_util::TryFutureExt;
use models::{emote::SlotPlatform, slot::Slot, swap_emote::SwapEmote};
use sqlx::PgPool;

pub async fn search_emote_by_name(
    emote: &str,
    channel_id: &str,
    pool: &PgPool,
) -> AnyResult<Option<Either<Slot, SwapEmote>>> {
    Slot::get_slot_by_emote_name(channel_id, emote, pool)
        .and_then(|opt| async {
            match opt {
                Some(v) => Ok(Some(Either::Left(v))),
                None => SwapEmote::by_name(channel_id, emote, pool)
                    .await
                    .map(|e| e.map(Either::Right)),
            }
        })
        .await
        .map_err(|_| anyhow!("Internal Error"))
}

pub async fn search_by_id(
    channel_id: &str,
    emote_id: &str,
    platform: SlotPlatform,
    pool: &PgPool,
) -> AnyResult<Option<Either<Slot, SwapEmote>>> {
    Slot::get_slot_by_emote_id(channel_id, emote_id, platform, pool)
        .and_then(|opt| async {
            match opt {
                Some(v) => Ok(Some(Either::Left(v))),
                None => SwapEmote::by_id(channel_id, emote_id, platform, pool)
                    .await
                    .map(|e| e.map(Either::Right)),
            }
        })
        .await
        .map_err(|_| anyhow!("Internal Error"))
}
