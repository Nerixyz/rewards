use crate::services::sql::SqlResult;
use actix::Message;
use twitch_irc::login::UserAccessToken;

#[derive(Message)]
#[rtype(result = "SqlResult<()>")]
pub struct SaveToken(pub UserAccessToken);

pub struct GetToken;

impl Message for GetToken {
    type Result = SqlResult<UserAccessToken>;
}
