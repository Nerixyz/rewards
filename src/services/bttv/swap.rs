use anyhow::{Error as AnyError, Result as AnyResult};
use rand::prelude::SliceRandom;
use sqlx::PgPool;

use crate::models::user::User;
use crate::services::bttv::prepare_add_emote;
use crate::services::bttv::requests::{
    add_shared_emote, delete_shared_emote, get_emote, BttvEmote,
};

pub async fn swap_or_add_emote(
    user_id: &str,
    emote_id: &str,
    pool: &PgPool,
) -> AnyResult<(Option<String>, String)> {
    let (this_user, bttv_user, user_limits, emote_data) =
        prepare_add_emote(user_id, emote_id, pool).await?;
    // if the emote-limit is reached, remove the oldest added emote or a random one
    let (removed_emote, mut history) = if bttv_user.shared_emotes.len() >= user_limits.shared_emotes
    {
        remove_emote(
            this_user.bttv_history.0,
            &bttv_user.id,
            bttv_user.shared_emotes,
        )
        .await
        .map_err(|_| AnyError::msg("No emote to be removed."))?
    } else {
        (None, this_user.bttv_history.0)
    };

    // now the actual adding happens
    if let Err(e) = add_shared_emote(emote_id, &bttv_user.id).await {
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

    let removed_emote = if let Some(id) = removed_emote {
        Some(get_emote(&id).await.map(|e| e.code).unwrap_or_else(|e| {
            log::warn!(
                "Emote {} was added in {} but isn't there anymore error={}",
                id,
                user_id,
                e
            );
            "[?]".to_string()
        }))
    } else {
        None
    };

    Ok((removed_emote, emote_data.code))
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
