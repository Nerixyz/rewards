use crate::{
    log_err,
    services::{
        emotes::{Emote, EmoteRW},
        text::trim_to,
    },
};
use anyhow::{Error as AnyError, Result as AnyResult};
use models::{banned_emote, reward::SwapRewardData, swap_emote::SwapEmote};
use sqlx::PgPool;
use std::{fmt::Display, str::FromStr};

pub async fn swap_or_add_emote<RW>(
    broadcaster_id: &str,
    emote_id: &str,
    reward_data: SwapRewardData,
    executing_user: &str,
    pool: &PgPool,
) -> AnyResult<(Option<String>, String)>
where
    RW: EmoteRW,
    RW::PlatformId: Display,
    RW::Emote: Emote<RW::EmoteId>,
    RW::EmoteId: Display + ToOwned<Owned = RW::EmoteId> + FromStr + Default,
{
    if banned_emote::is_banned(broadcaster_id, emote_id, RW::platform(), pool)
        .await?
    {
        return Err(AnyError::msg("This emote is banned"));
    }
    let data = RW::get_check_initial_data(
        broadcaster_id,
        emote_id,
        reward_data.allow_unlisted,
        pool,
    )
    .await?;
    let removed_emote = if data.current_emotes >= data.max_emotes
        || reward_data
            .limit
            .map(|l| data.history_len >= l as usize)
            .unwrap_or(false)
    {
        Some(
            remove_last_emote::<RW>(broadcaster_id, &data.platform_id, pool)
                .await
                .0?,
        )
    } else {
        None
    };

    log::info!(
        "Add emote_id={} to platform_id={}",
        data.emote.id(),
        data.platform_id
    );
    if let Err(e) = RW::add_emote(&data.platform_id, data.emote.id()).await {
        log::warn!("Could not add emote: {}", e);
        return Err(AnyError::msg(trim_to(
            format!("Couldn't add emote: {}", e),
            200,
        )));
    }
    SwapEmote::add(
        broadcaster_id,
        &data.emote.id().to_string(),
        RW::platform(),
        data.emote.name(),
        executing_user,
        pool,
    )
    .await
    .map_err(|_| AnyError::msg("Could not save emote in DB"))?;

    Ok((removed_emote, data.emote.into_name()))
}

pub async fn remove_last_emote<RW>(
    user_id: &str,
    platform_id: &RW::PlatformId,
    pool: &PgPool,
) -> (AnyResult<String>, usize)
where
    RW: EmoteRW,
    RW::PlatformId: Display,
    RW::Emote: Emote<RW::EmoteId>,
    RW::EmoteId: Display + ToOwned<Owned = RW::EmoteId> + FromStr + Default,
{
    let mut emote = None;
    let mut removed_from_db = 0;
    while let Ok(Some(db_emote)) =
        SwapEmote::oldest(user_id, RW::platform(), pool).await
    {
        let actually_removed = if let Err(e) = RW::remove_emote(
            platform_id,
            &RW::EmoteId::from_str(&db_emote.emote_id).unwrap_or_default(),
        )
        .await
        {
            log::info!("Skipping emote: {:?}; error={}", db_emote, e);
            false
        } else {
            true
        };
        log_err!(
            SwapEmote::remove(db_emote.id, pool).await,
            "Failed to remove a swap emote even though we just got the id"
        );
        if actually_removed {
            emote = Some(db_emote);
            removed_from_db += 1;
            break;
        }
    }

    (
        emote.map(|e| e.name).ok_or_else(|| {
            log::info!("Could not remove any emotes in {}.", user_id);
            AnyError::msg(
                "There are no recent emotes to remove - refusing to remove random emote.",
            )
        }),
        removed_from_db,
    )
}

pub async fn update_swap_limit<RW>(
    broadcaster_id: &str,
    limit: u8,
    pool: &PgPool,
) -> AnyResult<()>
where
    RW: EmoteRW,
    RW::PlatformId: Display,
    RW::Emote: Emote<RW::EmoteId>,
    RW::EmoteId: Display + ToOwned<Owned = RW::EmoteId> + FromStr + Default,
{
    let limit = limit as usize;
    let mut current_emotes =
        SwapEmote::emote_count(broadcaster_id, RW::platform(), pool).await?
            as usize;
    if current_emotes > limit {
        let platform_id = RW::get_platform_id(broadcaster_id, pool).await?;
        // remove the last emotes
        loop {
            let (res, removed) =
                remove_last_emote::<RW>(broadcaster_id, &platform_id, pool)
                    .await;
            current_emotes -= removed;
            let _ = res?;

            if current_emotes <= limit {
                break;
            }
        }
    }
    Ok(())
}
