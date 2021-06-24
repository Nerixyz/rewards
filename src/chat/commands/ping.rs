use crate::chat::command::ChatCommand;
use anyhow::Result as AnyResult;
use async_trait::async_trait;
use sqlx::PgPool;
use twitch_irc::message::PrivmsgMessage;

pub struct Ping;

#[async_trait]
impl ChatCommand for Ping {
    async fn execute(&mut self, msg: PrivmsgMessage, _pool: &PgPool) -> AnyResult<String> {
        Ok(format!("@{}, Pong! Uptime: {uptime} Git: {git_info} Compiled with Rust {rustc_info} on {build_info}",
                   msg.sender.login,
                   uptime = humantime::format_duration(uptimer::get_async().await.unwrap_or_default()),
                   git_info = env!("RW_GIT_INFO"),
                   rustc_info = env!("RW_RUSTC_INFO"),
                   build_info = env!("RW_BUILD_INFO")
        ))
    }

    fn parse(_msg: Option<&str>) -> AnyResult<Box<dyn ChatCommand + Send>> {
        Ok(Box::new(Self))
    }
}
