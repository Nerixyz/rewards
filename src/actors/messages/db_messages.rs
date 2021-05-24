use twitch_irc::login::UserAccessToken;
use actix::{Message};
use crate::services::sql::SqlError;

#[derive(Message)]
#[rtype(result = "Result<(), SqlError>")]
pub struct SaveToken(pub UserAccessToken);

pub struct GetToken;

impl Message for GetToken {
    type Result = Result<UserAccessToken, SqlError>;
}