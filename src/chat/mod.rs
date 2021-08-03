pub mod command;
pub mod commands;
pub mod parse;

use anyhow::Result as AnyResult;
use command::ChatCommand;
use commands::{
    about::About, emote_info::EmoteInfo, emotes::Emotes, ping::Ping, slots::SlotsCommand,
};

pub fn try_parse_command(
    command: &str,
    args: Option<&str>,
) -> Option<AnyResult<Box<dyn ChatCommand + Send>>> {
    let cmd = match command.to_lowercase().as_str() {
        "ping" | "bing" => Ping::parse(args),
        "about" | "rewardmore" | "who" | "bot" => About::parse(args),
        "emote" | "emoteinfo" | "ei" => EmoteInfo::parse(args),
        "slots" | "emoteslots" => SlotsCommand::parse(args),
        "emotes" | "currentemotes" | "ce" => Emotes::parse(args),
        _ => return None,
    };

    Some(cmd)
}
