use anyhow::{Error as AnyError, Result as AnyResult};

use config::CONFIG;

pub mod requests;

pub async fn verify_user(
    broadcaster_id: &str
) -> AnyResult<()> {
    let editors = requests::get_user(broadcaster_id).await?.user.editors;

    if editors.iter().any(|e| e.id == CONFIG.emotes.seven_tv.user_id) {
        Ok(())
    } else {
        Err(AnyError::msg("RewardMore isn't an editor for the user"))
    }
}
