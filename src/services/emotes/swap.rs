use crate::{
    models::reward::SwapRewardData,
    services::emotes::{Emote, EmoteRW},
};
use anyhow::{Error as AnyError, Result as AnyResult};
use rand::prelude::SliceRandom;
use sqlx::PgPool;
use std::fmt::Display;

pub async fn swap_or_add_emote<RW, I, E, EI>(
    broadcaster_id: &str,
    emote_id: &str,
    reward_data: SwapRewardData,
    pool: &PgPool,
) -> AnyResult<(Option<String>, String)>
where
    RW: EmoteRW<PlatformId = I, Emote = E, EmoteId = EI>,
    I: Display,
    EI: Display + ToOwned<Owned = EI>,
    E: Emote<EI>,
{
    let data = RW::get_check_initial_data(broadcaster_id, emote_id, pool).await?;
    let (removed_emote, mut history) = if data.current_emotes >= data.max_emotes
        || reward_data
            .limit
            .map(|l| data.history.len() >= l as usize)
            .unwrap_or(false)
    {
        remove_last_emote::<RW, I, E, EI>(data.history, &data.platform_id, &data.emotes).await?
    } else {
        (None, data.history)
    };

    log::info!(
        "Add emote_id={} to platform_id={}",
        data.emote.id(),
        data.platform_id
    );
    if let Err(e) = RW::add_emote(&data.platform_id, data.emote.id()).await {
        if let Err(sql_err) = RW::save_history(broadcaster_id, history, pool).await {
            log::warn!(
                "Error setting history after failing to insert shared emote: sql_error={}",
                sql_err
            );
        }
        log::warn!("Could not add emote: {}", e);
        return Err(AnyError::msg("Couldn't add emote."));
    }
    history.push(data.emote.id().to_owned());
    RW::save_history(broadcaster_id, history, pool)
        .await
        .map_err(|_| AnyError::msg("Internal Error"))?;

    let removed_emote = if let Some(id) = removed_emote {
        Some(
            RW::get_emote_by_id(&id)
                .await
                .map(|e| e.name())
                .unwrap_or_else(|e| {
                    log::warn!(
                        "Emote {} was added in {} but isn't there anymore error={}",
                        id,
                        broadcaster_id,
                        e
                    );
                    "[?]".to_string()
                }),
        )
    } else {
        None
    };

    Ok((removed_emote, data.emote.name()))
}

pub async fn remove_last_emote<RW, I, E, EI>(
    mut history: Vec<EI>,
    platform_id: &I,
    current_emotes: &[E],
) -> AnyResult<(Option<EI>, Vec<EI>)>
where
    RW: EmoteRW<PlatformId = I, Emote = E, EmoteId = EI>,
    I: Display,
    EI: Display + ToOwned<Owned = EI>,
    E: Emote<EI>,
{
    let mut iter = history.into_iter();
    let mut emote = None;
    while let Some(id) = iter.next() {
        if let Err(e) = RW::remove_emote(platform_id, &id).await {
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
            let emote = current_emotes.choose(&mut rand::thread_rng());

            if let Some(emote) = emote {
                RW::remove_emote(platform_id, emote.id()).await?;

                emote.id().to_owned()
            } else {
                // this should never happen as this function is only called if there are too many emotes
                log::warn!("Invalid branch - there are no emotes to remove but the limit is reached?! id={}", platform_id);
                return Err(AnyError::msg("There are no emotes to remove"));
            }
        }
    };

    Ok((Some(emote), history))
}

pub async fn update_swap_limit<RW, I, E, EI>(
    broadcaster_id: &str,
    limit: u8,
    pool: &PgPool,
) -> AnyResult<()>
where
    RW: EmoteRW<PlatformId = I, Emote = E, EmoteId = EI>,
    I: Display,
    EI: Display + ToOwned<Owned = EI>,
    E: Emote<EI>,
{
    let limit = limit as usize;
    let (data, id) = RW::get_history_and_platform_id(broadcaster_id, pool).await?;
    if data.len() > limit {
        // remove the last emotes

        // pretend like there are no current emotes.
        let current_emotes = [];
        let mut data = data;
        loop {
            data = remove_last_emote::<RW, _, _, _>(data, &id, &current_emotes)
                .await?
                .1;

            if data.len() <= limit {
                break;
            }
        }
        RW::save_history(broadcaster_id, data, pool).await?;
    }
    Ok(())
}
