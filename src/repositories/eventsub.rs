use crate::actors::irc_actor::IrcActor;
use crate::actors::messages::irc_messages::WhisperMessage;
use crate::actors::timeout_actor::TimeoutActor;
use crate::models::reward::Reward;
use crate::models::user::User;
use crate::services::rewards::execute::execute_reward;
use crate::services::twitch::eventsub::update_reward_redemption;
use actix::Addr;
use actix_web::{
    post,
    web::{self, ServiceConfig},
    HttpResponse, Result,
};
use sqlx::PgPool;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use twitch_api2::eventsub;
use twitch_api2::eventsub::Payload;
use twitch_api2::helix::points::CustomRewardRedemptionStatus;
use twitch_api2::twitch_oauth2::AppAccessToken;

#[post("/reward")]
async fn reward_redemption(
    pool: web::Data<PgPool>,
    irc: web::Data<Addr<IrcActor>>,
    payload: web::Json<eventsub::Payload>,
    app_token: web::Data<RwLock<AppAccessToken>>,
    timeout_actor: web::Data<Addr<TimeoutActor>>,
) -> Result<HttpResponse> {
    match payload.into_inner() {
        Payload::VerificationRequest(rq) => {
            log::info!("verification: {:?}", rq);
            Ok(HttpResponse::Ok().body(rq.challenge))
        }
        Payload::ChannelPointsCustomRewardRedemptionAddV1(redemption) => {
            // main path
            let user = User::get_by_id(&redemption.event.broadcaster_user_id, &pool).await?;

            log::info!("redemption: {:?}", redemption);

            let pool = pool.into_inner();
            let irc = irc.into_inner();
            let redemption_received = Instant::now();
            actix_web::rt::spawn(async move {
                let reward = Reward::get_by_id(&redemption.event.reward.id, &pool).await;

                let broadcaster_id = redemption.event.broadcaster_user_id.clone();
                let reward_id = redemption.event.reward.id.clone();
                let redemption_id = redemption.event.id.clone();

                let executing_user_login = redemption.event.user_name.clone();

                if let (Ok(reward), Ok(user_token)) =
                    (reward, User::get_by_id(&broadcaster_id, &pool).await)
                {
                    let reward_type = reward.data.0.to_string();
                    let status = match execute_reward(
                        redemption,
                        reward,
                        user,
                        &*pool,
                        irc.clone(),
                        timeout_actor.into_inner(),
                        app_token.into_inner(),
                    )
                    .await
                    {
                        Ok(_) => CustomRewardRedemptionStatus::Fulfilled,
                        Err(e) => {
                            log::warn!("Could not execute reward: {:?}", e);

                            match irc.send(WhisperMessage(executing_user_login, "[Refund] ⚠ I could not execute the reward. Make sure you provided the correct input!".to_string())).await {
                                Err(e) => log::warn!("MailboxError on sending chat: {}", e),
                                Ok(Err(e)) => log::warn!("Error sending chat: {}", e),
                                _ => ()
                            }

                            CustomRewardRedemptionStatus::Canceled
                        }
                    };
                    // here, the redemption is finally updated, so we'll log this
                    metrics::increment_counter!("rewards_redemptions", "status" => if status == CustomRewardRedemptionStatus::Fulfilled { "fulfilled" } else { "cancelled" }, "reward" => reward_type.clone());
                    let execution = Instant::now()
                        .checked_duration_since(redemption_received)
                        .unwrap_or_else(|| Duration::from_secs(0));
                    metrics::histogram!("rewards_redemption_execution_duration",
                        execution.as_secs_f64(),
                        "status" => if status == CustomRewardRedemptionStatus::Fulfilled { "fulfilled" } else { "cancelled" },
                        "reward" => reward_type.clone()
                    );

                    match update_reward_redemption(
                        &broadcaster_id,
                        &reward_id,
                        &redemption_id,
                        status,
                        &user_token.into(),
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
                } else {
                    log::warn!(
                        "failed to get user or reward: userId: {}, rewardID: {}",
                        redemption.event.broadcaster_user_id,
                        redemption.event.reward.id
                    );
                }
            });

            Ok(HttpResponse::Ok().finish())
        }
        Payload::UserAuthorizationRevokeV1(re) => {
            log::warn!("auth revoke: {:?}", re);
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
