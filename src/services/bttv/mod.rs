use anyhow::{Error as AnyError, Result as AnyResult};
use sqlx::PgPool;

use crate::models::user::{User, UserBttvData};
use crate::services::bttv::requests::{
    get_dashboards, get_emote, get_user, get_user_by_twitch_id, BttvEmote, BttvLimits, BttvUserInfo,
};
use futures::TryFutureExt;

pub mod requests;
pub mod slots;
pub mod swap;

pub async fn fetch_save_bttv_id(user_id: &str, pool: &PgPool) -> AnyResult<String> {
    let user = get_user_by_twitch_id(user_id).await?;
    User::set_bttv_id(user_id, &user.id, pool).await?;

    Ok(user.id)
}

pub async fn get_or_fetch_id(user_id: &str, pool: &PgPool) -> AnyResult<String> {
    let this_user = User::get_bttv_data(user_id, pool).await?;
    Ok(if let Some(id) = this_user.bttv_id {
        id
    } else {
        fetch_save_bttv_id(user_id, pool)
            .await
            .map_err(|_| AnyError::msg("The user hasn't registered on bttv yet"))?
    })
}

pub async fn verify_user(broadcaster_id: &str, pool: &PgPool) -> AnyResult<String> {
    let this_user = User::get_bttv_data(broadcaster_id, pool).await?;
    let bttv_id = if let Some(id) = &this_user.bttv_id {
        id.clone()
    } else {
        fetch_save_bttv_id(broadcaster_id, pool)
            .await
            .map_err(|_| AnyError::msg("The user hasn't registered on bttv yet"))?
    };
    get_user_limits(&bttv_id)
        .await
        .map_err(|_| AnyError::msg("RewardMore isn't an editor for the user"))?;

    Ok(bttv_id)
}

pub async fn get_user_limits(bttv_id: &str) -> AnyResult<BttvLimits> {
    get_dashboards()
        .await?
        .into_iter()
        .find(|d| d.id == bttv_id)
        .map(|u| u.limits)
        .ok_or_else(|| AnyError::msg("User isn't an editor"))
}

/// Checks if emote and user are valid and if the emote already exists.
/// It doesn't remove or add any emote
async fn prepare_add_emote(
    user_id: &str,
    emote_id: &str,
    pool: &PgPool,
) -> AnyResult<(UserBttvData, BttvUserInfo, BttvLimits, BttvEmote)> {
    let this_user = User::get_bttv_data(user_id, pool)
        .await
        .map_err(|_| AnyError::msg("Internal Error."))?;
    let bttv_id = if let Some(id) = &this_user.bttv_id {
        id.clone()
    } else {
        fetch_save_bttv_id(user_id, pool)
            .await
            .map_err(|_| AnyError::msg("No such user."))?
    };

    // get the data in parallel
    let (bttv_user, user_limits, emote_data) = futures::future::try_join3(
        get_user(&bttv_id).map_err(|_| AnyError::msg("No such user.")),
        get_user_limits(&bttv_id).map_err(|_| AnyError::msg("I'm not added as an editor.")),
        get_emote(emote_id).map_err(|_| AnyError::msg("This emote doesn't exist.")),
    )
    .await?;

    // check if there's already an emote with the same name or id
    // If the added emote will replace an emote with the same name it will never work!
    if bttv_user
        .shared_emotes
        .iter()
        .any(|e| e.id == emote_id || e.code == emote_data.code)
    {
        return Err(AnyError::msg("The emote already exists as a shared emote"));
    }
    if bttv_user
        .channel_emotes
        .iter()
        .any(|e| e.id == emote_id || e.code == emote_data.code)
    {
        return Err(AnyError::msg("The emote already exists as a channel emote"));
    }

    Ok((this_user, bttv_user, user_limits, emote_data))
}
