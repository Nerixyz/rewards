pub mod requests;

pub async fn is_editor_in(name: &str) -> bool {
    requests::get_channels()
        .await
        .map(|channels| channels.iter().any(|channel| channel == name))
        .unwrap_or(false)
}
