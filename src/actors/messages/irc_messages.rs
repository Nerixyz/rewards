use crate::models::timed_mode::Mode;
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
#[rtype(result = "Result<(), AnyError>")]
pub struct WhisperMessage(pub String, pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinAllMessage(pub Vec<String>);

#[derive(Message)]
#[rtype(result = "Result<(), AnyError>")]
pub struct TimeoutMessage {
    pub broadcaster: String,
    pub broadcaster_id: String,

    pub user: String,
    pub user_id: String,

    pub duration: u64,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct TimedModeMessage {
    pub mode: Mode,
    pub broadcaster: String,
    pub broadcaster_id: String,
    pub duration: u64,
}
