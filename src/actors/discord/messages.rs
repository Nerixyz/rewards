use crate::services::discord::Embed;
use actix::Message;

#[derive(Message)]
#[rtype(result = "()")]
pub struct LogToDiscordMessage {
    pub user_id: String,
    pub embed: Embed,
}
