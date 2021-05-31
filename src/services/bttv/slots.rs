use crate::models::bttv_slot::BttvSlot;
use crate::models::reward::BttvSlotRewardData;
use crate::models::user::User;
use crate::services::bttv::requests::{delete_shared_emote, get_user};
use crate::services::bttv::{self, get_user_limits, prepare_add_emote};
use crate::services::twitch::requests::update_reward;
use anyhow::{Error as AnyError, Result as AnyResult};
use chrono::{Duration, Utc};
use futures::TryFutureExt;
use sqlx::PgPool;
use std::cmp::Ordering;
use twitch_api2::helix::points::UpdateCustomRewardBody;
use twitch_api2::twitch_oauth2::UserToken;

pub async fn adjust_size(
    user_id: &str,
    bttv_id: &str,
    reward_id: &str,
    n_slots: usize,
    pool: &PgPool,
) -> AnyResult<()> {
    if n_slots == 0 {
        return Err(AnyError::msg("You can't have 0 slots"));
    }
    let mut current = BttvSlot::get_all_slots(user_id, reward_id, pool).await?;

    match current.len() {
        n_current_emotes if n_current_emotes > n_slots => {
            // delete slots
            current.sort_by(|a, b| match (&a.emote_id, &b.emote_id) {
                (Some(_), Some(_)) | (None, None) => Ordering::Equal,
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
            });

            let n_to_delete = current.len() - n_slots;
            let to_delete: Vec<BttvSlot> = current.into_iter().take(n_to_delete).collect();
            for emote in to_delete.iter().filter(|e| e.emote_id.is_some()) {
                if let Err(e) = delete_shared_emote(
                    emote
                        .emote_id
                        .as_ref()
                        .ok_or_else(|| AnyError::msg("never"))?,
                    bttv_id,
                )
                .await
                {
                    log::warn!("Couldn't delete {:?} error={}", emote.emote_id, e);
                }
            }
            for row in to_delete {
                // no WHERE in $1 saj
                BttvSlot::remove(row.id, &pool).await?
            }
        }
        n_current_emotes if n_current_emotes < n_slots => {
            // create slots
            let (bttv_limits, shared_emotes, available_slots) = futures::future::try_join3(
                get_user_limits(bttv_id),
                get_user(bttv_id),
                BttvSlot::get_available_slots(user_id, reward_id, pool).map_err(|e| {
                    log::warn!("Could not get slots {}", e);
                    AnyError::msg("Internal error")
                }),
            )
            .await?;

            let current_free = bttv_limits.shared_emotes - shared_emotes.shared_emotes.len();
            let known_free = available_slots.len();

            if current_free <= known_free {
                // something else changed - too few slots
                return Err(AnyError::msg(format!(
                    "There are {} free slots on bttv, {} slots are known to be available",
                    current_free, known_free
                )));
            }

            let actually_free = current_free - known_free;
            let needed_slots = n_slots - current.len();
            if actually_free < needed_slots {
                return Err(AnyError::msg(format!("There are {} free slots on bttv (actually {} because {} are already uses as slots here) but I need {}",
                                                 current_free, actually_free, known_free, needed_slots)));
            }

            for _ in 0..needed_slots {
                BttvSlot::create(user_id, reward_id, pool).await?;
            }
        }
        _ => (),
    };

    Ok(())
}

pub async fn add_emote(
    user_id: &str,
    reward_id: &str,
    data: BttvSlotRewardData,
    emote_id: &str,
    pool: &PgPool,
) -> AnyResult<(String, usize)> {
    let available_slots = BttvSlot::get_available_slots(user_id, reward_id, pool)
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
    let (_this_user, bttv_user, user_limits, emote_data) =
        prepare_add_emote(user_id, emote_id, pool).await?;

    if bttv_user.shared_emotes.len() >= user_limits.shared_emotes {
        return Err(AnyError::msg("There's no free slot!"));
    }

    bttv::requests::add_shared_emote(emote_id, &bttv_user.id)
        .await
        .map_err(|e| {
            log::warn!("Could not add: {}", e);
            AnyError::msg("Couldn't add emote")
        })?;

    let expiration = humantime::parse_duration(&data.expiration)
        .map_err(|_| AnyError::msg("No expiration set!"))?;
    slot.emote_id = Some(emote_id.to_string());
    slot.expires = Some(
        Utc::now()
            + Duration::from_std(expiration).map_err(|e| {
                log::warn!("Could not add duration: {}", e);
                AnyError::msg("Could not add duration lole")
            })?,
    );

    slot.update(pool).await.map_err(|e| {
        log::warn!("Failed to update reward-slot: {}", e);
        AnyError::msg("Internal error")
    })?;

    // disable reward if all slots are full now
    if n_available == 1 {
        log::info!("Disabling {} because all slots are filled", reward_id);

        let this_user = User::get_by_id(user_id, pool).await.map_err(|e| {
            log::warn!("Could not get user: {}", e);
            AnyError::msg("Internal error")
        })?;

        let token: UserToken = this_user.into();
        if let Err(e) = update_reward(
            &token.user_id,
            reward_id.to_string(),
            UpdateCustomRewardBody::builder()
                .is_enabled(Some(false))
                .build(),
            &token,
        )
        .await
        {
            log::warn!("Could not disable reward: {}", e);
        }
    }

    Ok((emote_data.code, n_available - 1))
}
