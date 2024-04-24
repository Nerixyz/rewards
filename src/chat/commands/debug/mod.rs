mod channel;
mod emotes;
mod platforms;
mod sync;

use crate::{
    chat::{command::ChatCommand, parse::opt_next_space},
    services::twitch::requests::get_user_by_login,
    PgPool, RedisConn, RedisPool,
};
use anyhow::{anyhow, Error as AnyError, Result as AnyResult};
use async_trait::async_trait;
use channel::ChannelData;
use config::CONFIG;
use models::editor;
use platforms::Platforms;
use std::sync::Arc;
use tokio::sync::RwLock;
use twitch_api::twitch_oauth2::AppAccessToken;
use twitch_irc::message::PrivmsgMessage;

pub enum DebugCommand {
    Channel(Option<String>),
    Platforms,
    Editor(String),
    RmEditor(String),
    SyncRewards(Option<String>),
}

#[async_trait]
impl ChatCommand for DebugCommand {
    async fn execute(
        &mut self,
        msg: PrivmsgMessage,
        pool: &PgPool,
        _redis: RedisPool,
        app_access_token: Arc<RwLock<AppAccessToken>>,
    ) -> AnyResult<String> {
        match &mut self {
            DebugCommand::Channel(chan) => {
                let data = match chan.take() {
                    Some(username) => {
                        let (id, login) = get_user_by_login(
                            username,
                            &*app_access_token.read().await,
                        )
                        .await
                        .map(|user| (user.id.take(), user.login.take()))
                        .map_err(|e| {
                            AnyError::msg(format!(
                                "This user doesn't seem to exist: {}",
                                e
                            ))
                        })?;
                        ChannelData::get(&id, &login, pool).await?
                    }
                    None => {
                        ChannelData::get(
                            &msg.channel_id,
                            &msg.channel_login,
                            pool,
                        )
                        .await?
                    }
                };
                Ok(format!("@{}, {}", msg.sender.login, data))
            }
            DebugCommand::Platforms => {
                let platforms = Platforms::get().await?;
                Ok(format!("@{}, {}", msg.sender.login, platforms))
            }
            DebugCommand::Editor(username) => {
                editor::Editor::add_editor_for_name(
                    username,
                    &CONFIG.owner.id,
                    pool,
                )
                .await?;
                Ok(format!("@{}, done.", msg.sender.login))
            }
            DebugCommand::RmEditor(username) => {
                editor::Editor::delete_editor_for_name(
                    username,
                    &CONFIG.owner.id,
                    pool,
                )
                .await?;
                Ok(format!("@{}, done.", msg.sender.login))
            }
            DebugCommand::SyncRewards(username) => {
                let data = match username.take() {
                    Some(username) => {
                        let id = get_user_by_login(
                            username,
                            &*app_access_token.read().await,
                        )
                        .await
                        .map(|user| user.id.take())
                        .map_err(|e| {
                            AnyError::msg(format!(
                                "This user doesn't seem to exist: {}",
                                e
                            ))
                        })?;
                        sync::sync_rewards(&id, pool).await?
                    }
                    None => sync::sync_rewards(&msg.channel_id, pool).await?,
                };
                Ok(format!("@{}, removed {data} internal rewards not present on Twitch.", msg.sender.login))
            }
        }
    }

    fn parse(
        _cmd: &str,
        args: Option<&str>,
    ) -> AnyResult<Box<dyn ChatCommand + Send>>
    where
        Self: Sized + Send,
    {
        let args = match args {
            Some(a) => a,
            None => return Err(anyhow!("Expected subcommand")),
        };
        let (subcmd, cmd) = opt_next_space(args);
        match subcmd {
            "channel" => Ok(Box::new(Self::Channel(
                cmd.map(|c| opt_next_space(c).0.to_string()),
            ))),
            "platforms" | "ep" => Ok(Box::new(Self::Platforms)),
            "edit" => Ok(Box::new(Self::Editor(
                cmd.map(|c| opt_next_space(c).0.to_string())
                    .ok_or_else(|| anyhow!("expected username"))?,
            ))),
            "rmedit" | "rm-edit" | "deledit" | "del-edit" => {
                Ok(Box::new(Self::RmEditor(
                    cmd.map(|c| opt_next_space(c).0.to_string())
                        .ok_or_else(|| anyhow!("expected username"))?,
                )))
            }
            "sync" => Ok(Box::new(Self::SyncRewards(
                cmd.map(|c| opt_next_space(c).0.to_string()),
            ))),
            _ => Err(anyhow!(
                "Expected subcommand (one of 'channel', 'platforms')"
            )),
        }
    }

    async fn check_permission(
        &mut self,
        msg: &PrivmsgMessage,
        _pool: &PgPool,
        _redis: &mut RedisConn,
    ) -> bool {
        msg.sender.id == CONFIG.owner.id
    }
}
