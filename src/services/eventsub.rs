use crate::{
    services::twitch::{eventsub::delete_subscription, RHelixClient},
    util::result::{ResultCExt, ResultExt as _},
};
use actix_web::Result as ActixResult;
use anyhow::Result as AnyhowResult;
use config::CONFIG;
use futures::{FutureExt, TryStreamExt};
use models::user::User;
use regex::Regex;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use twitch_api::{
    eventsub::{
        channel::{
            ChannelModerateV2, ChannelPointsCustomRewardRedemptionAddV1,
        },
        stream::StreamOnlineV1,
        EventSubscription, Status, TransportResponse,
    },
    helix::eventsub::CreateEventSubSubscription,
    twitch_oauth2::AppAccessToken,
};

use super::twitch::HelixResult;

trait KnownSub: EventSubscription {
    async fn subscribe(
        id: &str,
        token: &AppAccessToken,
    ) -> HelixResult<CreateEventSubSubscription<Self>>;
}

impl KnownSub for ChannelPointsCustomRewardRedemptionAddV1 {
    async fn subscribe(
        id: &str,
        token: &AppAccessToken,
    ) -> HelixResult<CreateEventSubSubscription<Self>> {
        super::twitch::eventsub::subscribe_to(
            token,
            Self::broadcaster_user_id(id),
        )
        .await
    }
}

impl KnownSub for StreamOnlineV1 {
    async fn subscribe(
        id: &str,
        token: &AppAccessToken,
    ) -> HelixResult<CreateEventSubSubscription<Self>> {
        super::twitch::eventsub::subscribe_to(
            token,
            Self::broadcaster_user_id(id),
        )
        .await
    }
}

impl KnownSub for ChannelModerateV2 {
    async fn subscribe(
        id: &str,
        token: &AppAccessToken,
    ) -> HelixResult<CreateEventSubSubscription<Self>> {
        super::twitch::eventsub::subscribe_to(
            token,
            Self::new(id, CONFIG.twitch.user_id.as_str()),
        )
        .await
    }
}

async fn register_for_id<S: KnownSub>(
    id: &str,
    token: &AppAccessToken,
    pool: &PgPool,
) -> ActixResult<()> {
    let sub = S::subscribe(id, token).await?;

    models::eventsub::add(&sub.id, id, S::EVENT_TYPE.to_str(), pool)
        .await
        .err_into()
}

pub async fn register_all_eventsub_for_id(
    id: impl AsRef<str>,
    token: &Arc<RwLock<AppAccessToken>>,
    pool: &PgPool,
) -> ActixResult<()> {
    let id = id.as_ref();
    let token = token.read().await;

    // points
    register_for_id::<ChannelPointsCustomRewardRedemptionAddV1>(
        id, &token, pool,
    )
    .await?;
    // stream.online
    register_for_id::<StreamOnlineV1>(id, &token, pool)
        .await
        .log_if_err("register stream online");
    // channel.moderate
    register_for_id::<ChannelModerateV2>(id, &token, pool)
        .await
        .log_if_err("register channel moderate");

    Ok(())
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

    // points
    register_for_unregistered::<ChannelPointsCustomRewardRedemptionAddV1>(
        &token, pool,
    )
    .await
    .log_if_err("register redemptions");

    // stream.online
    register_for_unregistered::<StreamOnlineV1>(&token, pool)
        .await
        .log_if_err("register streams");

    // channel.moderate
    register_for_unregistered::<ChannelModerateV2>(&token, pool)
        .await
        .log_if_err("register channel.moderate");

    Ok(())
}

async fn register_for_unregistered<S: KnownSub>(
    token: &AppAccessToken,
    pool: &PgPool,
) -> AnyhowResult<()> {
    let non_subs =
        User::get_all_non_subscribers(pool, S::EVENT_TYPE.to_str()).await?;

    for user_id in non_subs {
        register_for_id::<S>(&user_id, token, pool)
            .await
            .inspect_err(|e| {
                log::warn!(
                    "Register {} for {user_id}: {e}",
                    S::EVENT_TYPE.to_str()
                )
            })
            .ok();
    }

    Ok(())
}

pub async fn clear_invalid_subs(
    token: &Arc<RwLock<AppAccessToken>>,
    pool: &PgPool,
) -> AnyhowResult<()> {
    let token = token.read().await;
    let client = RHelixClient::default();

    let ng_re = Regex::new("https?://[\\w_-]+(:?\\.\\w+)?.ngrok.io").unwrap();

    let mut stream = client
        .get_eventsub_subscriptions(None, None, None, &*token)
        .map_ok(|it| {
            futures::stream::iter(
                it.subscriptions
                    .into_iter()
                    .map(Ok::<_, twitch_api::helix::ClientRequestError<_>>),
            )
        })
        .try_flatten();
    log::info!("stream");
    let mut n = 0;
    while let Some(sub) = stream.try_next().await? {
        n += 1;
        // delete subscriptions that are not enabled, that are not from this server (only for ngrok.io)

        let TransportResponse::Webhook(transport) = &sub.transport else {
            dbg!(&sub);
            continue; // websocket
        };

        let is_enabled = sub.status == Status::Enabled;
        if !is_enabled {
            dbg!(&sub.id);
        }
        let is_this_server = transport.callback.starts_with(&CONFIG.server.url);

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
    log::info!("stream over {n}");

    Ok(())
}
