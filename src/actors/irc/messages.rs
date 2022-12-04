use actix::Message;
use anyhow::Error as AnyError;
use twitch_irc::message::PrivmsgMessage;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ChatMessage(pub PrivmsgMessage);

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct PartMessage(pub String);

#[derive(Message)]
#[rtype(result = "Result<(), AnyError>")]
pub struct SayMessage(pub String, pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinAllMessage(pub Vec<String>);
