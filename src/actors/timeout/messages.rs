use actix::Message;
use std::time::Duration;

#[derive(Message)]
#[rtype("()")]
pub struct ChannelTimeoutMessage {
    pub channel_id: String,
    pub user_id: String,
    pub duration: Duration,
}

#[derive(Message)]
#[rtype("()")]
pub struct RemoveTimeoutMessage {
    pub channel_id: String,
    pub user_id: String,
    pub later: Duration,
}

pub struct CheckValidTimeoutMessage {
    pub channel_id: String,
    pub user_id: String,
}

impl Message for CheckValidTimeoutMessage {
    type Result = anyhow::Result<bool>;
}
