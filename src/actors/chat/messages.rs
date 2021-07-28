use actix::{Message, Recipient};
use twitch_irc::message::PrivmsgMessage;

use crate::{actors::irc::SayMessage, chat::command::ChatCommand};

#[derive(Message)]
#[rtype(result = "()")]
pub struct ExecuteCommandMessage {
    pub addr: Recipient<SayMessage>,
    pub raw: PrivmsgMessage,
    pub executor: Box<dyn ChatCommand + Send>,
}
