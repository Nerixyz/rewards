use crate::actors::messages::irc_messages::SayMessage;
use crate::chat::command::ChatCommand;
use actix::Message;
use actix::Recipient;
use twitch_irc::message::PrivmsgMessage;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ExecuteCommandMessage {
    pub addr: Recipient<SayMessage>,
    pub raw: PrivmsgMessage,
    pub executor: Box<dyn ChatCommand + Send>,
}
