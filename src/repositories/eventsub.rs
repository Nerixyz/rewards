use actix::Addr;
use actix_web::{
    post,
    web::{self, ServiceConfig},
    HttpResponse, Result,
};
use sqlx::PgPool;
use twitch_api::eventsub::{
    user::UserAuthorizationRevokeV1Payload, Event, Message, Payload,
};

use crate::{
    actors::rewards::RewardsActor,
    extractors::eventsub::EventsubPayload,
    log_discord,
    services::rewards::{
        redemption,
        redemption::{ReceiveRedemptionCtx, ReceiveRedemptionError},
    },
};
use models::user::User;

#[post("/reward")]
async fn reward_redemption(
    pool: web::Data<PgPool>,
    payload: EventsubPayload,
    executor: web::Data<Addr<RewardsActor>>,
) -> Result<HttpResponse> {
    match payload.0 {
        Event::ChannelPointsCustomRewardRedemptionAddV1(Payload {
            message: Message::VerificationRequest(req),
            subscription,
            ..
        }) => {
            log::info!("verification for sub: {:?}", subscription);
            Ok(HttpResponse::Ok().body(req.challenge))
        }
        Event::ChannelPointsCustomRewardRedemptionAddV1(Payload {
            message: Message::Notification(notification),
            subscription,
            ..
        }) => {
            // main path
            let user = User::get_by_id(
                notification.broadcaster_user_id.as_ref(),
                &pool,
            )
            .await?;

            log::info!(
                "redemption: {:?} - sub: {:?}",
                notification,
                subscription
            );

            let ctx = ReceiveRedemptionCtx {
                pool: pool.into_inner(),
                executor: executor.into_inner(),
                user,
                notification,
            };
            actix_web::rt::spawn(async move {
                match redemption::receive(ctx).await {
                    Ok(_) => (),
                    Err(ReceiveRedemptionError::NoReward) => (),
                }
            });

            Ok(HttpResponse::Ok().finish())
        }
        Event::UserAuthorizationRevokeV1(re) => {
            log::warn!("auth revoke: {:?}", re);
            log_discord!(
                "Auth",
                "Unhandled revocation",
                "User Login/Id" = match re.message {
                    Message::Notification(
                        UserAuthorizationRevokeV1Payload {
                            user_name: Some(login),
                            ..
                        },
                    ) => login.take(),
                    Message::Notification(
                        UserAuthorizationRevokeV1Payload { user_id, .. },
                    ) => user_id.take(),
                    _ => "no login or id".to_string(),
                }
            );
            // TODO
            Ok(HttpResponse::Ok().finish())
        }
        other => {
            log::warn!("unknown payload: {:?}", other);
            Ok(HttpResponse::Ok().body("I can't handle that!"))
        }
    }
}

pub fn init_eventsub_routes(config: &mut ServiceConfig) {
    config.service(reward_redemption);
}
