use crate::actors::irc_actor::IrcActor;
use crate::models::reward::Reward;
use crate::models::user::User;
use crate::services::rewards::execute_reward;
use crate::services::twitch::eventsub::update_reward_redemption;
use actix::Addr;
use actix_web::{
    post,
    web::{self, ServiceConfig},
    Result, HttpResponse,
};
use sqlx::PgPool;
use twitch_api2::eventsub;
use twitch_api2::eventsub::Payload;
use twitch_api2::helix::points::CustomRewardRedemptionStatus;

#[post("/reward")]
async fn reward_redemption(
    pool: web::Data<PgPool>,
    irc: web::Data<Addr<IrcActor>>,
    payload: web::Json<eventsub::Payload>,
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
            actix_web::rt::spawn(async move {
                let reward = Reward::get_by_id(&redemption.event.reward.id, &pool).await;

                let broadcaster_id = redemption.event.broadcaster_user_id.clone();
                let reward_id = redemption.event.reward.id.clone();
                let redemption_id = redemption.event.id.clone();

                if let (Ok(reward), Ok(user_token)) =
                    (reward, User::get_by_id(&broadcaster_id, &pool).await)
                {
                    let status = match execute_reward(redemption, reward, user, &*pool, irc).await {
                        Ok(_) => CustomRewardRedemptionStatus::Fulfilled,
                        Err(e) => {
                            log::warn!("Could not execute reward: {:?}", e);
                            CustomRewardRedemptionStatus::Canceled
                        }
                    };
                    match update_reward_redemption(
                        &broadcaster_id,
                        &reward_id,
                        &redemption_id,
                        status,
                        &user_token.into(),
                    )
                    .await
                    {
                        Ok(redemption) => log::info!("Final redemption: {:?}", redemption),
                        Err(error) => log::warn!("Couldn't update reward redemption: {}", error)
                    }
                } else {
                    log::warn!("failed to get user or reward: userId: {}, rewardID: {}", redemption.event.broadcaster_user_id, redemption.event.reward.id);
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
        },
    }
}

pub fn init_eventsub_routes(config: &mut ServiceConfig) {
    config.service(reward_redemption);
}
