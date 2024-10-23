use crate::{
    services::{bttv, ffz, seven_tv},
    PgPool,
};
use anyhow::{bail, Result as AnyResult};
use config::CONFIG;
use futures_util::future;
use models::{emote::SlotPlatform, reward, slot, swap_emote};
use std::fmt::{Display, Formatter};

pub struct EmoteData {
    pub seventv: EpDataOpt,
    pub ffz: EpDataOpt,
    pub bttv: EpDataOpt,
}

pub struct EmotePlatformData {
    pub remaining_emotes: usize,
    pub open_slots: usize,
    pub swap_capacity: usize,
}

#[repr(transparent)]
pub struct EpDataOpt(AnyResult<EmotePlatformData>);

impl EmoteData {
    pub async fn get(
        channel_id: &str,
        channel_login: &str,
        pool: &PgPool,
    ) -> Self {
        let (seventv, ffz, bttv) = future::join3(
            extract_seventv(channel_id, pool),
            extract_ffz(channel_id, channel_login, pool),
            extract_bttv(channel_id, pool),
        )
        .await;

        Self {
            seventv: seventv.into(),
            ffz: ffz.into(),
            bttv: bttv.into(),
        }
    }
}

impl Display for EmotePlatformData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "available={}, open-slots={}, available-swaps={}",
            self.remaining_emotes, self.open_slots, self.swap_capacity
        )
    }
}

impl Display for EpDataOpt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Ok(epd) => Display::fmt(epd, f),
            Err(e) => write!(f, "❌ {e}"),
        }
    }
}

async fn extract_seventv(
    channel_id: &str,
    pool: &PgPool,
) -> AnyResult<EmotePlatformData> {
    let stv_user = seven_tv::requests::get_user(channel_id).await?;
    // check if we're an editor
    if !stv_user
        .user
        .editors
        .iter()
        .any(|e| e.id == CONFIG.emotes.seven_tv.user_id)
    {
        bail!("not an editor");
    }

    let Some(ref set) = stv_user.emote_set else {
        bail!("active emote-set is null");
    };

    let (slots, swaps) = future::try_join(
        get_open_slots(channel_id, SlotPlatform::SevenTv, pool),
        get_swap_data(channel_id, SlotPlatform::SevenTv, pool),
    )
    .await?;

    Ok(EmotePlatformData {
        remaining_emotes: set.capacity.saturating_sub(set.emotes.len()),
        open_slots: slots,
        swap_capacity: swaps.1.unwrap_or(set.capacity).saturating_sub(swaps.0),
    })
}

async fn extract_ffz(
    channel_id: &str,
    channel_login: &str,
    pool: &PgPool,
) -> AnyResult<EmotePlatformData> {
    if !ffz::is_editor_in(channel_login).await {
        bail!("not an editor");
    }

    let (slots, swaps, user, room) = future::try_join4(
        get_open_slots(channel_id, SlotPlatform::Ffz, pool),
        get_swap_data(channel_id, SlotPlatform::Ffz, pool),
        ffz::requests::get_user(channel_id),
        ffz::requests::get_room(channel_id),
    )
    .await?;
    let added_emotes: usize =
        room.sets.values().map(|s| s.emoticons.len()).sum();
    Ok(EmotePlatformData {
        remaining_emotes: user.max_emoticons.saturating_sub(added_emotes),
        open_slots: slots,
        swap_capacity: swaps
            .1
            .unwrap_or(user.max_emoticons)
            .saturating_sub(swaps.0),
    })
}

async fn extract_bttv(
    channel_id: &str,
    pool: &PgPool,
) -> AnyResult<EmotePlatformData> {
    let bttv_id = bttv::get_or_fetch_id(channel_id, pool).await?;
    let limits = match bttv::get_user_limits(&bttv_id).await {
        Ok(l) => l,
        // user isn't an editor
        Err(_) => bail!("not an editor"),
    };

    let (slots, swaps, user) = future::try_join3(
        get_open_slots(channel_id, SlotPlatform::Bttv, pool),
        get_swap_data(channel_id, SlotPlatform::Bttv, pool),
        bttv::requests::get_user(&bttv_id),
    )
    .await?;

    Ok(EmotePlatformData {
        remaining_emotes: limits
            .channel_emotes
            .saturating_sub(user.shared_emotes.len()),
        open_slots: slots,
        swap_capacity: swaps
            .1
            .unwrap_or(limits.channel_emotes)
            .saturating_sub(swaps.0),
    })
}

async fn get_open_slots(
    channel_id: &str,
    platform: SlotPlatform,
    pool: &PgPool,
) -> AnyResult<usize> {
    let available = slot::Slot::get_n_available_slots_for_platform(
        channel_id, platform, pool,
    )
    .await?;
    Ok(available as usize)
}

async fn get_swap_data(
    channel_id: &str,
    platform: SlotPlatform,
    pool: &PgPool,
) -> AnyResult<(usize, Option<usize>)> {
    let (active, limit) = future::try_join(
        swap_emote::SwapEmote::emote_count(channel_id, platform, pool),
        reward::Reward::get_swap_limit_for_user(channel_id, platform, pool),
    )
    .await?;

    Ok((active as usize, limit))
}

impl From<AnyResult<EmotePlatformData>> for EpDataOpt {
    fn from(opt: AnyResult<EmotePlatformData>) -> Self {
        Self(opt)
    }
}
