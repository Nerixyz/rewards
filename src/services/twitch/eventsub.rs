use crate::{
    services::twitch::{errors::TwitchApiError, HelixResult, RHelixClient},
    util::result::ResultCExt,
};
use config::CONFIG;
use twitch_api::{
    eventsub::Transport,
    helix::{
        eventsub::{
            CreateEventSubSubscription, DeleteEventSubSubscriptionRequest,
        },
        points::{
            CustomRewardRedemption, CustomRewardRedemptionStatus,
            UpdateRedemptionStatusBody, UpdateRedemptionStatusInformation,
            UpdateRedemptionStatusRequest,
        },
        Response,
    },
    twitch_oauth2::{AppAccessToken, UserToken},
    types::{EventSubIdRef, IntoCow, RedemptionIdRef, RewardIdRef, UserIdRef},
};

pub async fn delete_subscription<'a>(
    token: &AppAccessToken,
    id: impl IntoCow<'a, EventSubIdRef> + 'a,
) -> HelixResult<()> {
    RHelixClient::default()
        .req_delete(DeleteEventSubSubscriptionRequest::id(id), token)
        .await?;

    Ok(())
}

pub async fn subscribe_to<T>(
    token: &AppAccessToken,
    subscription: T,
) -> HelixResult<CreateEventSubSubscription<T>>
where
    T: twitch_api::eventsub::EventSubscription + Send,
{
    RHelixClient::new()
        .create_eventsub_subscription(
            subscription,
            Transport::webhook(
                format!("{}/api/v1/eventsub/reward", CONFIG.server.url),
                CONFIG.twitch.eventsub.secret.to_string(),
            ),
            token,
        )
        .await
        .err_into()
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
            UpdateRedemptionStatusRequest::new(
                broadcaster_id,
                reward_id,
                redemption_id,
            ),
            UpdateRedemptionStatusBody::builder().status(status).build(),
            token,
        )
        .await?;

    match response.data {
        UpdateRedemptionStatusInformation::Success(r) => Ok(r),
        _ => Err(TwitchApiError::Custom),
    }
}
