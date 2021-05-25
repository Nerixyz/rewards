use crate::models::user::User;
use crate::services::twitch::eventsub::{delete_subscription, subscribe_to_rewards};
use actix_web::Error;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;
use twitch_api2::eventsub::Status;
use twitch_api2::helix::eventsub::{EventSubSubscriptions, GetEventSubSubscriptionsRequest};
use twitch_api2::helix::Response;
use twitch_api2::twitch_oauth2::AppAccessToken;
use twitch_api2::HelixClient;

pub async fn register_eventsub_for_id(
    id: &str,
    token: &Arc<Mutex<AppAccessToken>>,
    pool: &PgPool,
) -> Result<(), Error> {
    let token = token.lock().await;

    let reward = subscribe_to_rewards(&*token, id).await?;

    User::set_eventsub_id(id, &reward.id, pool).await?;

    Ok(())
}

pub async fn unregister_eventsub_for_id(
    id: String,
    token: &Arc<Mutex<AppAccessToken>>,
    pool: &PgPool,
) -> Result<(), Error> {
    let token = token.lock().await;

    User::clear_eventsub_id(&id, pool).await?;

    delete_subscription(&*token, id).await?;

    Ok(())
}

pub async fn register_eventsub_for_all_unregistered(
    token: &Arc<Mutex<AppAccessToken>>,
    pool: &PgPool,
) -> Result<(), anyhow::Error> {
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
) -> Result<(), anyhow::Error> {
    let token = token.lock().await;
    let client = HelixClient::<'_, reqwest::Client>::default();
    let mut rewards: Response<GetEventSubSubscriptionsRequest, EventSubSubscriptions> = client
        .req_get(GetEventSubSubscriptionsRequest::builder().build(), &*token)
        .await?;

    loop {
        for sub in &rewards.data.subscriptions {
            if sub.status != Status::Enabled {
                if let Err(e) = User::clear_eventsub_id(&sub.id, pool).await {
                    println!("Error clearing eventsub in db, but ignoring: {:?}", e);
                }
                if let Err(_) = delete_subscription(&*token, sub.id.clone()).await {
                    // TODO: this returns 200 which is ok but an error in twitch_api2
                    // println!("Error deleting eventsub on twitch, but ignoring: {:?}", e);
                }
            }
        }

        if let Some(_) = rewards.pagination {
            if let Some(res) = rewards.get_next(&client, &*token).await? {
                rewards = res;
                continue;
            }
        }
        break;
    }

    Ok(())
}
