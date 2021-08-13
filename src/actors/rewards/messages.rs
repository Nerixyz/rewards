use crate::models::{reward::Reward, user::User};
use actix::Message;
use twitch_api2::eventsub::{
    channel::ChannelPointsCustomRewardRedemptionAddV1, NotificationPayload,
};

pub struct ExecuteRewardMessage {
    pub reward: Reward,
    pub redemption: NotificationPayload<ChannelPointsCustomRewardRedemptionAddV1>,
    pub broadcaster: User,
}

impl Message for ExecuteRewardMessage {
    type Result = anyhow::Result<()>;
}
