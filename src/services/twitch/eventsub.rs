use crate::{
    config::CONFIG,
    services::twitch::{errors::TwitchApiError, HelixResult, RHelixClient},
};
use twitch_api2::{
    eventsub::{channel::ChannelPointsCustomRewardRedemptionAddV1, Transport, TransportMethod},
    helix::{
        eventsub::{
            CreateEventSubSubscription, CreateEventSubSubscriptionBody,
            CreateEventSubSubscriptionRequest, DeleteEventSubSubscriptionRequest,
        },
        points::{
            CustomRewardRedemption, CustomRewardRedemptionStatus, UpdateRedemptionStatusBody,
            UpdateRedemptionStatusInformation, UpdateRedemptionStatusRequest,
        },
        Response,
    },
    twitch_oauth2::{AppAccessToken, UserToken},
    types::EventSubId,
};

pub async fn delete_subscription<I: Into<EventSubId>>(
    token: &AppAccessToken,
    id: I,
) -> HelixResult<()> {
    RHelixClient::default()
        .req_delete(
            DeleteEventSubSubscriptionRequest::builder()
                .id(id.into())
                .build(),
            token,
        )
        .await?;

    Ok(())
}

pub async fn subscribe_to_rewards(
    token: &AppAccessToken,
    id: &str,
) -> HelixResult<CreateEventSubSubscription<ChannelPointsCustomRewardRedemptionAddV1>> {
    let response: Response<
        CreateEventSubSubscriptionRequest<ChannelPointsCustomRewardRedemptionAddV1>,
        CreateEventSubSubscription<ChannelPointsCustomRewardRedemptionAddV1>,
    > = RHelixClient::default()
        .req_post(
            CreateEventSubSubscriptionRequest::builder().build(),
            CreateEventSubSubscriptionBody::builder()
                .transport(
                    Transport::builder()
                        .method(TransportMethod::Webhook)
                        .secret(CONFIG.twitch.eventsub.secret.to_string())
                        .callback(format!("{}/api/v1/eventsub/reward", CONFIG.server.url))
                        .build(),
                )
                .subscription(
                    ChannelPointsCustomRewardRedemptionAddV1::builder()
                        .broadcaster_user_id(id)
                        .build(),
                )
                .build(),
            token,
        )
        .await?;

    Ok(response.data)
}

pub async fn update_reward_redemption(
    broadcaster_id: &str,
    reward_id: &str,
    redemption_id: &str,
    status: CustomRewardRedemptionStatus,
    token: &UserToken,
) -> HelixResult<CustomRewardRedemption> {
    let response: Response<UpdateRedemptionStatusRequest, UpdateRedemptionStatusInformation> =
        RHelixClient::default()
            .req_patch(
                UpdateRedemptionStatusRequest::builder()
                    .broadcaster_id(broadcaster_id)
                    .reward_id(reward_id)
                    .id(redemption_id)
                    .build(),
                UpdateRedemptionStatusBody::builder().status(status).build(),
                token,
            )
            .await?;

    match response.data {
        UpdateRedemptionStatusInformation::Success(r) => Ok(r),
        _ => Err(TwitchApiError::Custom),
    }
}
