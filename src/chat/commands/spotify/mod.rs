mod info;
mod skip;

use crate::{
    chat::{command::ChatCommand, parse::opt_next_space},
    AppAccessToken, PgPool, RedisConn, RedisPool,
};
use anyhow::{anyhow, Result as AnyResult};
use async_trait::async_trait;
use models::editor::Editor;
use std::sync::Arc;
use tokio::sync::RwLock;
use twitch_irc::message::PrivmsgMessage;

pub enum SpotifyAction {
    Skip(Option<String>),
    Info,
}

#[async_trait]
impl ChatCommand for SpotifyAction {
    async fn execute(
        &mut self,
        msg: PrivmsgMessage,
        pool: &PgPool,
        _redis: RedisPool,
        _: Arc<RwLock<AppAccessToken>>,
    ) -> AnyResult<String> {
        match &self {
            SpotifyAction::Skip(Some(_)) => {
                Err(anyhow!("Removing a specific song from the queue is not supported because of missing functionality in Spotify's API."))
            },
            SpotifyAction::Skip(None) => skip::execute(msg, pool).await,
            SpotifyAction::Info => info::execute(msg, pool).await,
        }
    }

    fn parse(
        _cmd: &str,
        args: Option<&str>,
    ) -> AnyResult<Box<dyn ChatCommand + Send>>
    where
        Self: Sized + Send,
    {
        const OPTIONS: &str = "spotify <(i)nfo/(s)kip>";

        let (target, args) = args
            .ok_or_else(|| anyhow!("No option specified ({OPTIONS})"))
            .map(opt_next_space)?;
        let target = target.to_lowercase();
        let cmd = match target.as_str() {
            "i" | "info" => Self::Info,
            "s" | "skip" => Self::Skip(args.map(|a| a.to_owned())),
            _ => return Err(anyhow!("Unknown subcommand ({OPTIONS})")),
        };
        Ok(Box::new(cmd))
    }

    async fn check_permission(
        &mut self,
        msg: &PrivmsgMessage,
        pool: &PgPool,
        _redis: &mut RedisConn,
    ) -> bool {
        if matches!(self, Self::Info) || msg.sender.id == msg.channel_id {
            true
        } else {
            Editor::get_broadcaster_for_editor(
                &msg.sender.id,
                &msg.channel_id,
                pool,
            )
            .await
            .is_ok()
        }
    }
}
