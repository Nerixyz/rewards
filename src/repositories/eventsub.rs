use std::time::{Duration, Instant};

use actix::Addr;
use actix_web::{
    post,
    web::{self, ServiceConfig},
    HttpResponse, Result,
};
use sqlx::PgPool;
use twitch_api2::{
    eventsub,
    eventsub::{user::UserAuthorizationRevokeV1Payload, Event, Message, Payload},
    helix::points::CustomRewardRedemptionStatus,
};

use crate::{
    actors::{
        irc::{IrcActor, WhisperMessage},
        rewards::{ExecuteRewardMessage, RewardsActor},
    },
    log_discord,
    models::{reward::Reward, user::User},
    services::twitch::eventsub::update_reward_redemption,
};

#[post("/reward")]
async fn reward_redemption(
    pool: web::Data<PgPool>,
    irc: web::Data<Addr<IrcActor>>,
    payload: web::Json<eventsub::Event>,
    executor: web::Data<Addr<RewardsActor>>,
) -> Result<HttpResponse> {
    match payload.into_inner() {
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
            let user = User::get_by_id(notification.broadcaster_user_id.as_ref(), &pool).await?;

            log::info!("redemption: {:?} - sub: {:?}", notification, subscription);

            let pool = pool.into_inner();
            let irc = irc.into_inner();
            let executor = executor.into_inner();
            let redemption_received = Instant::now();
            actix_web::rt::spawn(async move {
                let reward = Reward::get_by_id(notification.reward.id.as_ref(), &pool).await;

                let reward = match reward {
                    Ok(r) => r,
                    Err(_) => {
                        log::warn!(
                            "failed to get user or reward: userId: {}, rewardID: {}",
                            notification.broadcaster_user_id,
                            notification.reward.id
                        );
                        return;
                    }
                };

                let broadcaster_id: String = notification.broadcaster_user_id.clone().into_string();
                let reward_id: String = notification.reward.id.clone().into_string();
                let redemption_id: String = notification.id.clone().into_string();

                let executing_user_login: String = notification.user_name.clone().into_string();
                let broadcaster_login: String =
                    notification.broadcaster_user_login.clone().into_string();
                let reward_name: String = notification.reward.title.clone();
                let reward_type = reward.data.0.to_string();
                let user_input = notification.user_input.clone();

                let status = match executor
                    .send(ExecuteRewardMessage {
                        redemption: notification,
                        subscription,
                        broadcaster: user.clone(),
                        reward,
                    })
                    .await
                {
                    Ok(Ok(_)) => CustomRewardRedemptionStatus::Fulfilled,
                    e => {
                        let (debug, display) = match e {
                            Err(e) => (format!("{:?}", e), e.to_string()),
                            Ok(Err(e)) => (format!("{:?}", e), e.to_string()),
                            Ok(Ok(_)) => unreachable!(),
                        };

                        log::warn!("Could not execute reward: {:?}", debug);

                        log_discord!(
                            "Rewards",
                            format!("⚠ Failed to execute reward in {}", broadcaster_login),
                            0xfab43e,
                            "Reward" = reward_name.clone(),
                            "Type" = reward_type.clone(),
                            "Error" = display
                        );

                        match irc.send(WhisperMessage(executing_user_login.clone(), "[Refund] ⚠ I could not execute the reward. Make sure you provided the correct input!".to_string())).await {
                            Err(e) => log::warn!("MailboxError on sending chat: {}", e),
                            Ok(Err(e)) => log::warn!("Error sending chat: {}", e),
                            _ => ()
                        }

                        CustomRewardRedemptionStatus::Canceled
                    }
                };
                // here, the redemption is finally updated, so we'll log this
                metrics::increment_counter!("rewards_redemptions",
                    "status" => if status == CustomRewardRedemptionStatus::Fulfilled { "fulfilled" } else { "cancelled" },
                    "reward" => reward_type.clone()
                );
                let execution = Instant::now()
                    .checked_duration_since(redemption_received)
                    .unwrap_or_else(|| Duration::from_secs(0));
                metrics::histogram!("rewards_redemption_execution_duration",
                    execution.as_secs_f64(),
                    "status" => if status == CustomRewardRedemptionStatus::Fulfilled { "fulfilled" } else { "cancelled" },
                    "reward" => reward_type.clone()
                );

                log_discord!(
                    "Rewards",
                    format!("🗒 Executed reward in {}", broadcaster_login),
                    0x1ed760,
                    "Reward" = reward_name,
                    "Type" = reward_type,
                    "Status" = format!("{:?}", status),
                    "Execution Time" = execution.as_secs_f64().to_string(),
                    "User" = executing_user_login,
                    "Input" = user_input
                );

                match update_reward_redemption(
                    &broadcaster_id,
                    &reward_id,
                    &redemption_id,
                    status,
                    &user.into(),
                )
                .await
                {
                    Ok(redemption) => log::info!(
                        "Final redemption: status={:?} {:?}",
                        redemption.status,
                        redemption
                    ),
                    Err(error) => log::warn!("Couldn't update reward redemption: {}", error),
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
                    Message::Notification(UserAuthorizationRevokeV1Payload {
                        user_name: Some(login),
                        ..
                    }) => login.into_string(),
                    Message::Notification(UserAuthorizationRevokeV1Payload { user_id, .. }) =>
                        user_id.into_string(),
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
