use std::collections::HashSet;

use futures_util::TryFutureExt;
use sqlx::PgPool;
use twitch_api::{helix::points::CustomReward, types::RewardIdRef};

use crate::{services::twitch, AnyError};
use anyhow::Result as AnyResult;

async fn get_twitch_rewards(
    user_id: &str,
    pool: &PgPool,
) -> AnyResult<Vec<CustomReward>> {
    let user = models::user::User::get_by_id(user_id, pool).await?;
    let rewards =
        twitch::requests::get_rewards_for_id(user_id, &user.into()).await?;

    Ok(rewards)
}

pub async fn sync_rewards(user_id: &str, pool: &PgPool) -> AnyResult<usize> {
    let (rewards, saved_rewards) = futures::future::try_join(
        get_twitch_rewards(user_id, pool),
        models::reward::Reward::get_all_for_user(user_id, pool)
            .map_err(AnyError::from),
    )
    .await?;

    let ids: HashSet<&RewardIdRef> =
        rewards.iter().map(|x| x.id.as_ref()).collect();

    let mut removed = 0;
    for reward in saved_rewards
        .iter()
        .filter(|x| !ids.contains(RewardIdRef::from_str(&x.id)))
    {
        models::reward::Reward::delete(&reward.id, pool).await?;
        removed += 1;
    }

    Ok(removed)
}
