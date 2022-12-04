use crate::{chat::command::ChatCommand, AppAccessToken, RedisConn};
use anyhow::Result as AnyResult;
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use twitch_irc::message::PrivmsgMessage;

pub struct Ping;

#[async_trait]
impl ChatCommand for Ping {
    async fn execute(
        &mut self,
        msg: PrivmsgMessage,
        _pool: &PgPool,
        _: &mut RedisConn,
        _: Arc<RwLock<AppAccessToken>>,
    ) -> AnyResult<String> {
        Ok(format!("@{}, 🤖 Pong! ⏱ Uptime: {uptime} 📜 Git: {git_info} 🛠 Compiled with Rust {rustc_info} on 🖥 {build_info} 📦 {build_profile}",
                   msg.sender.login,
                   uptime = humantime::format_duration(uptimer::get().unwrap_or_default()),
                   git_info = env!("RW_GIT_INFO"),
                   rustc_info = env!("RW_RUSTC_INFO"),
                   build_info = env!("RW_BUILD_INFO"),
                   build_profile = env!("RW_BUILD_PROFILE")
        ))
    }

    fn parse(
        _cmd: &str,
        _msg: Option<&str>,
    ) -> AnyResult<Box<dyn ChatCommand + Send>> {
        Ok(Box::new(Self))
    }
}
