use crate::{
    log_err,
    services::{
        emotes::{Emote, EmoteId, EmoteRW},
        text::trim_to,
        twitch::requests::update_reward,
    },
};
use anyhow::{Error as AnyError, Result as AnyResult};
use chrono::{Duration, Utc};
use futures::TryFutureExt;
use models::{
    banned_emote, log_entry::LogEntry, reward::SlotRewardData, slot::Slot,
    user::User,
};
use sqlx::PgPool;
use std::{cmp::Ordering, fmt::Display};
use twitch_api2::{
    helix::points::UpdateCustomRewardBody, twitch_oauth2::UserToken,
};

pub async fn adjust_size<RW, I, E, EI>(
    broadcaster_id: &str,
    platform_id: &I,
    reward_id: &str,
    n_slots: usize,
    pool: &PgPool,
) -> AnyResult<()>
where
    RW: EmoteRW<PlatformId = I, Emote = E, EmoteId = EI>,
    EI: EmoteId,
    E: Emote<EI>,
{
    if n_slots == 0 {
        return Err(AnyError::msg("You can't have 0 slots"));
    }
    let mut current =
        Slot::get_all_slots(broadcaster_id, reward_id, pool).await?;

    match current.len() {
        n_current_emotes if n_current_emotes > n_slots => {
            // delete slots
            current.sort_by(|a, b| match (&a.emote_id, &b.emote_id) {
                (Some(_), Some(_)) | (None, None) => Ordering::Equal,
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
            });

            let n_to_delete = current.len() - n_slots;
            let to_delete: Vec<Slot> =
                current.into_iter().take(n_to_delete).collect();
            for emote in to_delete.iter().filter(|e| e.emote_id.is_some()) {
                if let Err(e) = RW::remove_emote(
                    platform_id,
                    &EI::from_db(
                        emote
                            .emote_id
                            .as_ref()
                            .ok_or_else(|| AnyError::msg("never"))?,
                    )?,
                )
                .await
                {
                    log::warn!(
                        "Couldn't delete {:?} error={}",
                        emote.emote_id,
                        e
                    );
                }
            }
            for row in to_delete {
                // no WHERE in $1 saj
                Slot::remove(row.id, pool).await?
            }
        }
        n_current_emotes if n_current_emotes < n_slots => {
            // create slots
            let (env, available_slots) = futures::future::try_join(
                RW::get_emote_env_data(broadcaster_id, platform_id),
                Slot::get_available_slots(broadcaster_id, reward_id, pool)
                    .map_err(|e| {
                        log::warn!("Could not get slots {}", e);
                        AnyError::msg("Internal error")
                    }),
            )
            .await?;

            let current_free = env.max_emotes - env.current_emotes;
            let known_free = available_slots.len();

            if current_free <= known_free {
                // something else changed - too few slots
                return Err(AnyError::msg(format!(
                    "There are {} free slots on {:?}, {} slots are known to be available",
                    current_free,
                    RW::platform(),
                    known_free
                )));
            }

            let actually_free = current_free - known_free;
            let needed_slots = n_slots - current.len();
            if actually_free < needed_slots {
                return Err(AnyError::msg(format!("There are {} free slots on {:?} (actually {} because {} are already uses as slots here) but I need {}",
                                                 current_free, RW::platform(), actually_free, known_free, needed_slots)));
            }

            for _ in 0..needed_slots {
                Slot::create(broadcaster_id, reward_id, RW::platform(), pool)
                    .await?;
            }
        }
        _ => (),
    };

    // update reward status
    let token = User::get_by_id(broadcaster_id, pool).await?.into();
    update_reward(
        broadcaster_id,
        reward_id.to_string(),
        UpdateCustomRewardBody::builder()
            .is_paused(Some(
                Slot::get_n_available_slots(broadcaster_id, reward_id, pool)
                    .await?
                    <= 0,
            ))
            .build(),
        &token,
    )
    .await?;

    Ok(())
}

pub async fn add_slot_emote<RW, I, E, EI>(
    broadcaster_id: &str,
    reward_id: &str,
    slot_data: SlotRewardData,
    emote_id: &str,
    redeemed_user_login: &str,
    pool: &PgPool,
) -> AnyResult<(String, usize)>
where
    RW: EmoteRW<PlatformId = I, EmoteId = EI, Emote = E>,
    E: Emote<EI>,
    EI: Display,
{
    if banned_emote::is_banned(broadcaster_id, emote_id, RW::platform(), pool)
        .await?
    {
        return Err(AnyError::msg("This emote is banned"));
    }
    let available_slots =
        Slot::get_available_slots(broadcaster_id, reward_id, pool)
            .await
            .map_err(|e| {
                log::warn!("Could not query: {}", e);
                AnyError::msg("Internal error")
            })?;
    let n_available = available_slots.len();
    let mut slot = available_slots
        .into_iter()
        .next()
        .ok_or_else(|| AnyError::msg("No free slot is available!"))?;
    let emote_data =
        RW::get_check_initial_data(broadcaster_id, emote_id, pool).await?;

    if emote_data.current_emotes >= emote_data.max_emotes {
        return Err(AnyError::msg("There's no free slot!"));
    }

    RW::add_emote(&emote_data.platform_id, emote_data.emote.id())
        .await
        .map_err(|e| {
            log::warn!("Could not add: {}", e);
            AnyError::msg(trim_to(format!("Couldn't add emote: {}", e), 200))
        })?;

    let expiration = humantime::parse_duration(&slot_data.expiration)
        .map_err(|_| AnyError::msg("No expiration set!"))?;
    slot.emote_id = Some(emote_data.emote.id().to_string());
    let now = Utc::now();
    slot.expires = Some(
        now + Duration::from_std(expiration).map_err(|e| {
            log::warn!("Could not add duration: {}", e);
            AnyError::msg("Could not add duration lole")
        })?,
    );
    let emote_name = emote_data.emote.into_name();
    slot.name = Some(emote_name.clone());
    slot.added_by = Some(redeemed_user_login.to_string());
    slot.added_at = Some(now);

    slot.update(pool).await.map_err(|e| {
        log::warn!("Failed to update reward-slot: {}", e);
        AnyError::msg("Internal error")
    })?;

    // disable reward if all slots are full now
    if n_available == 1 {
        log::info!("Disabling {} because all slots are filled", reward_id);

        let this_user =
            User::get_by_id(broadcaster_id, pool).await.map_err(|e| {
                log::warn!("Could not get user: {}", e);
                AnyError::msg("Internal error")
            })?;

        let token: UserToken = this_user.into();

        log_err!(
            update_reward(
                token.user_id.clone(),
                reward_id.to_string(),
                UpdateCustomRewardBody::builder()
                    .is_paused(Some(true))
                    .build(),
                &token
            )
            .await,
            "Failed to update reward"
        );
    }

    // TODO: log::info
    log_err!(
        LogEntry::create(
            broadcaster_id,
            &format!(
                "[slots::{:?}] Added {}; slots-open={}; expires={:?}; redeemed={}; slot_id={}",
                RW::platform(),
                emote_name,
                n_available - 1,
                slot.expires.map(|exp| exp.to_string()),
                redeemed_user_login,
                slot.id
            ),
            pool
        )
        .await,
        "Could not create log-entry"
    );
    Ok((emote_name, n_available - 1))
}
