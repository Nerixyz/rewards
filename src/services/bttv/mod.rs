use anyhow::{Error as AnyError, Result as AnyResult};
use sqlx::PgPool;

use crate::models::user::User;
use crate::services::bttv::requests::{get_dashboards, get_user_by_twitch_id, BttvLimits};

pub mod requests;

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
