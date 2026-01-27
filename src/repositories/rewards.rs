use crate::{
    log_discord,
    services::{
        jwt::JwtClaims,
        rewards::{
            save::save_reward,
            verify::{verify_live_delay, verify_reward},
        },
        sql::get_user_or_editor,
        twitch::requests::{
            create_reward, delete_reward, get_reward_for_broadcaster_by_id,
            get_rewards_for_id, update_reward,
        },
    },
    RedisPool,
};
use actix_web::{delete, get, patch, put, web, HttpResponse, Result};
use models::reward::{Reward, RewardData};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use twitch_api::{
    helix::points::{
        CreateCustomRewardBody, CustomReward, UpdateCustomRewardBody,
    },
    types::{RewardId, RewardIdRef},
};

#[derive(Deserialize, Debug)]
struct CreateRewardBody {
    pub twitch: CreateCustomRewardBody<'static>,
    pub data: RewardData,
    pub live_delay: Option<String>,
    pub auto_accept: bool,
}

#[derive(Deserialize, Debug)]
struct UpdateRewardBody {
    pub twitch: UpdateCustomRewardBody<'static>,
    pub data: RewardData,
    pub live_delay: Option<String>,
    pub auto_accept: bool,
}

#[derive(Serialize)]
struct CustomRewardResponse {
    twitch: CustomReward,
    data: RewardData,
    live_delay: Option<String>,
    auto_accept: bool,
}

