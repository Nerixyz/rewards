use crate::services::twitch::{
    errors::TwitchApiError, HelixResult, RHelixClient, HELIX_CLIENT,
};
use anyhow::anyhow;
use config::CONFIG;
use models::timed_mode;
use reqwest::StatusCode;
use twitch_api::{
    helix::{
        chat::{
            ChatSettings, GetChatSettingsRequest, SendChatMessageBody,
            SendChatMessageRequest, UpdateChatSettingsBody,
            UpdateChatSettingsRequest,
        },
        moderation::{BanUserBody, BanUserRequest, GetBannedUsersRequest},
        points::{
            update_custom_reward::UpdateCustomReward, CreateCustomRewardBody,
            CreateCustomRewardRequest, CreateCustomRewardResponse,
            CustomReward, DeleteCustomRewardRequest, GetCustomRewardRequest,
            UpdateCustomRewardBody, UpdateCustomRewardRequest,
        },
        streams::{GetStreamsRequest, Stream},
        users::{GetUsersRequest, User},
        ClientRequestError, HelixRequestPostError, Response,
    },
    twitch_oauth2::{tokens::errors::ValidationError, TwitchToken, UserToken},
    types::{IntoCow, RewardIdRef, UserId, UserIdRef, UserNameRef},
};

pub async fn create_reward<'a>(
    user_id: &str,
    req: CreateCustomRewardBody<'a>,
    token: &UserToken,
) -> HelixResult<CreateCustomRewardResponse> {
    let response: Response<
        CreateCustomRewardRequest,
        CreateCustomRewardResponse,
    > = RHelixClient::default()
        .req_post(
            CreateCustomRewardRequest::broadcaster_id(user_id),
            req,
            token,
        )
        .await?;

    Ok(response.data)
}

pub async fn update_reward<'a>(
    user_id: impl IntoCow<'a, UserIdRef> + 'a,
    id: impl IntoCow<'a, RewardIdRef> + 'a,
    req: UpdateCustomRewardBody<'a>,
    token: &UserToken,
) -> HelixResult<CustomReward> {
    let response: Response<UpdateCustomRewardRequest, UpdateCustomReward> =
        RHelixClient::default()
            .req_patch(UpdateCustomRewardRequest::new(user_id, id), req, token)
            .await?;

    match response.data {
        UpdateCustomReward::Success(r) => Ok(r),
        _ => Err(TwitchApiError::Other("Expected Success".to_string())),
    }
}

pub async fn delete_reward<'a>(
    user_id: impl IntoCow<'a, UserIdRef> + 'a,
    id: impl IntoCow<'a, RewardIdRef> + 'a,
    token: &UserToken,
) -> HelixResult<()> {
    RHelixClient::default()
        .req_delete(DeleteCustomRewardRequest::new(user_id, id), token)
        .await?;

    Ok(())
}

pub async fn get_rewards_for_id<'a>(
    broadcaster: impl IntoCow<'a, UserIdRef> + 'a,
    token: &UserToken,
) -> HelixResult<Vec<CustomReward>> {
    let response: Response<GetCustomRewardRequest, Vec<CustomReward>> =
        RHelixClient::default()
            .req_get(
                GetCustomRewardRequest::broadcaster_id(broadcaster)
                    .only_manageable_rewards(true),
                token,
            )
            .await?;

    Ok(response.data)
}

pub async fn get_reward_for_broadcaster_by_id<'a>(
    user_id: impl IntoCow<'a, UserIdRef> + 'a,
    ids: &'a [&'a RewardIdRef],
    token: &UserToken,
) -> HelixResult<CustomReward> {
    let response: Response<GetCustomRewardRequest, Vec<CustomReward>> =
        RHelixClient::default()
            .req_get(
                GetCustomRewardRequest::broadcaster_id(user_id)
                    .only_manageable_rewards(true)
                    .ids(ids),
                token,
            )
            .await?;

    response
        .data
        .into_iter()
        .next()
        .ok_or_else(|| TwitchApiError::Other("No reward found".to_string()))
}

pub async fn get_user(
    id: impl AsRef<str>,
    token: &UserToken,
) -> HelixResult<User> {
    let ids: &[&UserIdRef] = &[id.as_ref().into()];
    let response: Response<GetUsersRequest, Vec<User>> =
        RHelixClient::default()
            .req_get(GetUsersRequest::ids(ids), token)
            .await?;

    response
        .data
        .into_iter()
        .next()
        .ok_or_else(|| TwitchApiError::Other("Could not find user".to_string()))
}

pub async fn get_user_by_login<T: TwitchToken>(
    login: impl AsRef<str>,
    token: &T,
) -> HelixResult<User> {
    let logins: &[&UserNameRef] = &[login.as_ref().into()];
    let response: Response<GetUsersRequest, Vec<User>> =
        RHelixClient::default()
            .req_get(GetUsersRequest::logins(logins), token)
            .await?;

    response
        .data
        .into_iter()
        .next()
        .ok_or_else(|| TwitchApiError::Other("Could not find user".to_string()))
}

