use models::emote::SlotPlatform;

pub fn format_emote_url(platform: SlotPlatform, emote_id: &str) -> String {
    match platform {
        SlotPlatform::Bttv => format!("https://betterttv.com/emotes/{}", emote_id),
        SlotPlatform::Ffz => format!("https://www.frankerfacez.com/emoticon/{}", emote_id),
        SlotPlatform::SevenTv => format!("https://7tv.app/emotes/{}", emote_id),
    }
}
