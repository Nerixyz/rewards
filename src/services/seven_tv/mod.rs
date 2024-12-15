use anyhow::{Error as AnyError, Result as AnyResult};

use config::CONFIG;
use log::warn;
use requests::SevenEditorState;

pub mod requests;

pub async fn verify_user(broadcaster_id: &str) -> AnyResult<()> {
    let editors = requests::get_user(broadcaster_id).await?.user.editors;

    if editors
        .iter()
        .any(|e| e.id == CONFIG.emotes.seven_tv.user_id)
    {
        Ok(())
    } else {
        Err(AnyError::msg("RewardMore isn't an editor for the user - if you just registered, please wait a minute"))
    }
}

pub async fn approve_all_pending_editors() -> AnyResult<()> {
    let all_relations = requests::get_editor_relations().await?;

    for relation in all_relations
        .iter()
        .filter(|r| r.state == SevenEditorState::Pending)
    {
        if let Err(e) =
            requests::approve_editor(&relation.user_id, &relation.editor_id)
                .await
        {
            warn!("Failed to approve 7TV user {} - {e}", relation.user_id);
        }
    }
    Ok(())
}
