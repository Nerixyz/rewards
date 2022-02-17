use crate::{
    chat::{command::ChatCommand, parse::opt_next_space},
    services::{
        emotes::{format::format_emote_url, remove::remove_emote, search::search_emote_by_name},
        formatting::human_format_duration,
        text::first_capture,
    },
    PgPool, RedisConn,
};
use anyhow::{anyhow, Error as AnyError, Result as AnyResult};
use async_trait::async_trait;
use chrono::Utc;
use deadpool_redis::redis::AsyncCommands;
use either::Either;
use lazy_static::lazy_static;
use models::{
    banned_emote, editor::Editor, emote::SlotPlatform, slot::Slot, swap_emote::SwapEmote,
};
use regex::Regex;
use std::borrow::Cow;
use twitch_irc::message::PrivmsgMessage;

pub enum EmoteManagement {
    Info(String),
    Ban(String),
    Unban(String),
}

async fn execute_info(
    msg: &PrivmsgMessage,
    emote: &str,
    pool: &PgPool,
    redis: &mut RedisConn,
) -> AnyResult<String> {
    match search_emote_by_name(emote, &msg.channel_id, pool).await? {
        Some(Either::Left(slot)) => format_slot(&msg.sender.login, slot),
        Some(Either::Right(swap)) => format_swap(&msg.sender.login, swap),
        None => {
            if let Some(slot) = redis
                .get::<_, String>(format!(
                    "rewards:exp-slots:{}:{}",
                    msg.channel_id,
                    emote.to_lowercase()
                ))
                .await
                .ok()
                .and_then(|s| serde_json::from_str::<Slot>(&s).ok())
            {
                format_slot(&msg.sender.login, slot)
            } else {
                Err(AnyError::msg("This emote is unknown to me."))
            }
        }
    }
}

async fn execute_ban(msg: &PrivmsgMessage, emote: &str, pool: &PgPool) -> AnyResult<String> {
    let (emote_id, platform) = extract_emote_data(emote, &msg.channel_id, pool)
        .await
        .ok_or_else(|| anyhow!("Could not find emote. Try to specify the emote url!"))?;
    banned_emote::add(&msg.channel_id, &emote_id, platform, pool)
        .await
        .map_err(|_| anyhow!("Couldn't add ban, the emote might be banned already"))?;
    // .ok because it may not be added
    remove_emote(&msg.channel_id, &emote_id, platform, pool)
        .await
        .ok();
    Ok(format!(
        "@{}, 🚫 Banned {}",
        msg.sender.login,
        format_emote_url(platform, &emote_id)
    ))
}

async fn execute_unban(msg: &PrivmsgMessage, emote: &str, pool: &PgPool) -> AnyResult<String> {
    let (emote_id, platform) = extract_emote_data(emote, &msg.channel_id, pool)
        .await
        .ok_or_else(|| anyhow!("Could not find emote. Try to specify the emote url!"))?;
    banned_emote::remove(&msg.channel_id, &emote_id, platform, pool).await?;
    Ok(format!(
        "@{}, ✅ Unbanned {}",
        msg.sender.login,
        format_emote_url(platform, &emote_id)
    ))
}

#[async_trait]
impl ChatCommand for EmoteManagement {
    async fn execute(
        &mut self,
        msg: PrivmsgMessage,
        pool: &PgPool,
        redis: &mut RedisConn,
    ) -> AnyResult<String> {
        match &self {
            Self::Info(emote) => execute_info(&msg, emote, pool, redis).await,
            Self::Ban(emote) => execute_ban(&msg, emote, pool).await,
            Self::Unban(emote) => execute_unban(&msg, emote, pool).await,
        }
    }

