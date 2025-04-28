use anyhow::Result as AnyhowResult;
use futures::TryStreamExt;
use models::reward::RewardToUpdate;
use sqlx::PgPool;
use twitch_api::{
    helix::{
        points::{
            CustomRewardRedemption, CustomRewardRedemptionStatus,
            GetCustomRewardRedemptionRequest, UpdateRedemptionStatusBody,
            UpdateRedemptionStatusRequest,
        },
        Response,
    },
    twitch_oauth2::UserToken,
    types::{IntoCow, RewardIdRef},
};

use crate::log_err;

use super::twitch::RHelixClient;

pub async fn clear_unfulfilled_redemptions(pool: &PgPool) -> AnyhowResult<()> {
    let mut stream = RewardToUpdate::get_all(pool);
    let client = RHelixClient::default();

    while let Some(reward_with_user) = stream.try_next().await? {
        let (reward_id, token) = reward_with_user.into();
        log_err!(
            clear_unfulfilled_redemptions_for_id(reward_id, &token, &client)
                .await,
            "Could not clear redemptions for id"
        );
    }

    Ok(())
}

async fn clear_unfulfilled_redemptions_for_id<'a>(
    reward_id: impl IntoCow<'a, RewardIdRef> + 'a,
    token: &'a UserToken,
    client: &RHelixClient<'_>,
) -> AnyhowResult<()> {
    let mut rewards: Response<
        GetCustomRewardRedemptionRequest,
        Vec<CustomRewardRedemption>,
    > = client
        .req_get(
            GetCustomRewardRedemptionRequest::broadcaster_id(
                token.user_id.as_str(),
            )
            .reward_id(reward_id)
            .status(CustomRewardRedemptionStatus::Unfulfilled),
            token,
        )
        .await?;

    loop {
        for redemption in &rewards.data {
            log::info!(
                "Clearing unfulfilled: broadcaster={}; reward_id={}; redemption_id={}",
                redemption.broadcaster_login,
                redemption.reward.id,
                redemption.id,
            );

            if let Err(e) = client
                .req_patch(
                    UpdateRedemptionStatusRequest::builder()
                        .broadcaster_id(redemption.broadcaster_id.clone())
                        .reward_id(redemption.reward.id.clone())
                        .id(redemption.id.clone())
                        .build(),
                    UpdateRedemptionStatusBody::builder()
                        .status(CustomRewardRedemptionStatus::Canceled)
                        .build(),
                    token,
                )
                .await
            {
                log::warn!("Could not update redemption: {}", e);
            }
        }

        if rewards.pagination.is_some() {
            if let Some(res) = rewards.get_next(client, token).await? {
                rewards = res;
                continue;
            }
        }
        break;
    }

    Ok(())
}
