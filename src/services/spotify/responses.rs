use itertools::Itertools;
use serde::Deserialize;

#[derive(Deserialize)]
#[non_exhaustive]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Deserialize)]
#[non_exhaustive]
pub struct RefreshTokenResponse {
    pub access_token: String,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct PlayerResponse {
    pub is_playing: bool,
    pub item: Option<PlayingItem>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum PlayingItem {
    Episode(EpisodeObject),
    Track(TrackObject),
}

impl std::fmt::Display for PlayingItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayingItem::Episode(ep) => {
                write!(f, "{} from {}", ep.name, ep.show.name)
            }
            PlayingItem::Track(track) => write!(f, "{}", track),
        }
    }
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct TrackObject {
    pub name: String,
    pub artists: Vec<ArtistObject>,
    pub uri: String,
    pub explicit: bool,
}

impl std::fmt::Display for TrackObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\"{}\" by {}",
            self.name,
            self.artists.iter().map(|i| &i.name).join(", ")
        )
    }
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct ArtistObject {
    pub name: String,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct EpisodeObject {
    pub name: String,
    pub show: SimplifiedShowObject,
}

#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct SimplifiedShowObject {
    pub name: String,
}

#[derive(Deserialize)]
#[non_exhaustive]
pub struct SearchResponse {
    pub tracks: Option<PagingObject<TrackObject>>,
}

#[derive(Deserialize)]
#[non_exhaustive]
pub struct PagingObject<T> {
    pub items: Vec<T>,
}
