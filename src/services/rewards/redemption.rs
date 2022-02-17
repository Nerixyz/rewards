use super::Redemption;
use crate::{
    actors::{irc::WhisperMessage, rewards::ExecuteRewardMessage},
    log_discord,
    services::twitch::eventsub::update_reward_redemption,
    IrcActor, PgPool, RewardsActor, User,
};
use actix::{Addr, MailboxError};
use models::reward::Reward;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use thiserror::Error;
use twitch_api2::{
    eventsub::{channel::ChannelPointsCustomRewardRedemptionAddV1, EventSubscriptionInformation},
    helix::points::CustomRewardRedemptionStatus,
};

#[derive(Error, Debug)]
pub enum ReceiveRedemptionError {
    #[error("This reward isn't handled by RewardMore")]
    NoReward,
}

pub struct ReceiveRedemptionCtx {
    pub pool: Arc<PgPool>,
    pub executor: Arc<Addr<RewardsActor>>,
    pub irc: Arc<Addr<IrcActor>>,
    pub notification: Redemption,
    pub subscription: EventSubscriptionInformation<ChannelPointsCustomRewardRedemptionAddV1>,
    pub user: User,
}

struct RedemptionCtx {
    broadcaster_login: String,
    executing_user_login: String,
    user_input: String,

    reward_name: String,
    reward_type: String,
}

struct RedemptionUpdateHandle {
    broadcaster_id: String,
    reward_id: String,
    redemption_id: String,
}

impl From<(&Redemption, &Reward)> for RedemptionCtx {
    fn from((notification, reward): (&Redemption, &Reward)) -> Self {
        Self {
            executing_user_login: notification.user_name.clone().into_string(),
            broadcaster_login: notification.broadcaster_user_login.clone().into_string(),
            reward_name: notification.reward.title.clone(),
            reward_type: reward.data.0.to_string(),
            user_input: notification.user_input.clone(),
        }
    }
}

impl From<&Redemption> for RedemptionUpdateHandle {
    fn from(notification: &Redemption) -> Self {
        Self {
            broadcaster_id: notification.broadcaster_user_id.clone().into_string(),
            reward_id: notification.reward.id.clone().into_string(),
            redemption_id: notification.id.clone().into_string(),
        }
    }
}

impl RedemptionCtx {
    async fn handle_execution_error(
        &self,
        error: Result<anyhow::Result<()>, MailboxError>,
        irc: &Arc<Addr<IrcActor>>,
    ) {
        let (debug, display) = match error {
            Err(e) => (format!("{:?}", e), e.to_string()),
            Ok(Err(e)) => (format!("{:?}", e), e.to_string()),
            Ok(Ok(_)) => unreachable!(),
        };

        log::warn!("Could not execute reward: {:?}", debug);

        log_discord!(
            "Rewards",
            format!("⚠ Failed to execute reward in {}", self.broadcaster_login),
            0xfab43e,
            "Reward" = self.reward_name.clone(),
            "Type" = self.reward_type.clone(),
            "Error" = display
        );

        match irc.send(WhisperMessage(self.executing_user_login.clone(), "[Refund] ⚠ I could not execute the reward. Make sure you provided the correct input!".to_string())).await {
            Err(e) => log::warn!("MailboxError on sending chat: {}", e),
            Ok(Err(e)) => log::warn!("Error sending chat: {}", e),
            _ => ()
        }
    }

    async fn log_redemption(
        self,
        status: CustomRewardRedemptionStatus,
        redemption_received: Instant,
    ) {
        metrics::increment_counter!("rewards_redemptions",
            "status" => if status == CustomRewardRedemptionStatus::Fulfilled { "fulfilled" } else { "cancelled" },
            "reward" => self.reward_type.clone()
        );
        let execution = Instant::now()
            .checked_duration_since(redemption_received)
            .unwrap_or_else(|| Duration::from_secs(0));
        metrics::histogram!("rewards_redemption_execution_duration",
            execution.as_secs_f64(),
            "status" => if status == CustomRewardRedemptionStatus::Fulfilled { "fulfilled" } else { "cancelled" },
            "reward" => self.reward_type.clone()
        );

        log_discord!(
            "Rewards",
            format!("🗒 Executed reward in {}", self.broadcaster_login),
            0x1ed760,
            "Reward" = self.reward_name,
            "Type" = self.reward_type,
            "Status" = format!("{:?}", status),
            "Execution Time" = execution.as_secs_f64().to_string(),
            "User" = self.executing_user_login,
            "Input" = if self.user_input.is_empty() {
                "<no input>".to_string()
            } else {
                self.user_input
            }
        );
    }
}

impl RedemptionUpdateHandle {
    async fn update(self, user: User, status: CustomRewardRedemptionStatus) {
        match update_reward_redemption(
            &self.broadcaster_id,
            &self.reward_id,
            &self.redemption_id,
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
    }
}

pub async fn receive(ctx: ReceiveRedemptionCtx) -> Result<(), ReceiveRedemptionError> {
    let ReceiveRedemptionCtx {
        pool,
        executor,
        irc,
        notification,
        user,
        subscription,
    } = ctx;
    let redemption_received = Instant::now();

    let reward = Reward::get_by_id(notification.reward.id.as_ref(), &pool)
        .await
        .map_err(|_| ReceiveRedemptionError::NoReward)?;

    let ctx = RedemptionCtx::from((&notification, &reward));
    let update_handle = RedemptionUpdateHandle::from(&notification);

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
            ctx.handle_execution_error(e, &irc).await;
            CustomRewardRedemptionStatus::Canceled
        }
    };
    // here, the redemption is finally updated, so we'll log this
    ctx.log_redemption(status.clone(), redemption_received)
        .await;
    update_handle.update(user, status).await;

    Ok(())
}
