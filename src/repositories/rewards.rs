use crate::models::reward::{Reward, RewardData};
use crate::services::errors;
use crate::services::jwt::JwtClaims;
use crate::services::rewards::save::save_reward;
use crate::services::rewards::verify::{verify_live_delay, verify_reward};
use crate::services::sql::get_user_or_editor;
use crate::services::twitch::requests::{
    create_reward, delete_reward, get_reward_for_broadcaster_by_id, get_rewards_for_id,
    update_reward,
};
use actix_web::{delete, get, patch, put, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use twitch_api2::helix::points::{CreateCustomRewardBody, CustomReward, UpdateCustomRewardBody};

#[derive(Deserialize, Debug)]
struct CreateRewardBody {
    pub twitch: CreateCustomRewardBody,
    pub data: RewardData,
    pub live_delay: Option<String>,
}

#[derive(Deserialize, Debug)]
struct UpdateRewardBody {
    pub twitch: UpdateCustomRewardBody,
    pub data: RewardData,
    pub live_delay: Option<String>,
}

#[derive(Serialize)]
struct CustomRewardResponse {
    twitch: CustomReward,
    data: RewardData,
    live_delay: Option<String>,
}

#[put("/{broadcaster_id}")]
async fn create(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    body: web::Json<CreateRewardBody>,
    broadcaster_id: web::Path<String>,
) -> Result<HttpResponse> {
    let token = get_user_or_editor(&claims, &broadcaster_id, &pool)
        .await?
        .into();

    let body = body.into_inner();

    log::info!(
        "Create reward: broadcaster_id={}; data={:?}",
        broadcaster_id,
        body
    );

    verify_live_delay(&body.live_delay)
        .map_err(|e| errors::ErrorBadRequest(format!("Your live delay is invalid: {}", e)))?;
    verify_reward(&body.data, &broadcaster_id, &pool, &token)
        .await
        .map_err(|e| errors::ErrorBadRequest(format!("Your reward action is invalid: {}", e)))?;

    let reward = create_reward(&broadcaster_id, body.twitch, &token).await?;

    let db_reward = Reward::from_response(&reward, body.data.clone(), body.live_delay);
    db_reward.create(&pool).await?;

    if let Err(e) = save_reward(&body.data, reward.id.as_ref(), &broadcaster_id, &pool).await {
        log::warn!("Could not save reward: {}", e);

        let (internal, twitch) = futures::future::join(
            Reward::delete(reward.id.as_ref(), &pool),
            delete_reward(&broadcaster_id, reward.id.clone(), &token),
        )
        .await;
        if let Err(e) = internal {
            log::warn!("Could not delete invalid reward internally: {}", e);
        }
        if let Err(e) = twitch {
            log::warn!("Could not delete invalid reward: {}", e);
        }

        return Err(errors::ErrorBadRequest(format!(
            "Your reward could not be saved: {}",
            e
        )));
    }

    Ok(HttpResponse::Ok().json(CustomRewardResponse {
        twitch: reward,
        data: db_reward.data.0,
        live_delay: db_reward.live_delay,
    }))
}

#[patch("/{broadcaster_id}/{reward_id}")]
async fn update(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    body: web::Json<UpdateRewardBody>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse> {
    let (broadcaster_id, reward_id) = path.into_inner();
    let token = get_user_or_editor(&claims, &broadcaster_id, &pool)
        .await?
        .into();

    let body = body.into_inner();

    log::info!(
        "Update reward: broadcaster_id={}; reward_id={}; data={:?}",
        broadcaster_id,
        reward_id,
        body
    );

    verify_live_delay(&body.live_delay)
        .map_err(|e| errors::ErrorBadRequest(format!("Your live delay is invalid: {}", e)))?;
    verify_reward(&body.data, &broadcaster_id, &pool, &token)
        .await
        .map_err(|e| errors::ErrorBadRequest(format!("Your reward action is invalid: {}", e)))?;

    // check this before it's actually saved
    if let Err(e) = save_reward(&body.data, &reward_id, &broadcaster_id, &pool).await {
        log::warn!("Could not save reward: {}", e);

        return Err(errors::ErrorBadRequest(format!(
            "Your reward could not be saved: {}",
            e
        )));
    }

    let reward = update_reward(&broadcaster_id, reward_id, body.twitch, &token).await?;
    let db_reward = Reward::from_response(&reward, body.data, body.live_delay);
    db_reward.update(&pool).await?;

    Ok(HttpResponse::Ok().json(CustomRewardResponse {
        twitch: reward,
        data: db_reward.data.0,
        live_delay: db_reward.live_delay,
    }))
}

#[delete("/{broadcaster_id}/{reward_id}")]
async fn delete(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse> {
    let (broadcaster_id, reward_id) = path.into_inner();
    let token = get_user_or_editor(&claims, &broadcaster_id, &pool)
        .await?
        .into();

    delete_reward(&broadcaster_id, reward_id.clone(), &token).await?;
    // this has to be done afterwards as only then the reward is removed
    Reward::delete(&reward_id, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Serialize)]
struct GetRewardsResponse {
    twitch: Vec<CustomReward>,
    data: Vec<Reward>,
}

#[get("/{broadcaster_id}")]
async fn list_for_user(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    broadcaster_id: web::Path<String>,
) -> Result<HttpResponse> {
    let token = get_user_or_editor(&claims, &broadcaster_id, &pool)
        .await?
        .into();

    let (rewards, saved_rewards) = futures::future::join(
        get_rewards_for_id(&broadcaster_id, &token),
        Reward::get_all_for_user(&broadcaster_id, &pool),
    )
    .await;

    Ok(HttpResponse::Ok().json(GetRewardsResponse {
        data: saved_rewards?,
        twitch: rewards?,
    }))
}

#[derive(Serialize)]
struct GetRewardResponse {
    twitch: CustomReward,
    data: Reward,
}

#[get("/{broadcaster_id}/{reward_id}")]
async fn get_reward(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse> {
    let (broadcaster_id, reward_id) = path.into_inner();
    let token = get_user_or_editor(&claims, &broadcaster_id, &pool)
        .await?
        .into();

    let (reward, saved_reward) = futures::future::join(
        get_reward_for_broadcaster_by_id(&broadcaster_id, reward_id.clone(), &token),
        Reward::get_by_id(&reward_id, &pool),
    )
    .await;

    Ok(HttpResponse::Ok().json(GetRewardResponse {
        data: saved_reward?,
        twitch: reward?,
    }))
}

pub fn init_rewards_routes(config: &mut web::ServiceConfig) {
    config
        .service(create)
        .service(update)
        .service(delete)
        .service(list_for_user);
}
