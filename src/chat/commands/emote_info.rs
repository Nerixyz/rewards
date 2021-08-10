use crate::{
    chat::{command::ChatCommand, parse::opt_next_space},
    models::{slot::Slot, swap_emote::SwapEmote},
    services::formatting::human_format_duration,
    RedisConn,
};
use anyhow::{Error as AnyError, Result as AnyResult};
use async_trait::async_trait;
use chrono::Utc;
use deadpool_redis::redis::AsyncCommands;
use either::Either;
use futures::TryFutureExt;
use sqlx::PgPool;
use twitch_irc::message::PrivmsgMessage;

pub struct EmoteInfo(String);

#[async_trait]
impl ChatCommand for EmoteInfo {
    async fn execute(
        &mut self,
        msg: PrivmsgMessage,
        pool: &PgPool,
        redis: &mut RedisConn,
    ) -> AnyResult<String> {
        let emote: Option<Either<Slot, SwapEmote>> =
            Slot::get_slot_by_emote_name(&msg.channel_id, &self.0, pool)
                .and_then(|opt| async {
                    match opt {
                        Some(v) => Ok(Some(Either::Left(v))),
                        None => SwapEmote::by_name(&msg.channel_id, &self.0, pool)
                            .await
                            .map(|e| e.map(Either::Right)),
                    }
                })
                .await
                .map_err(|_| AnyError::msg("Internal Error"))?;
        match emote {
            Some(Either::Left(slot)) => format_slot(&msg.sender.login, slot),
            Some(Either::Right(swap)) => format_swap(&msg.sender.login, swap),
            None => {
                if let Some(slot) = redis
                    .get::<_, String>(format!(
                        "rewards:exp-slots:{}:{}",
                        msg.channel_id,
                        self.0.to_lowercase()
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

    fn parse(args: Option<&str>) -> AnyResult<Box<dyn ChatCommand + Send>>
    where
        Self: Sized + Send,
    {
        let args = args.ok_or_else(|| AnyError::msg("No emote specified"))?;
        let (user, _) = opt_next_space(args);
        Ok(Box::new(Self(user.to_string())))
    }
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
