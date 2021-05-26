use crate::actors::irc_actor::IrcActor;
use crate::models::reward::Reward;
use crate::models::user::User;
use crate::services::rewards::execute_reward;
use crate::services::twitch::eventsub::update_reward_redemption;
use actix::Addr;
use actix_web::{
    post,
    web::{self, ServiceConfig},
    Error, HttpResponse,
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
) -> Result<HttpResponse, Error> {
    match payload.into_inner() {
        Payload::VerificationRequest(rq) => Ok(HttpResponse::Ok().body(rq.challenge)),
        Payload::ChannelPointsCustomRewardRedemptionAddV1(redemption) => {
            // main path
            let user = User::get_by_id(&redemption.event.broadcaster_user_id, &pool).await?;

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
                            println!("Could not execute reward: {:?}", e);
                            CustomRewardRedemptionStatus::Canceled
                        }
                    };
                    if let Err(e) = update_reward_redemption(
                        &broadcaster_id,
                        &reward_id,
                        &redemption_id,
                        status,
                        &user_token.into(),
                    )
                    .await
                    {
                        println!("No shot, couldn't update reward redemption: {}", e);
                        // TODO: logging, better error handling
                    }
                }
                // TODO: logging
            });

            Ok(HttpResponse::NoContent().finish())
        }
        Payload::UserAuthorizationRevokeV1(_) => {
            // TODO
            Ok(HttpResponse::NoContent().finish())
        }
        _ => Ok(HttpResponse::NotFound().body("I can't handle that!")),
    }
}

pub fn init_eventsub_routes(config: &mut ServiceConfig) {
    config.service(reward_redemption);
}
