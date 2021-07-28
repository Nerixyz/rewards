use actix::Message;

#[derive(Message)]
#[rtype(result = "()")]
/// This is already the listen command at the auth token is needed
pub struct SubMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
/// This is already the listen command at the auth token is needed
pub struct SubAllMessage(pub Vec<String>);
