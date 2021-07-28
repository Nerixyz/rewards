use actix::Message;

#[derive(Message)]
#[rtype(result = "()")]
pub struct LiveMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct OfflineMessage(pub String);
