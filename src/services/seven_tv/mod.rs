use anyhow::{Error as AnyError, Result as AnyResult};

use crate::{config::CONFIG, models::user::User, services::seven_tv::requests::get_user_editors};
use sqlx::PgPool;

pub mod requests;

pub async fn fetch_save_seventv_id(
    user_id: &str,
    user_login: &str,
    pool: &PgPool,
) -> AnyResult<String> {
    let user = requests::get_user(user_login).await?;
    User::set_seventv_id(user_id, &user.id, pool).await?;

    Ok(user.id)
}

pub async fn get_or_fetch_id(user_id: &str, pool: &PgPool) -> AnyResult<String> {
    let this_user = User::get_seventv_data(user_id, pool).await?;
    Ok(if let Some(id) = this_user.seventv_id {
        id
    } else {
        fetch_save_seventv_id(user_id, &this_user.name, pool)
            .await
            .map_err(|_| AnyError::msg("The user hasn't registered on seventv yet"))?
    })
}

pub async fn verify_user(broadcaster_id: &str, pool: &PgPool) -> AnyResult<String> {
    let this_user = User::get_seventv_data(broadcaster_id, pool).await?;
    let seventv_id = if let Some(id) = &this_user.seventv_id {
        id.clone()
    } else {
        fetch_save_seventv_id(broadcaster_id, &this_user.name, pool)
            .await
            .map_err(|_| AnyError::msg("The user hasn't registered on seventv yet"))?
    };
    let editors = get_user_editors(&seventv_id).await?;

    if editors.iter().any(|e| e.login == CONFIG.twitch.login) {
        Ok(seventv_id)
    } else {
        Err(AnyError::msg("RewardMore isn't an editor for the user"))
    }
}