    fn parse(cmd: &str, args: Option<&str>) -> AnyResult<Box<dyn ChatCommand + Send>>
    where
        Self: Sized + Send,
    {
        Ok(Box::new(match cmd {
            "ei" | "emoteinfo" => Self::Info(
                args.ok_or_else(|| anyhow!("No emote specified"))
                    .map(opt_next_space)?
                    .0
                    .to_string(),
            ),
            _ => {
                let (target, args) = args
                    .ok_or_else(|| anyhow!("No option specified (emote <ban/unban/info/{{emote}}>"))
                    .map(opt_next_space)?;
                let target = target.to_lowercase();
                match target.as_str() {
                    "ban" | "unban" => {
                        let emote = args
                            .ok_or_else(|| anyhow!("No emote url specified"))
                            .map(opt_next_space)?
                            .0
                            .to_string();
                        if target == "ban" {
                            Self::Ban(emote)
                        } else {
                            Self::Unban(emote)
                        }
                    }
                    _ => {
                        let emote = if target == "info" {
                            args.ok_or_else(|| anyhow!("No emote specified"))
                                .map(opt_next_space)?
                                .0
                                .to_string()
                        } else {
                            target
                        };
                        Self::Info(emote)
                    }
                }
            }
        }))
    }

    async fn check_permission(
        &mut self,
        msg: &PrivmsgMessage,
        pool: &PgPool,
        _redis: &mut RedisConn,
    ) -> bool {
        if matches!(self, Self::Info(_)) || msg.sender.id == msg.channel_id {
            true
        } else {
            Editor::get_broadcaster_for_editor(&msg.sender.id, &msg.channel_id, pool)
                .await
                .is_ok()
        }
    }
}

lazy_static! {
    static ref BTTV_REGEX: Regex =
        Regex::new("(?:https?://)?betterttv\\.com/emotes/([a-f0-9]{24})").expect("must compile");
    static ref FFZ_REGEX: Regex =
        Regex::new("(?:https?://)?(?:www\\.)?frankerfacez\\.com/emoticon/(\\d+)")
            .expect("must compile");
    static ref SEVENTV_REGEX: Regex =
        Regex::new("(?:https?://)?7tv\\.app/emotes/([a-f0-9]{24})").expect("must compile");
}

async fn extract_emote_data<'a>(
    emote: &'a str,
    channel_id: &str,
    pool: &PgPool,
) -> Option<(Cow<'a, str>, SlotPlatform)> {
    Some(if let Some(id) = first_capture(emote, &BTTV_REGEX) {
        (id.into(), SlotPlatform::Bttv)
    } else if let Some(id) = first_capture(emote, &FFZ_REGEX) {
        (id.into(), SlotPlatform::Ffz)
    } else if let Some(id) = first_capture(emote, &SEVENTV_REGEX) {
        (id.into(), SlotPlatform::SevenTv)
    } else {
        match search_emote_by_name(emote, channel_id, pool).await.ok()?? {
            Either::Left(slot) => (slot.emote_id?.into(), slot.platform),
            Either::Right(swap) => (swap.emote_id.into(), swap.platform),
        }
    })
}

fn format_slot(sender: &str, slot: Slot) -> AnyResult<String> {
    match (slot.name, slot.added_at, slot.added_by, slot.expires) {
        (Some(name), Some(added_at), Some(added_by), Some(expired)) => {
            let now = Utc::now();
            let added_duration = now - added_at;
            let expired_duration = now - expired;
            let in_past = now > expired;
            Ok(format!(
                "@{}, {} was added {} by @{} and {} removed {}",
                sender,
                name,
                human_format_duration(&added_duration),
                added_by,
                if in_past { "was" } else { "will be" },
                human_format_duration(&expired_duration)
            ))
        }
        _ => Err(AnyError::msg("Not enough information")),
    }
}

fn format_swap(sender: &str, emote: SwapEmote) -> AnyResult<String> {
    let now = Utc::now();
    let added_duration = now - emote.added_at;
    Ok(format!(
        "@{}, {} was added {} by @{} [{}]",
        sender,
        emote.name,
        human_format_duration(&added_duration),
        emote.added_by,
        emote.platform
    ))
}
