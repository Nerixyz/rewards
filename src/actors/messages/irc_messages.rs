use twitch_irc::message::PrivmsgMessage;
use actix::Message;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ChatMessage(pub PrivmsgMessage);

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinAllMessage(pub Vec<String>);