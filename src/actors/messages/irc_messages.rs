use actix::Message;
use anyhow::Error as AnyError;
use derive_more::Display;
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

#[derive(Message)]
#[rtype(result = "Result<(), AnyError>")]
pub struct TimeoutMessage {
    pub broadcaster: String,
    pub user: String,
    pub duration: u64,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct TimedModeMessage {
    pub mode: TimedMode,
    pub broadcaster: String,
    pub duration: u64,
}

#[derive(Display)]
pub enum TimedMode {
    #[display(fmt = "emoteonly")]
    Emote,
    #[display(fmt = "subscribers")]
    Sub,
}
