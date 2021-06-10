use actix::Recipient;
use crate::actors::messages::irc_messages::SayMessage;
use twitch_irc::message::PrivmsgMessage;
use crate::chat::command::ChatCommand;
use actix::Message;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ExecuteCommandMessage {
    pub addr: Recipient<SayMessage>,
    pub raw: PrivmsgMessage,
    pub executor: Box<dyn ChatCommand + Send>,
}