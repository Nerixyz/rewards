use actix::Message;
use anyhow::Error as AnyError;
use std::time::Duration;
use twitch_irc::message::PrivmsgMessage;
use derive_more::{Display};

#[derive(Message)]
#[rtype(result = "()")]
pub struct ChatMessage(pub PrivmsgMessage);

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinAllMessage(pub Vec<String>);

#[derive(Message)]
#[rtype(result = "Result<(), AnyError>")]
pub struct TimeoutMessage {
    pub broadcaster: String,
    pub user: String,
    pub duration: Duration,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct TimedModeMessage {
    pub mode: TimedMode,
    pub broadcaster: String,
    pub duration: Duration,
}

#[derive(Display)]
pub enum TimedMode {
    #[display(fmt = "emoteonly")]
    Emote,
    #[display(fmt = "subscribers")]
    Sub,
}
