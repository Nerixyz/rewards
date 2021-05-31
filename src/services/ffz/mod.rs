use crate::models::user::User;
use crate::services::ffz::requests::{
    add_emote, delete_emote, get_channels, get_emote, get_room, get_user, FfzEmote,
};
use anyhow::{Error as AnyError, Result as AnyResult};
use rand::prelude::SliceRandom;
use sqlx::PgPool;

mod requests;

pub async fn swap_or_add_emote(
    user_id: &str,
    emote_id: &str,
    pool: &PgPool,
) -> AnyResult<(Option<String>, String)> {
    // TODO: investigate try_join4
    let (ffz_user, ffz_emote, ffz_room, ffz_history) = futures::future::join4(
        get_user(user_id),
        get_emote(emote_id),
        get_room(user_id),
        User::get_ffz_history(user_id, pool),
    )
    .await;
    let (ffz_user, ffz_emote, ffz_room, ffz_history) = (
        ffz_user.map_err(|e| {
            log::warn!("err: {}", e);
            AnyError::msg("No such ffz-user")
        })?,
        ffz_emote.map_err(|e| {
            log::warn!("err: {}", e);
            AnyError::msg("No such emote")
        })?,
        ffz_room.map_err(|e| {
            log::warn!("err: {}", e);
            AnyError::msg("No such ffz-room")
        })?,
        ffz_history.map_err(|e| {
            log::warn!("err: {}", e);
            AnyError::msg("No history?!")
        })?,
    );
    let room_emotes: Vec<FfzEmote> = ffz_room
        .sets
        .into_iter()
        .map(|s| s.1.emoticons)
        .flatten()
        .collect();

    if room_emotes
        .iter()
        .any(|e| e.id == ffz_emote.id || e.name == ffz_emote.name)
    {
        return Err(AnyError::msg("The emote is already added"));
    }

    let (removed_emote, mut history) = if room_emotes.len() >= ffz_user.max_emoticons {
        remove_emote(ffz_history, ffz_room.room._id, room_emotes)
            .await
            .map_err(|_| AnyError::msg("No emote to be removed"))?
    } else {
        (None, ffz_history)
    };

    if let Err(e) = add_emote(ffz_room.room._id, ffz_emote.id).await {
        if let Err(sql_err) = User::set_ffz_history(user_id, history, pool).await {
            log::warn!(
                "Error setting ffz history after failing to insert shared emote: sql_error={}",
                sql_err
            );
        }
        log::warn!("Could not add ffz emote: {}", e);
        return Err(AnyError::msg("Couldn't add ffz emote."));
    }
    history.push(ffz_emote.id);
    User::set_ffz_history(user_id, history, pool)
        .await
        .map_err(|_| AnyError::msg("Internal error"))?;

    let removed_emote = if let Some(id) = removed_emote {
        Some(
            get_emote(&id.to_string())
                .await
                .map(|e| e.name)
                .unwrap_or_else(|e| {
                    log::warn!(
                        "Emote {} was added in {} but isn't there anymore error={}",
                        id,
                        ffz_user.name,
                        e
                    );
                    "[?]".to_string()
                }),
        )
    } else {
        None
    };
    Ok((removed_emote, ffz_emote.name))
}

pub async fn is_editor_in(name: &str) -> bool {
    get_channels()
        .await
        .map(|channels| channels.iter().any(|channel| channel == name))
        .unwrap_or(false)
}

// TODO: similar to bttv one - merge
async fn remove_emote(
    mut history: Vec<usize>,
    channel_id: usize,
    emotes: Vec<FfzEmote>,
) -> AnyResult<(Option<usize>, Vec<usize>)> {
    let mut iter = history.into_iter();
    let mut emote = None;
    while let Some(id) = iter.next() {
        if let Err(e) = delete_emote(channel_id, id).await {
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
            let emote = emotes.choose(&mut rand::thread_rng());

            if let Some(emote) = emote {
                delete_emote(channel_id, emote.id).await?;

                emote.id
            } else {
                // this should never happen as this function is only called if there are too many emotes
                log::warn!("Invalid branch - there are no emotes to remove but the limit is reached?! id={}", channel_id);
                return Err(AnyError::msg("There are no emotes to remove"));
            }
        }
    };

    Ok((Some(emote), history))
}
