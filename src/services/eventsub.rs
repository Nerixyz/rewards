use crate::constants::SERVER_URL;
use crate::models::reward::RewardToUpdate;
use crate::models::user::User;
use crate::services::twitch::eventsub::{delete_subscription, subscribe_to_rewards};
use crate::services::twitch::RHelixClient;
use actix_web::Result as ActixResult;
use anyhow::Result as AnyhowResult;
use futures::TryStreamExt;
use regex::Regex;
use sqlx::PgPool;
use std::convert::TryInto;
use std::sync::Arc;
use tokio::sync::Mutex;
use twitch_api2::eventsub::Status;
use twitch_api2::helix::eventsub::{EventSubSubscriptions, GetEventSubSubscriptionsRequest};
use twitch_api2::helix::points::{
    CustomRewardRedemption, CustomRewardRedemptionStatus, GetCustomRewardRedemptionRequest,
    UpdateRedemptionStatusBody, UpdateRedemptionStatusRequest,
};
use twitch_api2::helix::Response;
use twitch_api2::twitch_oauth2::{AppAccessToken, UserToken};

pub async fn register_eventsub_for_id(
    id: &str,
    token: &Arc<Mutex<AppAccessToken>>,
    pool: &PgPool,
) -> ActixResult<()> {
    let token = token.lock().await;

    // this clears every subscription so we make sure, there's a new fresh one
    unregister_eventsub_for_user(id, &*token, pool).await.ok();

    let reward = subscribe_to_rewards(&*token, id).await?;

    User::set_eventsub_id(id, &reward.id, pool).await?;

    Ok(())
}

pub async fn unregister_eventsub_for_user(
    id: &str,
    token: &AppAccessToken,
    pool: &PgPool,
) -> ActixResult<()> {
    let old_id = User::clear_eventsub_for_user(id, pool).await?;

    if let Some(old_id) = old_id {
        log::info!("Clearing old subscription id={} user={}", old_id, id);
        delete_subscription(token, old_id).await?;
    }

    Ok(())
}

pub async fn unregister_eventsub_for_id(
    id: String,
    token: &Arc<Mutex<AppAccessToken>>,
    pool: &PgPool,
) -> ActixResult<()> {
    let token = token.lock().await;

    User::clear_eventsub_id(&id, pool).await?;

    delete_subscription(&*token, id).await?;

    Ok(())
}

pub async fn register_eventsub_for_all_unregistered(
    token: &Arc<Mutex<AppAccessToken>>,
    pool: &PgPool,
) -> AnyhowResult<()> {
    let non_subs = User::get_all_non_subscribers(pool).await?;

    for user_id in non_subs {
        register_eventsub_for_id(&user_id, token, pool)
            .await
            .map_err(|_| anyhow::Error::msg("Failed to subscribe to eventsub"))?;
    }

    Ok(())
}

pub async fn clear_invalid_rewards(
    token: &Arc<Mutex<AppAccessToken>>,
    pool: &PgPool,
) -> AnyhowResult<()> {
    let token = token.lock().await;
    let client = RHelixClient::default();
    let mut rewards: Response<GetEventSubSubscriptionsRequest, EventSubSubscriptions> = client
        .req_get(GetEventSubSubscriptionsRequest::builder().build(), &*token)
        .await?;

    loop {
        for sub in &rewards.data.subscriptions {
            // delete subscriptions that are not enabled, that are not from this server (only for ngrok.io)

            let is_enabled = sub.status == Status::Enabled;
            let is_this_server = sub.transport.callback.starts_with(SERVER_URL);

            if !is_enabled || !is_this_server {
                if let Err(e) = User::clear_eventsub_id(&sub.id, pool).await {
                    log::warn!("Error clearing eventsub in db, but ignoring: {:?}", e);
                }
            }
            if !is_enabled
                || (!is_this_server
                    && Regex::new("https?://[\\w_]+.ngrok.io")
                        .unwrap()
                        .is_match(&sub.transport.callback))
            {
                if let Err(e) = delete_subscription(&*token, sub.id.clone()).await {
                    log::warn!("Error deleting eventsub on twitch, but ignoring: {:?}", e);
                }
            }
        }

        if rewards.pagination.is_some() {
            if let Some(res) = rewards.get_next(&client, &*token).await? {
                rewards = res;
                continue;
            }
        }
        break;
    }

    Ok(())
}

pub async fn clear_unfulfilled_redemptions(pool: &PgPool) -> AnyhowResult<()> {
    let mut stream = RewardToUpdate::get_all(pool);
    let client = RHelixClient::default();

    while let Some(reward_with_user) = stream.try_next().await? {
        if let Ok((reward_id, token)) = reward_with_user.try_into() {
            clear_unfulfilled_redemptions_for_id(reward_id, &token, &client).await?;
        }
    }

    Ok(())
}

pub async fn clear_unfulfilled_redemptions_for_id(
    reward_id: String,
    token: &UserToken,
    client: &RHelixClient<'_>,
) -> AnyhowResult<()> {
    let mut rewards: Response<GetCustomRewardRedemptionRequest, Vec<CustomRewardRedemption>> =
        client
            .req_get(
                GetCustomRewardRedemptionRequest::builder()
                    .broadcaster_id(token.user_id.clone())
                    .reward_id(reward_id)
                    .status(Some(CustomRewardRedemptionStatus::Unfulfilled))
                    .build(),
                token,
            )
            .await?;

    loop {
        for redemption in &rewards.data {
            log::info!(
                "Clearing unfulfilled: broadcaster={}; reward_id={}; redemption_id={}",
                redemption.broadcaster_login,
                redemption.reward.id,
                redemption.id,
            );

            if let Err(e) = client
                .req_patch(
                    UpdateRedemptionStatusRequest::builder()
                        .broadcaster_id(redemption.broadcaster_id.clone())
                        .reward_id(redemption.reward.id.clone())
                        .id(redemption.id.clone())
                        .build(),
                    UpdateRedemptionStatusBody::builder()
                        .status(CustomRewardRedemptionStatus::Canceled)
                        .build(),
                    token,
                )
                .await
            {
                log::warn!("Could not update redemption: {}", e);
            }
        }

        if rewards.pagination.is_some() {
            if let Some(res) = rewards.get_next(client, token).await? {
                rewards = res;
                continue;
            }
        }
        break;
    }

    Ok(())
}