#[put("/{broadcaster_id}")]
async fn create(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
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

    verify_live_delay(&body.live_delay).map_err(|e| {
        errors::ErrorBadRequest(format!("Your live delay is invalid: {}", e))
    })?;
    verify_reward(&body.data, &broadcaster_id, &pool, &token)
        .await
        .map_err(|e| {
            errors::ErrorBadRequest(format!(
                "Your reward action is invalid: {}",
                e
            ))
        })?;

    let reward = create_reward(&broadcaster_id, body.twitch, &token).await?;

    let db_reward = Reward::from_response(
        &reward,
        body.data.clone(),
        body.live_delay,
        body.auto_accept,
    );
    db_reward.create(&pool).await?;

    if let Err(e) = save_reward(
        &body.data,
        reward.id.as_ref(),
        &broadcaster_id,
        &pool,
        &redis_pool,
    )
    .await
    {
        log::warn!("Could not save reward: {}", e);

        let (internal, twitch) = futures::future::join(
            Reward::delete(reward.id.as_ref(), &pool),
            delete_reward(
                broadcaster_id.as_str(),
                <RewardId as AsRef<RewardIdRef>>::as_ref(&reward.id),
                &token,
            ),
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

    log_discord!(
        "Rewards",
        "🎉 Created reward",
        0x9355fb,
        "User" = reward.broadcaster_login.clone().take(),
        "Title" = reward.title.clone(),
        "Type" = body.data.to_string(),
        "Id" = reward.id.clone().take()
    );

    Ok(HttpResponse::Ok().json(CustomRewardResponse {
        twitch: reward,
        data: db_reward.data.0,
        live_delay: db_reward.live_delay,
        auto_accept: db_reward.auto_accept,
    }))
}

#[patch("/{broadcaster_id}/{reward_id}")]
async fn update(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
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

    verify_live_delay(&body.live_delay).map_err(|e| {
        errors::ErrorBadRequest(format!("Your live delay is invalid: {}", e))
    })?;
    verify_reward(&body.data, &broadcaster_id, &pool, &token)
        .await
        .map_err(|e| {
            errors::ErrorBadRequest(format!(
                "Your reward action is invalid: {}",
                e
            ))
        })?;

    // check this before it's actually saved
    if let Err(e) =
        save_reward(&body.data, &reward_id, &broadcaster_id, &pool, &redis_pool)
            .await
    {
        log::warn!("Could not save reward: {}", e);

        return Err(errors::ErrorBadRequest(format!(
            "Your reward could not be saved: {}",
            e
        )));
    }

    let reward =
        update_reward(broadcaster_id, reward_id, body.twitch, &token).await?;
    let data_type = body.data.to_string();
    let db_reward = Reward::from_response(
        &reward,
        body.data,
        body.live_delay,
        body.auto_accept,
    );
    db_reward.update(&pool).await?;

    log_discord!(
        "Rewards",
        "🔁 Updated reward",
        0xf99500,
        "User" = reward.broadcaster_login.clone().take(),
        "Title" = reward.title.clone(),
        "Type" = data_type,
        "Id" = reward.id.clone().take()
    );

    Ok(HttpResponse::Ok().json(CustomRewardResponse {
        twitch: reward,
        data: db_reward.data.0,
        live_delay: db_reward.live_delay,
        auto_accept: db_reward.auto_accept,
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

    log_discord!(
        "Rewards",
        "🗑 Deleted reward",
        0xff0a12,
        "User" = token.login.take(),
        "Id" = reward_id
    );

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
        get_rewards_for_id(broadcaster_id.as_str(), &token),
        Reward::get_all_for_user(broadcaster_id.as_str(), &pool),
    )
    .await;

    Ok(HttpResponse::Ok().json(GetRewardsResponse {
        data: saved_rewards?,
        twitch: rewards?,
    }))
}

#[derive(Serialize)]
struct ListSwapEmotesResponse {
    twitch: CustomReward,
    data: Reward,
    emotes: Vec<models::swap_emote::SwapEmote>,
}

#[get("/{broadcaster_id}/{reward_id}/swap-emotes")]
async fn list_swap_emotes(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse> {
    let (broadcaster_id, reward_id) = path.into_inner();
    let token = get_user_or_editor(&claims, &broadcaster_id, &pool)
        .await?
        .into();

    let (reward, saved_reward, emotes) = futures::future::join3(
        get_reward_for_broadcaster_by_id(
            &broadcaster_id,
            &[reward_id.as_str().into()],
            &token,
        ),
        Reward::get_by_id(&reward_id, &pool),
        models::swap_emote::SwapEmote::all_for_reward(
            &broadcaster_id,
            &reward_id,
            &pool,
        ),
    )
    .await;

    Ok(HttpResponse::Ok().json(ListSwapEmotesResponse {
        data: saved_reward?,
        twitch: reward?,
        emotes: emotes?,
    }))
}

#[derive(Serialize)]
struct GetSwapEmoteUsage {
    usage: i64,
}

#[get("/{broadcaster_id}/{reward_id}/swap-emotes/usage")]
async fn get_swap_emotes_usage(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse> {
    let (broadcaster_id, reward_id) = path.into_inner();
    get_user_or_editor(&claims, &broadcaster_id, &pool).await?;

    let count = models::swap_emote::SwapEmote::emote_count(
        &broadcaster_id,
        &reward_id,
        &pool,
    )
    .await?;

    Ok(HttpResponse::Ok().json(GetSwapEmoteUsage { usage: count }))
}

#[delete("/{broadcaster_id}/{reward_id}/swap-emotes/{id}")]
async fn untrack_swap_emote(
    claims: JwtClaims,
    pool: web::Data<PgPool>,
    path: web::Path<(String, String, i64)>,
) -> Result<HttpResponse> {
    let (broadcaster_id, reward_id, id) = path.into_inner();
    get_user_or_editor(&claims, &broadcaster_id, &pool).await?;

    models::swap_emote::SwapEmote::remove_on_reward(
        id,
        &broadcaster_id,
        &reward_id,
        &pool,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

pub fn init_rewards_routes(config: &mut web::ServiceConfig) {
    config
        .service(create)
        .service(update)
        .service(delete)
        .service(list_for_user)
        .service(list_swap_emotes)
        .service(get_swap_emotes_usage)
        .service(untrack_swap_emote);
}
