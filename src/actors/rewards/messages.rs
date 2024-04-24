use actix::Message;
use models::{reward::Reward, user::User};
use twitch_api::eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1Payload;

pub struct ExecuteRewardMessage {
    pub reward: Reward,
    pub redemption: ChannelPointsCustomRewardRedemptionAddV1Payload,
    pub broadcaster: User,
}

impl Message for ExecuteRewardMessage {
    type Result = anyhow::Result<()>;
}
