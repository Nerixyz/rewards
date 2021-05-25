use crate::constants::{EVENTSUB_BASE64_SECRET, SERVER_URL};
use crate::services::twitch::errors::to_response_error;
use actix_web::{error, Error};
use twitch_api2::eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1;
use twitch_api2::eventsub::{Transport, TransportMethod};
use twitch_api2::helix::eventsub::{
    CreateEventSubSubscription, CreateEventSubSubscriptionBody, CreateEventSubSubscriptionRequest,
    DeleteEventSubSubscription, DeleteEventSubSubscriptionRequest,
};
use twitch_api2::helix::points::{
    CustomRewardRedemptionStatus, UpdateRedemptionStatusBody, UpdateRedemptionStatusInformation,
    UpdateRedemptionStatusRequest,
};
use twitch_api2::helix::Response;
use twitch_api2::twitch_oauth2::{AppAccessToken, UserToken};
use twitch_api2::HelixClient;

pub async fn delete_subscription(token: &AppAccessToken, id: String) -> Result<(), Error> {
    let response: DeleteEventSubSubscription = HelixClient::<'_, reqwest::Client>::default()
        .req_delete(
            DeleteEventSubSubscriptionRequest::builder().id(id).build(),
            token,
        )
        .await
        .map_err(to_response_error)?;

    match response {
        DeleteEventSubSubscription::Success => Ok(()),
        _ => Err(error::ErrorInternalServerError("")),
    }
}

pub async fn subscribe_to_rewards(
    token: &AppAccessToken,
    id: &str,
) -> Result<CreateEventSubSubscription<ChannelPointsCustomRewardRedemptionAddV1>, Error> {
    let response: Response<
        CreateEventSubSubscriptionRequest<ChannelPointsCustomRewardRedemptionAddV1>,
        CreateEventSubSubscription<ChannelPointsCustomRewardRedemptionAddV1>,
    > = HelixClient::<'_, reqwest::Client>::default()
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
        .await
        .map_err(to_response_error)?;

    Ok(response.data)
}

pub async fn update_reward_redemption(
    broadcaster_id: &str,
    reward_id: &str,
    redemption_id: &str,
    status: CustomRewardRedemptionStatus,
    token: &UserToken,
) -> Result<(), Error> {
    let response: Response<UpdateRedemptionStatusRequest, UpdateRedemptionStatusInformation> =
        HelixClient::<'_, reqwest::Client>::default()
            .req_patch(
                UpdateRedemptionStatusRequest::builder()
                    .broadcaster_id(broadcaster_id)
                    .reward_id(reward_id)
                    .id(redemption_id)
                    .build(),
                UpdateRedemptionStatusBody::builder().status(status).build(),
                token,
            )
            .await
            .map_err(to_response_error)?;

    match response.data {
        UpdateRedemptionStatusInformation::Success(_) => Ok(()),
        _ => Err(error::ErrorInternalServerError("")),
    }
}
