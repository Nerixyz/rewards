use crate::{
    chat::{command::ChatCommand, parse::opt_next_space},
    AppAccessToken, RedisConn,
};
use anyhow::Result as AnyResult;
use async_trait::async_trait;
use futures::future;
use itertools::Itertools;
use models::{emote::SlotPlatform, slot::Slot, swap_emote::SwapEmote};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use twitch_irc::message::PrivmsgMessage;

enum Requested {
    Slots,
    Bttv,
    Ffz,
    SevenTv,
}

pub struct Emotes {
    requested: Option<Requested>,
}

#[async_trait]
impl ChatCommand for Emotes {
    async fn execute(
        &mut self,
        msg: PrivmsgMessage,
        pool: &PgPool,
        _: &mut RedisConn,
        _: Arc<RwLock<AppAccessToken>>,
    ) -> AnyResult<String> {
        let resp = match self.requested {
            None => future::try_join(
                Slot::get_occupied_emotes(&msg.channel_id, pool),
                SwapEmote::all_emote_names(&msg.channel_id, pool),
            )
            .await
            .map(|(mut slots, mut swap)| {
                slots.append(&mut swap);
                slots
            }),
            Some(Requested::Slots) => Slot::get_occupied_emotes(&msg.channel_id, pool).await,
            Some(Requested::Bttv) => {
                SwapEmote::platform_emote_names(&msg.channel_id, SlotPlatform::Bttv, pool).await
            }
            Some(Requested::Ffz) => {
                SwapEmote::platform_emote_names(&msg.channel_id, SlotPlatform::Ffz, pool).await
            }
            Some(Requested::SevenTv) => {
                SwapEmote::platform_emote_names(&msg.channel_id, SlotPlatform::SevenTv, pool).await
            }
        }?;
        Ok(if resp.is_empty() {
            format!(
                "@{}, no managed emotes found for this channel",
                msg.sender.login
            )
        } else {
            format!("@{}, {}", msg.sender.login, resp.iter().join(" "))
        })
    }

    fn parse(_cmd: &str, args: Option<&str>) -> AnyResult<Box<dyn ChatCommand + Send>>
    where
        Self: Sized + Send,
    {
        let requested =
            args.map(opt_next_space)
                .and_then(|(arg, _)| match arg.to_lowercase().as_str() {
                    "slot" | "slots" => Some(Requested::Slots),
                    "bttv" | "betterttv" => Some(Requested::Bttv),
                    "ffz" | "frankerfacez" => Some(Requested::Ffz),
                    "seventv" | "7tv" => Some(Requested::SevenTv),
                    _ => None,
                });
        Ok(Box::new(Self { requested }))
    }
}
