use crate::{
    log_err,
    services::twitch::{
        eventsub::{delete_subscription, subscribe_to_rewards},
        RHelixClient,
    },
};
use actix_web::Result as ActixResult;
use anyhow::Result as AnyhowResult;
use config::CONFIG;
use futures::TryStreamExt;
use models::{reward::RewardToUpdate, user::User};
use regex::Regex;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use twitch_api::{
    eventsub::{Status, TransportResponse},
    helix::{
        eventsub::{EventSubSubscriptions, GetEventSubSubscriptionsRequest},
        points::{
            CustomRewardRedemption, CustomRewardRedemptionStatus,
            GetCustomRewardRedemptionRequest, UpdateRedemptionStatusBody,
            UpdateRedemptionStatusRequest,
        },
        Response,
    },
    twitch_oauth2::{AppAccessToken, UserToken},
    types::{IntoCow, RewardIdRef},
};

pub async fn register_eventsub_for_id(
    id: impl AsRef<str>,
    token: &Arc<RwLock<AppAccessToken>>,
    pool: &PgPool,
) -> ActixResult<()> {
    let id = id.as_ref();
    let token = token.read().await;

    // this clears every subscription so we make sure, there's a new fresh one
    unregister_eventsub_for_user(id, &token, pool).await.ok();

    let reward = subscribe_to_rewards(&token, id).await?;

    User::set_eventsub_id(id, &reward.id.take(), pool).await?;

    Ok(())
}

pub async fn unregister_eventsub_for_user(
    id: impl AsRef<str>,
    token: &AppAccessToken,
    pool: &PgPool,
) -> ActixResult<()> {
    let old_id = User::clear_eventsub_for_user(id.as_ref(), pool).await?;

    if let Some(old_id) = old_id {
        log::info!(
            "Clearing old subscription id={} user={}",
            old_id,
            id.as_ref()
        );
        delete_subscription(token, old_id).await?;
    }

    Ok(())
}

pub async fn unregister_eventsub_for_id(
    id: String,
    token: &Arc<RwLock<AppAccessToken>>,
    pool: &PgPool,
) -> ActixResult<()> {
    let token = token.read().await;

    User::clear_eventsub_id(&id, pool).await?;

    delete_subscription(&token, id).await?;

    Ok(())
}

pub async fn register_eventsub_for_all_unregistered(
    token: &Arc<RwLock<AppAccessToken>>,
    pool: &PgPool,
) -> AnyhowResult<()> {
    let non_subs = User::get_all_non_subscribers(pool).await?;

    for user_id in non_subs {
        log_err!(
            register_eventsub_for_id(&user_id, token, pool).await,
            "Could not register eventsub for id {} - {}",
            user_id
        );
    }

    Ok(())
}

pub async fn clear_invalid_rewards(
    token: &Arc<RwLock<AppAccessToken>>,
    pool: &PgPool,
) -> AnyhowResult<()> {
    let token = token.read().await;
    let client = RHelixClient::default();
    let mut rewards: Response<
        GetEventSubSubscriptionsRequest,
        EventSubSubscriptions,
    > = client
        .req_get(GetEventSubSubscriptionsRequest::default(), &*token)
        .await?;

    loop {
        for sub in &rewards.data.subscriptions {
            // delete subscriptions that are not enabled, that are not from this server (only for ngrok.io)

            let TransportResponse::Webhook(transport) = &sub.transport else {
                continue; // websocket
            };

            let is_enabled = sub.status == Status::Enabled;
            let is_this_server =
                transport.callback.starts_with(&CONFIG.server.url);

            if !is_enabled || !is_this_server {
                if let Err(e) =
                    User::clear_eventsub_id(sub.id.as_ref(), pool).await
                {
                    log::warn!(
                        "Error clearing eventsub in db, but ignoring: {:?}",
                        e
                    );
                }
            }
            if !is_enabled
                || (!is_this_server
                    && Regex::new("https?://[\\w_-]+(:?\\.\\w+)?.ngrok.io")
                        .unwrap()
                        .is_match(&transport.callback))
            {
                if let Err(e) =
                    delete_subscription(&token, sub.id.clone()).await
                {
                    log::warn!(
                        "Error deleting eventsub on twitch, but ignoring: {:?}",
                        e
                    );
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
        let (reward_id, token) = reward_with_user.into();
        log_err!(
            clear_unfulfilled_redemptions_for_id(reward_id, &token, &client)
                .await,
            "Could not clear redemptions for id"
        );
    }

    Ok(())
}

pub async fn clear_unfulfilled_redemptions_for_id<'a>(
    reward_id: impl IntoCow<'a, RewardIdRef> + 'a,
    token: &'a UserToken,
    client: &RHelixClient<'_>,
) -> AnyhowResult<()> {
    let mut rewards: Response<
        GetCustomRewardRedemptionRequest,
        Vec<CustomRewardRedemption>,
    > = client
        .req_get(
            GetCustomRewardRedemptionRequest::broadcaster_id(
                token.user_id.as_str(),
            )
            .reward_id(reward_id)
            .status(CustomRewardRedemptionStatus::Unfulfilled),
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
