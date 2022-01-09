use twitch_api2::eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1Payload;

pub mod execute;
pub mod extract;
pub mod redemption;
pub mod reply;
pub mod save;
pub mod verify;

pub type Redemption = ChannelPointsCustomRewardRedemptionAddV1Payload;
