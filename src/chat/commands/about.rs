use crate::{chat::command::ChatCommand, RedisConn};
use anyhow::Result as AnyResult;
use async_trait::async_trait;
use sqlx::PgPool;
use twitch_irc::message::PrivmsgMessage;

pub struct About;

#[async_trait]
impl ChatCommand for About {
    async fn execute(
        &mut self,
        msg: PrivmsgMessage,
        _pool: &PgPool,
        _: &mut RedisConn,
    ) -> AnyResult<String> {
        Ok(format!("@{}, I'm a bot made for rewards.nerixyz.de by @nerixyz in Rust {rustc_info} ({build_profile}) 📝 github.com/Nerixyz/rewards",
                   msg.sender.login,
                   rustc_info = env!("RW_RUSTC_INFO"),
                   build_profile = env!("RW_BUILD_PROFILE")
        ))
    }

    fn parse(_msg: Option<&str>) -> AnyResult<Box<dyn ChatCommand + Send>> {
        Ok(Box::new(Self))
    }
}
