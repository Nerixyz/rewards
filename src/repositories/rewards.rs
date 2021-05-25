use crate::models::reward::{Reward, RewardData};
use crate::services::jwt::JwtClaims;
use crate::services::rewards::verify_reward;
use crate::services::sql::get_user_or_editor;
use crate::services::twitch::requests::{
    create_reward, delete_reward, get_reward_for_broadcaster_by_id, get_rewards_for_id,
    update_reward,
};
use actix_web::{delete, error, get, patch, put, web, Error, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use twitch_api2::helix::points::{CreateCustomRewardBody, CustomReward, UpdateCustomRewardBody};

#[derive(Deserialize)]
struct CreateRewardBody {
    pub twitch: CreateCustomRewardBody,
    pub data: RewardData,
}

#[derive(Deserialize)]
struct UpdateRewardBody {
    pub twitch: UpdateCustomRewardBody,
    pub data: RewardData,
}

#[derive(Serialize)]
struct CustomRewardResponse {
    twitch: CustomReward,
    data: RewardData,
}

#[put("/{broadcaster_id}")]
async fn create(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    body: web::Json<CreateRewardBody>,
    broadcaster_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let token = get_user_or_editor(&claims, &broadcaster_id, &pool)
        .await?
        .into();

    let body = body.into_inner();

    verify_reward(&body.data)
        .map_err(|e| error::ErrorBadRequest(format!("Your reward action is invalid: {}", e)))?;

    let reward = create_reward(&broadcaster_id, body.twitch, &token).await?;
    let db_reward = Reward::from_response(&reward, body.data);
    db_reward.create(&pool).await?;

    Ok(HttpResponse::Ok().json(CustomRewardResponse {
        twitch: reward,
        data: db_reward.data.0,
    }))
}

#[patch("/{broadcaster_id}/{reward_id}")]
async fn update(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    body: web::Json<UpdateRewardBody>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
    let (broadcaster_id, reward_id) = path.into_inner();
    let token = get_user_or_editor(&claims, &broadcaster_id, &pool)
        .await?
        .into();

    let body = body.into_inner();

    verify_reward(&body.data)
        .map_err(|e| error::ErrorBadRequest(format!("Your reward action is invalid: {}", e)))?;

    let reward = update_reward(&broadcaster_id, reward_id, body.twitch, &token).await?;
    let db_reward = Reward::from_response(&reward, body.data);
    db_reward.update(&pool).await?;

    Ok(HttpResponse::Ok().json(CustomRewardResponse {
        twitch: reward,
        data: db_reward.data.0,
    }))
}

#[delete("/{broadcaster_id}/{reward_id}")]
async fn delete(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
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
) -> Result<HttpResponse, Error> {
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
) -> Result<HttpResponse, Error> {
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
