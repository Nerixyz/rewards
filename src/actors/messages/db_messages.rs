use crate::services::sql::SqlError;
use actix::Message;
use twitch_irc::login::UserAccessToken;

#[derive(Message)]
#[rtype(result = "Result<(), SqlError>")]
pub struct SaveToken(pub UserAccessToken);

pub struct GetToken;

impl Message for GetToken {
    type Result = Result<UserAccessToken, SqlError>;
}
