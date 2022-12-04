use crate::services::twitch::{
    errors::TwitchApiError, HelixResult, RHelixClient,
};
use config::CONFIG;
use twitch_api2::{
    eventsub::{
        channel::ChannelPointsCustomRewardRedemptionAddV1, Transport,
        TransportMethod,
    },
    helix::{
        eventsub::{
            CreateEventSubSubscription, CreateEventSubSubscriptionBody,
            CreateEventSubSubscriptionRequest,
            DeleteEventSubSubscriptionRequest,
        },
        points::{
            CustomRewardRedemption, CustomRewardRedemptionStatus,
            UpdateRedemptionStatusBody, UpdateRedemptionStatusInformation,
            UpdateRedemptionStatusRequest,
        },
        Response,
    },
    twitch_oauth2::{AppAccessToken, UserToken},
};
use twitch_api2::types::{EventSubIdRef, IntoCow, RedemptionIdRef, RewardIdRef, UserId, UserIdRef};

pub async fn delete_subscription<'a>(
    token: &AppAccessToken,
    id: impl IntoCow<'a, EventSubIdRef> + 'a,
) -> HelixResult<()> {
    RHelixClient::default()
        .req_delete(
            DeleteEventSubSubscriptionRequest::id(id),
            token,
        )
        .await?;

    Ok(())
}

pub async fn subscribe_to_rewards(
    token: &AppAccessToken,
    id: impl Into<UserId>,
) -> HelixResult<
    CreateEventSubSubscription<ChannelPointsCustomRewardRedemptionAddV1>,
> {
    let response: Response<
        CreateEventSubSubscriptionRequest<
            ChannelPointsCustomRewardRedemptionAddV1,
        >,
        CreateEventSubSubscription<ChannelPointsCustomRewardRedemptionAddV1>,
    > = RHelixClient::default()
        .req_post(
            CreateEventSubSubscriptionRequest::builder().build(),
            CreateEventSubSubscriptionBody::builder()
                .transport(
                    Transport::builder()
                        .method(TransportMethod::Webhook)
                        .secret(CONFIG.twitch.eventsub.secret.to_string())
                        .callback(format!(
                            "{}/api/v1/eventsub/reward",
                            CONFIG.server.url
                        ))
                        .build(),
                )
                .subscription(
                    ChannelPointsCustomRewardRedemptionAddV1::broadcaster_user_id(id),
                )
                .build(),
            token,
        )
        .await?;

    Ok(response.data)
}

pub async fn update_reward_redemption<'a>(
    broadcaster_id: impl IntoCow<'a, UserIdRef> + 'a,
    reward_id: impl IntoCow<'a, RewardIdRef> + 'a,
    redemption_id: impl IntoCow<'a, RedemptionIdRef> + 'a,
    status: CustomRewardRedemptionStatus,
    token: &UserToken,
) -> HelixResult<CustomRewardRedemption> {
    let response: Response<
        UpdateRedemptionStatusRequest,
        UpdateRedemptionStatusInformation,
    > = RHelixClient::default()
        .req_patch(
            UpdateRedemptionStatusRequest::new(broadcaster_id, reward_id, redemption_id),
            UpdateRedemptionStatusBody::builder().status(status).build(),
            token,
        )
        .await?;

    match response.data {
        UpdateRedemptionStatusInformation::Success(r) => Ok(r),
        _ => Err(TwitchApiError::Custom),
    }
}
