pub mod command;
pub mod commands;
pub mod parse;

use crate::chat::commands::debug::DebugCommand;
use anyhow::Result as AnyResult;
use command::ChatCommand;
use commands::{
    about::About, emote_management::EmoteManagement, emotes::Emotes,
    ping::Ping, slots::SlotsCommand,
};

pub fn try_parse_command(
    command: &str,
    args: Option<&str>,
) -> Option<AnyResult<Box<dyn ChatCommand + Send>>> {
    let command = command.to_lowercase();
    let cmd = match command.as_str() {
        "ping" | "bing" => Ping::parse(&command, args),
        "about" | "rewardmore" | "who" | "bot" => About::parse(&command, args),
        "emote" | "emoteinfo" | "ei" => EmoteManagement::parse(&command, args),
        "slots" | "emoteslots" => SlotsCommand::parse(&command, args),
        "emotes" | "currentemotes" | "ce" => Emotes::parse(&command, args),
        "debug" | "dbg" => DebugCommand::parse(&command, args),
        _ => return None,
    };

    Some(cmd)
}
