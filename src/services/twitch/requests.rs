use crate::services::twitch::{HelixResult, RHelixClient};
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
        _ => Err("".into()),
    }
}

pub async fn delete_reward(user_id: &str, id: String, token: &UserToken) -> HelixResult<()> {
    RHelixClient::default()
        .req_delete(
            DeleteCustomRewardRequest::builder()
                .broadcaster_id(user_id)
                .id(id)
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
                .id(vec![id])
                .only_manageable_rewards(Some(true))
                .build(),
            token,
        )
        .await?;

    response
        .data
        .into_iter()
        .nth(0)
        .ok_or("Could not find reward".into())
}

pub async fn get_user(id: String, token: &UserToken) -> HelixResult<User> {
    let response: Response<GetUsersRequest, Vec<User>> = RHelixClient::default()
        .req_get(GetUsersRequest::builder().id(vec![id]).build(), token)
        .await?;

    response
        .data
        .into_iter()
        .nth(0)
        .ok_or("Could not find user".into())
}

pub async fn get_user_by_login(login: String, token: &UserToken) -> HelixResult<User> {
    let response: Response<GetUsersRequest, Vec<User>> = RHelixClient::default()
        .req_get(GetUsersRequest::builder().login(vec![login]).build(), token)
        .await?;

    response
        .data
        .into_iter()
        .nth(0)
        .ok_or("Could not find user".into())
}

pub async fn get_users(ids: Vec<String>, token: &UserToken) -> HelixResult<Vec<User>> {
    let response: Response<GetUsersRequest, Vec<User>> = RHelixClient::default()
        .req_get(GetUsersRequest::builder().id(ids).build(), token)
        .await?;

    Ok(response.data)
}