pub async fn get_users(
    ids: &[&UserIdRef],
    token: &UserToken,
) -> HelixResult<Vec<User>> {
    let response: Response<GetUsersRequest, Vec<User>> =
        RHelixClient::default()
            .req_get(GetUsersRequest::ids(ids), token)
            .await?;

    Ok(response.data)
}

pub async fn is_user_live<'a, T: TwitchToken>(
    id: impl Into<&'a UserIdRef>,
    token: &T,
) -> HelixResult<bool> {
    let ids: &[&UserIdRef] = &[id.into()];
    let response: Response<GetStreamsRequest, Vec<Stream>> =
        RHelixClient::default()
            .req_get(GetStreamsRequest::user_ids(ids), token)
            .await?;

    Ok(response.data.into_iter().next().is_some())
}

pub async fn validate_token(token: &UserToken) -> anyhow::Result<bool> {
    match token.validate_token(&RHelixClient::default()).await {
        Ok(_) => Ok(true),
        Err(ValidationError::NotAuthorized) => Ok(false),
        Err(e) => Err(e.into()),
    }
}

// copesen this will work some day
// TODO: user_ids -> user_id
#[allow(unused)]
pub async fn get_moderator_id_for_banned_user<'a>(
    broadcaster_id: &'a str,
    user_ids: &'a [&'a UserIdRef],
    token: &impl TwitchToken,
) -> anyhow::Result<Option<UserId>> {
    let data = HELIX_CLIENT
        .req_get(
            GetBannedUsersRequest::broadcaster_id(
                CONFIG.debug_overrides.twitch(broadcaster_id),
            )
            .users(user_ids),
            token,
        )
        .await?;

    Ok(data.data.into_iter().next().map(|u| u.moderator_id))
}

pub async fn timeout_user<'a>(
    broadcaster_id: &'a str,
    moderator_id: impl IntoCow<'a, UserIdRef> + 'a,
    user_id: impl IntoCow<'a, UserIdRef> + 'a,
    duration: std::time::Duration,
    reason: &'a str,
    token: &impl TwitchToken,
) -> anyhow::Result<()> {
    let res = HELIX_CLIENT
        .req_post(
            BanUserRequest::new(
                CONFIG.debug_overrides.twitch(broadcaster_id),
                moderator_id,
            ),
            BanUserBody::new(user_id, reason, Some(duration.as_secs() as u32)),
            token,
        )
        .await;

    match res {
        Ok(_) => Ok(()),
        Err(ClientRequestError::HelixRequestPostError(
            HelixRequestPostError::Error {
                status: StatusCode::BAD_REQUEST,
                message,
                ..
            },
        )) if message.contains(
            "user specified in the user_id field is already banned",
        ) =>
        {
            Err(anyhow!("I can't timeout banned users."))
        }
        Err(ClientRequestError::HelixRequestPostError(
            HelixRequestPostError::Error {
                status: StatusCode::BAD_REQUEST,
                message,
                ..
            },
        )) if message.contains(
            "user specified in the user_id field may not be banned",
        ) =>
        {
            Err(anyhow!("I can't timeout mods or broadcasters."))
        }
        Err(e) => Err(e.into()),
    }
}

pub async fn get_chat_settings(
    broadcaster_id: &str,
    token: &impl TwitchToken,
) -> anyhow::Result<ChatSettings> {
    let res = HELIX_CLIENT
        .req_get(
            GetChatSettingsRequest::broadcaster_id(
                CONFIG.debug_overrides.twitch(broadcaster_id),
            ),
            token,
        )
        .await?;
    Ok(res.data)
}

pub async fn update_chat_settings<'a>(
    broadcaster_id: &'a str,
    moderator_id: impl IntoCow<'a, UserIdRef> + 'a,
    timed_mode: timed_mode::Mode,
    enable: bool,
    token: &impl TwitchToken,
) -> anyhow::Result<()> {
    HELIX_CLIENT
        .req_patch(
            UpdateChatSettingsRequest::new(
                CONFIG.debug_overrides.twitch(broadcaster_id),
                moderator_id,
            ),
            UpdateChatSettingsBody::builder()
                .emote_mode(
                    (timed_mode == timed_mode::Mode::Emoteonly)
                        .then_some(enable),
                )
                .subscriber_mode(
                    (timed_mode == timed_mode::Mode::Subonly).then_some(enable),
                )
                .follower_mode(None)
                .follower_mode_duration(None)
                .slow_mode(None)
                .non_moderator_chat_delay(None)
                .non_moderator_chat_delay_duration(None)
                .slow_mode_wait_time(None)
                .unique_chat_mode(None)
                .build(),
            token,
        )
        .await?;
    Ok(())
}

pub async fn send_chat_message(
    broadcaster_id: &str,
    message: &str,
    token: &impl TwitchToken,
) -> anyhow::Result<()> {
    let res = HELIX_CLIENT
        .req_post(
            SendChatMessageRequest::new(),
            SendChatMessageBody::new(
                broadcaster_id,
                &CONFIG.twitch.user_id,
                message,
            ),
            token,
        )
        .await?;
    if res.data.is_sent {
        Ok(())
    } else {
        Err(anyhow::Error::msg(
            res.data
                .drop_reason
                .map(|d| d.message)
                .unwrap_or("No drop reason given".to_string()),
        ))
    }
}
