use actix::Message;
use twitch_api2::pubsub::video_playback::VideoPlaybackReply;

#[derive(Message)]
#[rtype(result = "()")]
/// This is already the listen command at the auth token is needed
pub struct SubMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
/// This is already the listen command at the auth token is needed
pub struct SubAllMessage(pub Vec<String>);

#[derive(Message)]
#[rtype(result = "()")]
pub struct PongMessage;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ReconnectMessage;

#[derive(Message)]
#[rtype(result = "()")]
pub struct VideoPlaybackMessage(pub String, pub VideoPlaybackReply);
