use actix_web::{error, http::StatusCode};
use derive_more::Display;
use twitch_api2::helix::{
    ClientRequestError, CreateRequestError, HelixRequestDeleteError, HelixRequestGetError,
    HelixRequestPatchError, HelixRequestPostError, HelixRequestPutError,
};

#[derive(Display, Debug)]
pub enum TwitchApiError {
    #[display(fmt = "Reqwest Error")]
    ReqwestError,
    #[display(fmt = "Http crate error")]
    HttpCrateError,
    #[display(fmt = "No Page")]
    NoPage,
    #[display(fmt = "Invalid Uri")]
    InvalidUri,
    #[display(fmt = "Invalid Utf8 received")]
    Utf8,
    #[display(fmt = "Serde error")]
    Serde,
    #[display(fmt = "TwitchError: {} - {}", _0, _1)]
    Response(StatusCode, String),
    #[display(fmt = "Some custom error")]
    Custom,
    #[display(fmt = "{}", _0)]
    Other(String),
}

impl std::error::Error for TwitchApiError {}
impl error::ResponseError for TwitchApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Response(status, _) => *status,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<ClientRequestError<reqwest::Error>> for TwitchApiError {
    fn from(e: ClientRequestError<reqwest::Error>) -> Self {
        // Although `ClientRequestError` is Display and Debug, it's too much to expose, so here we're converting...
        // Still. It should be improved.
        match e {
            ClientRequestError::RequestError(_) => Self::ReqwestError,
            ClientRequestError::NoPage => Self::NoPage,
            ClientRequestError::CreateRequestError(e) => match e {
                CreateRequestError::HttpError(_) => Self::HttpCrateError,
                CreateRequestError::SerializeError(_) => Self::Serde,
                CreateRequestError::InvalidUri(_) => Self::InvalidUri,
                CreateRequestError::Custom(_) => Self::Custom,
            },
            ClientRequestError::HelixRequestGetError(e) => match e {
                HelixRequestGetError::Error {
                    status, message, ..
                } => Self::Response(status, message),
                HelixRequestGetError::Utf8Error(_, _, _) => Self::Utf8,
                HelixRequestGetError::DeserializeError(_, _, _, _) => Self::Serde,
                HelixRequestGetError::InvalidResponse { status, reason, .. } => {
                    Self::Response(status, reason.to_string())
                }
                HelixRequestGetError::InvalidUri(_) => Self::InvalidUri,
            },
            ClientRequestError::HelixRequestPutError(e) => match e {
                HelixRequestPutError::Error {
                    status, message, ..
                } => Self::Response(status, message),
                HelixRequestPutError::Utf8Error(_, _, _) => Self::Utf8,
                HelixRequestPutError::DeserializeError(_, _, _, _) => Self::Serde,
                HelixRequestPutError::InvalidResponse { status, reason, .. } => {
                    Self::Response(status, reason.to_string())
                }
            },
            ClientRequestError::HelixRequestPostError(e) => match e {
                HelixRequestPostError::Error {
                    status, message, ..
                } => Self::Response(status, message),
                HelixRequestPostError::Utf8Error(_, _, _) => Self::Utf8,
                HelixRequestPostError::DeserializeError(_, _, _, _) => Self::Serde,
                HelixRequestPostError::InvalidResponse { status, reason, .. } => {
                    Self::Response(status, reason.to_string())
                }
            },
            ClientRequestError::HelixRequestPatchError(e) => match e {
                HelixRequestPatchError::Error {
                    status, message, ..
                } => Self::Response(status, message),
                HelixRequestPatchError::Utf8Error(_, _, _) => Self::Utf8,
                HelixRequestPatchError::DeserializeError(_, _, _, _) => Self::Serde,
                HelixRequestPatchError::InvalidResponse { status, reason, .. } => {
                    Self::Response(status, reason.to_string())
                }
            },
            ClientRequestError::HelixRequestDeleteError(e) => match e {
                HelixRequestDeleteError::Error {
                    status, message, ..
                } => Self::Response(status, message),
                HelixRequestDeleteError::Utf8Error(_, _, _) => Self::Utf8,
                HelixRequestDeleteError::InvalidResponse { status, reason, .. } => {
                    Self::Response(status, reason.to_string())
                }
            },
            ClientRequestError::Custom(_) => Self::Custom,
        }
    }
}
