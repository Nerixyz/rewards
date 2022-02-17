use actix::Message;

#[derive(Message)]
#[rtype("()")]
pub struct Recheck;
