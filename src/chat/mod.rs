pub mod command;
pub mod commands;
pub mod parse;

use crate::chat::command::ChatCommand;
use anyhow::Result as AnyResult;
use commands::{emote_info::EmoteInfo, ping::Ping, slots::SlotsCommand};

pub fn try_parse_command(
    command: &str,
    args: Option<&str>,
) -> Option<AnyResult<Box<dyn ChatCommand + Send>>> {
    let cmd = match command.to_lowercase().as_str() {
        "ping" | "bing" => Ping::parse(args),
        "emote" | "emoteinfo" | "ei" => EmoteInfo::parse(args),
        "slots" | "emoteslots" => SlotsCommand::parse(args),
        _ => return None,
    };

    Some(cmd)
}
