use std::time::Duration;

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
    actors::{
        live::{self, LiveActor},
        rewards::RewardsActor,
        timeout::{self, TimeoutActor},
    },
    extractors::eventsub::EventsubPayload,
    log_discord,
    services::rewards::redemption::{
        self, ReceiveRedemptionCtx, ReceiveRedemptionError,
    },
};
use models::user::User;

#[post("/reward")]
async fn reward_redemption(
    pool: web::Data<PgPool>,
    payload: EventsubPayload,
    executor: web::Data<Addr<RewardsActor>>,
    live_actor: web::Data<Addr<LiveActor>>,
    timeout_actor: web::Data<Addr<TimeoutActor>>,
) -> Result<HttpResponse> {
    dbg!(&payload.0);

    match payload.0 {
        // verification
        Event::ChannelPointsCustomRewardRedemptionAddV1(Payload {
            message: Message::VerificationRequest(req),
            ..
        })
        | Event::StreamOnlineV1(Payload {
            message: Message::VerificationRequest(req),
            ..
        })
        | Event::StreamOfflineV1(Payload {
            message: Message::VerificationRequest(req),
            ..
        })
        | Event::ChannelModerateV2(Payload {
            message: Message::VerificationRequest(req),
            ..
        }) => Ok(HttpResponse::Ok().body(req.challenge)),

        // actual events
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

        Event::StreamOnlineV1(Payload {
            message: Message::Notification(notification),
            ..
        }) => {
            live_actor.do_send(live::LiveMessage(
                notification.broadcaster_user_id.take(),
            ));
            Ok(HttpResponse::Ok().finish())
        }
        Event::StreamOfflineV1(Payload {
            message: Message::Notification(notification),
            ..
        }) => {
            live_actor.do_send(live::OfflineMessage(
                notification.broadcaster_user_id.take(),
            ));
            Ok(HttpResponse::Ok().finish())
        }

        Event::ChannelModerateV2(Payload {
            message: Message::Notification(notification),
            ..
        }) => {
            match notification.action {
                twitch_api::eventsub::channel::moderate::ActionV2::Untimeout(untimeout) => {
                    timeout_actor
                        .do_send(timeout::RemoveTimeoutMessage {
                            channel_id: notification.broadcaster_user_id.take(),
                            user_id: untimeout.user_id.take(),
                            later: Duration::from_secs(0),
                        });
                },
                _ => (),
            }
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
