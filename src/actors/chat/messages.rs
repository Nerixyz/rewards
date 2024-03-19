use actix::Message;
use twitch_irc::message::PrivmsgMessage;

use crate::chat::command::ChatCommand;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ExecuteCommandMessage {
    pub raw: PrivmsgMessage,
    pub executor: Box<dyn ChatCommand + Send>,
}
