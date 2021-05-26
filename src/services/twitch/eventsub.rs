use crate::constants::{EVENTSUB_BASE64_SECRET, SERVER_URL};
use crate::services::twitch::{HelixResult, RHelixClient};
use twitch_api2::{
    eventsub::{channel::ChannelPointsCustomRewardRedemptionAddV1, Transport, TransportMethod},
    helix::{
        eventsub::{
            CreateEventSubSubscription, CreateEventSubSubscriptionBody,
            CreateEventSubSubscriptionRequest, DeleteEventSubSubscriptionRequest,
        },
        points::{
            CustomRewardRedemptionStatus, UpdateRedemptionStatusBody,
            UpdateRedemptionStatusInformation, UpdateRedemptionStatusRequest,
        },
        Response,
    },
    twitch_oauth2::{AppAccessToken, UserToken},
};

pub async fn delete_subscription(token: &AppAccessToken, id: String) -> HelixResult<()> {
    RHelixClient::default()
        .req_delete(
            DeleteEventSubSubscriptionRequest::builder().id(id).build(),
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
                        .secret(EVENTSUB_BASE64_SECRET.to_string())
                        .callback(format!("{}/api/v1/eventsub/reward", SERVER_URL))
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
) -> HelixResult<()> {
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
        UpdateRedemptionStatusInformation::Success(_) => Ok(()),
        _ => Err("".into()),
    }
}
