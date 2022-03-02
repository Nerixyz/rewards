mod banning;
mod eject;
mod extract;
mod info;

use crate::{
    chat::{command::ChatCommand, parse::opt_next_space},
    PgPool, RedisConn,
};
use anyhow::{anyhow, Result as AnyResult};
use async_trait::async_trait;
use banning::{execute_ban, execute_unban};
use eject::execute_eject;
use info::execute_info;
use models::editor::Editor;
use twitch_irc::message::PrivmsgMessage;

pub enum EmoteManagement {
    Info(String),
    Ban(String),
    Unban(String),
    Eject(String),
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
            Self::Eject(emote) => execute_eject(&msg, emote, pool).await,
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
                    "ban" | "unban" | "eject" => {
                        let emote = args
                            .ok_or_else(|| anyhow!("No emote url specified"))
                            .map(opt_next_space)?
                            .0
                            .to_string();
                        match target.as_str() {
                            "ban" => Self::Ban(emote),
                            "unban" => Self::Unban(emote),
                            _ => Self::Eject(emote),
                        }
                    }
                    "info" => Self::Info(
                        args.ok_or_else(|| anyhow!("No emote specified"))
                            .map(opt_next_space)?
                            .0
                            .to_string(),
                    ),
                    _ => Self::Info(target),
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
