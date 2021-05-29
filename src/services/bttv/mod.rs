use crate::models::user::User;
use crate::services::bttv::requests::{
    add_shared_emote, delete_shared_emote, get_dashboards, get_emote, get_user,
    get_user_by_twitch_id, BttvEmote, BttvLimits,
};
use anyhow::{Error as AnyError, Result as AnyResult};
use rand::prelude::SliceRandom;
use sqlx::PgPool;

pub mod requests;

pub async fn fetch_save_bttv_id(user_id: &str, pool: &PgPool) -> AnyResult<String> {
    let user = get_user_by_twitch_id(user_id).await?;
    User::set_bttv_id(user_id, &user.id, pool).await?;

    Ok(user.id)
}

pub async fn swap_or_add_emote(
    user_id: &str,
    emote_id: &str,
    pool: &PgPool,
) -> AnyResult<(Option<String>, String)> {
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
    let (bttv_user, user_limits, emote_data) = futures::future::join3(
        get_user(&bttv_id),
        get_user_limits(&bttv_id),
        get_emote(emote_id),
    )
    .await;
    let (bttv_user, user_limits, emote_data) = (
        bttv_user.map_err(|_| AnyError::msg("No such user."))?,
        user_limits.map_err(|_| AnyError::msg("I'm not added as an editor."))?,
        emote_data.map_err(|_| AnyError::msg("This emote doesn't exist."))?,
    );

    // check if there's already an emote with the same name or id
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

    // if the emote-limit is reached, remove the oldest added emote or a random one
    let (removed_emote, mut history) = if bttv_user.shared_emotes.len() >= user_limits.shared_emotes
    {
        remove_emote(this_user.bttv_history.0, &bttv_id, bttv_user.shared_emotes)
            .await
            .map_err(|_| AnyError::msg("No emote to be removed."))?
    } else {
        (None, this_user.bttv_history.0)
    };

    // now the actual adding happens
    if let Err(e) = add_shared_emote(emote_id, &bttv_id).await {
        if let Err(sql_err) = User::set_bttv_history(user_id, history, pool).await {
            log::warn!(
                "Error setting bttv history after failing to insert shared emote: sql_error={}",
                sql_err
            );
        }
        log::warn!("Could not add shared emote: {}", e);
        return Err(AnyError::msg("Couldn't add shared emote."));
    }
    history.push(emote_id.to_string());
    User::set_bttv_history(user_id, history, pool)
        .await
        .map_err(|_| AnyError::msg("Internal error"))?;

    Ok((removed_emote, emote_data.code))
}

pub async fn get_user_limits(bttv_id: &str) -> AnyResult<BttvLimits> {
    get_dashboards()
        .await?
        .into_iter()
        .find(|d| d.id == bttv_id)
        .map(|u| u.limits)
        .ok_or_else(|| AnyError::msg("User isn't an editor"))
}

async fn remove_emote(
    mut history: Vec<String>,
    bttv_id: &str,
    shared_emotes: Vec<BttvEmote>,
) -> AnyResult<(Option<String>, Vec<String>)> {
    let mut iter = history.into_iter();
    let mut emote = None;
    while let Some(id) = iter.next() {
        if let Err(e) = delete_shared_emote(&id, bttv_id).await {
            log::info!("Skipping shared emote: id={}; error={}", id, e);
            continue;
        }
        emote = Some(id);
        break;
    }
    // add the remaining back to the history
    history = iter.collect();

    let emote = match emote {
        Some(id) => id,
        None => {
            // There are no emotes in history, remove a random one
            let emote = shared_emotes.choose(&mut rand::thread_rng());

            if let Some(emote) = emote {
                delete_shared_emote(&emote.id, bttv_id).await?;

                emote.id.clone()
            } else {
                // this should never happen as this function is only called if there are too many emotes
                log::warn!("Invalid branch - there are no emotes to remove but the limit is reached?! id={}", bttv_id);
                return Err(AnyError::msg("There are no emotes to remove"));
            }
        }
    };

    Ok((Some(emote), history))
}
