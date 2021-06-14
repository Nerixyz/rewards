use crate::actors::irc_actor::IrcActor;
use crate::actors::messages::irc_messages::TimedModeMessage;
use crate::log_err;
use crate::models::timed_mode::TimedMode;
use actix::Addr;
use anyhow::Result as AnyResult;
use chrono::Utc;
use sqlx::PgPool;

pub async fn resolve_timed_modes(irc: Addr<IrcActor>, pool: &PgPool) -> AnyResult<()> {
    let all = TimedMode::get_all(pool).await?;
    if !all.is_empty() {
        log::info!("Resolving {} timed modes", all.len());
    }
    for mode in all {
        // make sure to delete the emote at the start - the irc functions create an entry
        TimedMode::delete_mode(mode.id, pool).await?;

        let duration = (mode.end_ts - Utc::now())
            .to_std()
            .unwrap_or_else(|_| std::time::Duration::from_secs(0));
        log_err!(
            irc.send(TimedModeMessage {
                mode: mode.mode,
                broadcaster: match mode.user_name {
                    Some(name) => name,
                    // don't return an error
                    None => continue,
                },
                broadcaster_id: mode.user_id,
                duration: duration.as_secs()
            })
            .await,
            "Could not send IRC message"
        );
    }

    Ok(())
}