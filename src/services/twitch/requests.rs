use crate::services::twitch::errors::TwitchApiError;
use crate::services::twitch::{HelixResult, RHelixClient};
use twitch_api2::helix::streams::{GetStreamsRequest, Stream};
use twitch_api2::twitch_oauth2::TwitchToken;
use twitch_api2::types::{Nickname, RewardId, UserId};
use twitch_api2::{
    helix::{
        points::{
            update_custom_reward::UpdateCustomReward, CreateCustomRewardBody,
            CreateCustomRewardRequest, CreateCustomRewardResponse, CustomReward,
            DeleteCustomRewardRequest, GetCustomRewardRequest, UpdateCustomRewardBody,
            UpdateCustomRewardRequest,
        },
        users::{GetUsersRequest, User},
        Response,
    },
    twitch_oauth2::UserToken,
};

pub async fn create_reward(
    user_id: &str,
    req: CreateCustomRewardBody,
    token: &UserToken,
) -> HelixResult<CreateCustomRewardResponse> {
    let response: Response<CreateCustomRewardRequest, CreateCustomRewardResponse> =
        RHelixClient::default()
            .req_post(
                CreateCustomRewardRequest::builder()
                    .broadcaster_id(user_id)
                    .build(),
                req,
                token,
            )
            .await?;

    Ok(response.data)
}

pub async fn update_reward(
    user_id: &str,
    id: String,
    req: UpdateCustomRewardBody,
    token: &UserToken,
) -> HelixResult<CustomReward> {
    let response: Response<UpdateCustomRewardRequest, UpdateCustomReward> = RHelixClient::default()
        .req_patch(
            UpdateCustomRewardRequest::builder()
                .broadcaster_id(user_id)
                .id(id)
                .build(),
            req,
            token,
        )
        .await?;

    match response.data {
        UpdateCustomReward::Success(r) => Ok(r),
        _ => Err(TwitchApiError::Other("Expected Success".to_string())),
    }
}

pub async fn delete_reward<I: Into<RewardId>>(
    user_id: &str,
    id: I,
    token: &UserToken,
) -> HelixResult<()> {
    RHelixClient::default()
        .req_delete(
            DeleteCustomRewardRequest::builder()
                .broadcaster_id(user_id)
                .id(id.into())
                .build(),
            token,
        )
        .await?;

    Ok(())
}

pub async fn get_rewards_for_id(
    broadcaster: &str,
    token: &UserToken,
) -> HelixResult<Vec<CustomReward>> {
    let response: Response<GetCustomRewardRequest, Vec<CustomReward>> = RHelixClient::default()
        .req_get(
            GetCustomRewardRequest::builder()
                .broadcaster_id(broadcaster)
                .only_manageable_rewards(Some(true))
                .build(),
            token,
        )
        .await?;

    Ok(response.data)
}

pub async fn get_reward_for_broadcaster_by_id(
    broadcaster: &str,
    id: String,
    token: &UserToken,
) -> HelixResult<CustomReward> {
    let response: Response<GetCustomRewardRequest, Vec<CustomReward>> = RHelixClient::default()
        .req_get(
            GetCustomRewardRequest::builder()
                .broadcaster_id(broadcaster)
                .id(vec![RewardId::new(id)])
                .only_manageable_rewards(Some(true))
                .build(),
            token,
        )
        .await?;

    response
        .data
        .into_iter()
        .next()
        .ok_or_else(|| TwitchApiError::Other("No reward found".to_string()))
}

pub async fn get_user(id: String, token: &UserToken) -> HelixResult<User> {
    let response: Response<GetUsersRequest, Vec<User>> = RHelixClient::default()
        .req_get(
            GetUsersRequest::builder().id(vec![UserId::new(id)]).build(),
            token,
        )
        .await?;

    response
        .data
        .into_iter()
        .next()
        .ok_or_else(|| TwitchApiError::Other("Could not find user".to_string()))
}

pub async fn get_user_by_login<T: TwitchToken>(login: String, token: &T) -> HelixResult<User> {
    let response: Response<GetUsersRequest, Vec<User>> = RHelixClient::default()
        .req_get(
            GetUsersRequest::builder()
                .login(vec![Nickname::new(login)])
                .build(),
            token,
        )
        .await?;

    response
        .data
        .into_iter()
        .next()
        .ok_or_else(|| TwitchApiError::Other("Could not find user".to_string()))
}

pub async fn get_users(ids: Vec<String>, token: &UserToken) -> HelixResult<Vec<User>> {
    let response: Response<GetUsersRequest, Vec<User>> = RHelixClient::default()
        .req_get(
            GetUsersRequest::builder()
                .id(ids.into_iter().map(UserId::new).collect())
                .build(),
            token,
        )
        .await?;

    Ok(response.data)
}

pub async fn is_user_live<T: TwitchToken>(id: String, token: &T) -> HelixResult<bool> {
    let response: Response<GetStreamsRequest, Vec<Stream>> = RHelixClient::default()
        .req_get(
            GetStreamsRequest::builder()
                .user_id(vec![UserId::new(id)])
                .build(),
            token,
        )
        .await?;

    Ok(response.data.into_iter().next().is_some())
}
