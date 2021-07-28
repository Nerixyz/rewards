use actix::Message;
use errors::sql::SqlResult;
use twitch_irc::login::UserAccessToken;

#[derive(Message)]
#[rtype(result = "SqlResult<()>")]
pub struct SaveToken(pub UserAccessToken);

pub struct GetToken;

impl Message for GetToken {
    type Result = SqlResult<UserAccessToken>;
}
