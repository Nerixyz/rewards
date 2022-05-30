use crate::{
    services::{emotes::search::search_emote_by_name, text::first_capture},
    PgPool,
};
use either::Either;
use lazy_static::lazy_static;
use models::emote::SlotPlatform;
use regex::Regex;
use std::borrow::Cow;

lazy_static! {
    static ref BTTV_REGEX: Regex =
        Regex::new("(?:https?://)?betterttv\\.com/emotes/([a-f0-9]{24})").expect("must compile");
    static ref FFZ_REGEX: Regex =
        Regex::new("(?:https?://)?(?:www\\.)?frankerfacez\\.com/emoticon/(\\d+)")
            .expect("must compile");
    static ref SEVENTV_REGEX: Regex =
        Regex::new("(?:https?://)?7tv\\.app/emotes/([a-f0-9]{24})").expect("must compile");
}

pub async fn extract_emote_data<'a>(
    emote: &'a str,
    channel_id: &str,
    pool: &PgPool,
) -> Option<(Cow<'a, str>, SlotPlatform)> {
    Some(if let Some(data) = extract_emote_by_url(emote) {
        data
    } else {
        match search_emote_by_name(emote, channel_id, pool).await.ok()?? {
            Either::Left(slot) => (slot.emote_id?.into(), slot.platform),
            Either::Right(swap) => (swap.emote_id.into(), swap.platform),
        }
    })
}

pub fn extract_emote_by_url(emote: &str) -> Option<(Cow<str>, SlotPlatform)> {
    if let Some(id) = first_capture(emote, &BTTV_REGEX) {
        Some((id.into(), SlotPlatform::Bttv))
    } else if let Some(id) = first_capture(emote, &FFZ_REGEX) {
        Some((id.into(), SlotPlatform::Ffz))
    } else if let Some(id) = first_capture(emote, &SEVENTV_REGEX) {
        Some((id.into(), SlotPlatform::SevenTv))
    } else {
        None
    }
}
