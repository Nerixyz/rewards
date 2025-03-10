use crate::{
    log_err,
    services::twitch::{
        eventsub::{delete_subscription, subscribe_to_rewards},
        RHelixClient,
    },
    util::result::{ResultCExt, ResultExt as _},
};
use actix_web::Result as ActixResult;
use anyhow::Result as AnyhowResult;
use config::CONFIG;
use futures::{FutureExt, TryStreamExt};
use models::{reward::RewardToUpdate, user::User};
use regex::Regex;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use twitch_api::{
    eventsub::{
        channel::ChannelPointsCustomRewardRedemptionAddV1, EventSubscription,
        Status, TransportResponse,
    },
    helix::{
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

pub async fn register_all_eventsub_for_id(
    id: impl AsRef<str>,
    token: &Arc<RwLock<AppAccessToken>>,
    pool: &PgPool,
) -> ActixResult<()> {
    let id = id.as_ref();
    let token = token.read().await;
    register_redemption_eventsub_for_id(id, &token, pool).await?;

    Ok(())
}

async fn register_redemption_eventsub_for_id(
    id: &str,
    token: &AppAccessToken,
    pool: &PgPool,
) -> ActixResult<()> {
    let reward = subscribe_to_rewards(&token, id).await?;

    models::eventsub::add(
        &reward.id,
        id,
        ChannelPointsCustomRewardRedemptionAddV1::EVENT_TYPE.to_str(),
        pool,
    )
    .await
    .err_into()
}

pub async fn unregister_eventsub_for_user(
    id: &str,
    token: &Arc<RwLock<AppAccessToken>>,
    pool: &PgPool,
) -> ActixResult<()> {
    let token = token.read().await;

    let ids = models::eventsub::all_for_user(id, pool).await?;

    futures::future::join_all(
        ids.iter()
            .map(|id| delete_subscription(&token, id).map(|_| ())),
    )
    .await;

    Ok(())
}

pub async fn register_eventsub_for_all_unregistered(
    token: &Arc<RwLock<AppAccessToken>>,
    pool: &PgPool,
) -> AnyhowResult<()> {
    let token = token.read().await;
    register_redemptions_for_unregistered(&token, pool)
        .await
        .log_if_err("register redemptions");

    Ok(())
}

async fn register_redemptions_for_unregistered(
    token: &AppAccessToken,
    pool: &PgPool,
) -> AnyhowResult<()> {
    let non_subs = User::get_all_non_subscribers(
        pool,
        ChannelPointsCustomRewardRedemptionAddV1::EVENT_TYPE.to_str(),
    )
    .await?;

    for user_id in non_subs {
        register_redemption_eventsub_for_id(&user_id, token, pool)
            .await
            .inspect_err(|e| {
                log::warn!("Failed to register redemptions for {user_id}: {e}")
            })
            .ok();
    }

    Ok(())
}

pub async fn clear_invalid_rewards(
    token: &Arc<RwLock<AppAccessToken>>,
    pool: &PgPool,
) -> AnyhowResult<()> {
    let token = token.read().await;
    let client = RHelixClient::default();

    let ng_re = Regex::new("https?://[\\w_-]+(:?\\.\\w+)?.ngrok.io").unwrap();

    let mut stream =
        client.get_eventsub_subscriptions(None, None, None, &*token);
    while let Some(subs) = stream.try_next().await? {
        for sub in subs.subscriptions {
            // delete subscriptions that are not enabled, that are not from this server (only for ngrok.io)

            let TransportResponse::Webhook(transport) = &sub.transport else {
                continue; // websocket
            };

            let is_enabled = sub.status == Status::Enabled;
            let is_this_server =
                transport.callback.starts_with(&CONFIG.server.url);

            if !is_enabled || !is_this_server {
                models::eventsub::remove(sub.id.as_ref(), pool)
                    .await
                    .dbg_if_err("clearing eventsub in db");
            }
            if !is_enabled
                || (!is_this_server && ng_re.is_match(&transport.callback))
            {
                delete_subscription(&token, sub.id.clone())
                    .await
                    .log_if_err("deleting eventsub on twitch");
            }
        }
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
