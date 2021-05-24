use crate::services::twitch::errors::to_response_error;
use actix_web::{error, Error};
use twitch_api2::helix::points::update_custom_reward::UpdateCustomReward;
use twitch_api2::helix::points::{
    CreateCustomRewardBody, CreateCustomRewardRequest, CreateCustomRewardResponse, CustomReward,
    DeleteCustomReward, DeleteCustomRewardRequest, GetCustomRewardRequest, UpdateCustomRewardBody,
    UpdateCustomRewardRequest,
};
use twitch_api2::helix::users::{GetUsersRequest, User};
use twitch_api2::helix::Response;
use twitch_api2::twitch_oauth2::UserToken;
use twitch_api2::HelixClient;

pub async fn create_reward(
    user_id: &str,
    req: CreateCustomRewardBody,
    token: &UserToken,
) -> Result<CreateCustomRewardResponse, Error> {
    let response: Response<CreateCustomRewardRequest, CreateCustomRewardResponse> =
        HelixClient::<'_, reqwest::Client>::default()
            .req_post(
                CreateCustomRewardRequest::builder()
                    .broadcaster_id(user_id)
                    .build(),
                req,
                token,
            )
            .await
            .map_err(to_response_error)?;

    Ok(response.data)
}

pub async fn update_reward(
    user_id: &str,
    id: String,
    req: UpdateCustomRewardBody,
    token: &UserToken,
) -> Result<CustomReward, Error> {
    let response: Response<UpdateCustomRewardRequest, UpdateCustomReward> =
        HelixClient::<'_, reqwest::Client>::default()
            .req_patch(
                UpdateCustomRewardRequest::builder()
                    .broadcaster_id(user_id)
                    .id(id)
                    .build(),
                req,
                token,
            )
            .await
            .map_err(to_response_error)?;

    match response.data {
        UpdateCustomReward::Success(r) => Ok(r),
        _ => Err(error::ErrorInternalServerError("")),
    }
}

pub async fn delete_reward(user_id: &str, id: String, token: &UserToken) -> Result<(), Error> {
    let response: DeleteCustomReward = HelixClient::<'_, reqwest::Client>::default()
        .req_delete(
            DeleteCustomRewardRequest::builder()
                .broadcaster_id(user_id)
                .id(id)
                .build(),
            token,
        )
        .await
        .map_err(to_response_error)?;

    match response {
        DeleteCustomReward::Success => Ok(()),
        _ => Err(error::ErrorInternalServerError("")),
    }
}

pub async fn get_rewards_for_id(
    broadcaster: &str,
    token: &UserToken,
) -> Result<Vec<CustomReward>, Error> {
    let response: Response<GetCustomRewardRequest, Vec<CustomReward>> =
        HelixClient::<'_, reqwest::Client>::default()
            .req_get(
                GetCustomRewardRequest::builder()
                    .broadcaster_id(broadcaster)
                    .only_manageable_rewards(Some(true))
                    .build(),
                token,
            )
            .await
            .map_err(to_response_error)?;

    Ok(response.data)
}

pub async fn get_reward_for_broadcaster_by_id(
    broadcaster: &str,
    id: String,
    token: &UserToken,
) -> Result<CustomReward, Error> {
    let response: Response<GetCustomRewardRequest, Vec<CustomReward>> =
        HelixClient::<'_, reqwest::Client>::default()
            .req_get(
                GetCustomRewardRequest::builder()
                    .broadcaster_id(broadcaster)
                    .id(vec![id])
                    .only_manageable_rewards(Some(true))
                    .build(),
                token,
            )
            .await
            .map_err(to_response_error)?;

    response
        .data
        .into_iter()
        .nth(0)
        .ok_or(error::ErrorNotFound("").into())
}

pub async fn get_user(id: String, token: &UserToken) -> Result<User, Error> {
    let response: Response<GetUsersRequest, Vec<User>> =
        HelixClient::<'_, reqwest::Client>::default()
            .req_get(GetUsersRequest::builder().id(vec![id]).build(), token)
            .await
            .map_err(to_response_error)?;

    response
        .data
        .into_iter()
        .nth(0)
        .ok_or(error::ErrorNotFound("").into())
}

pub async fn get_user_by_login(login: String, token: &UserToken) -> Result<User, Error> {
    let response: Response<GetUsersRequest, Vec<User>> =
        HelixClient::<'_, reqwest::Client>::default()
            .req_get(GetUsersRequest::builder().login(vec![login]).build(), token)
            .await
            .map_err(to_response_error)?;

    response
        .data
        .into_iter()
        .nth(0)
        .ok_or(error::ErrorNotFound("").into())
}

pub async fn get_users(ids: Vec<String>, token: &UserToken) -> Result<Vec<User>, Error> {
    let response: Response<GetUsersRequest, Vec<User>> =
        HelixClient::<'_, reqwest::Client>::default()
            .req_get(GetUsersRequest::builder().id(ids).build(), token)
            .await
            .map_err(to_response_error)?;

    Ok(response.data)
}
