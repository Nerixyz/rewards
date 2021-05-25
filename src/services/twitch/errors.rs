use actix_web::{error, http::StatusCode, Error};
use derive_more::Error;
use serde::Serialize;
use std::fmt::{Display, Formatter};
use twitch_api2::helix::{
    ClientRequestError, CreateRequestError, HelixRequestDeleteError, HelixRequestGetError,
    HelixRequestPatchError, HelixRequestPostError, HelixRequestPutError,
};

#[derive(Serialize, Error, Debug)]
struct ErrorResponse {
    status: Option<u16>,
    message: String,
}

impl ErrorResponse {
    pub fn new(message: String) -> Self {
        Self {
            message,
            status: None,
        }
    }

    pub fn with_status(message: String, status: StatusCode) -> Self {
        Self {
            message,
            status: Some(status.as_u16()),
        }
    }
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            serde_json::to_string(&self)
                .unwrap_or("".to_string())
                .as_ref(),
        )
    }
}

impl error::ResponseError for ErrorResponse {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub fn to_response_error(e: ClientRequestError<reqwest::Error>) -> Error {
    // TODO: logging?
    // most of these errors are rare or even unreachable but this provides an error in any case
    match e {
        ClientRequestError::RequestError(e) => match e {
            reqwest::Error { .. } => ErrorResponse::new("ReqwestError".to_string()),
        },
        ClientRequestError::NoPage => ErrorResponse::new("NoPage".to_string()),
        ClientRequestError::CreateRequestError(e) => match e {
            CreateRequestError::HttpError(e) => ErrorResponse::new(format!("{}", e)),
            CreateRequestError::SerializeError(_) => {
                ErrorResponse::new("SerializeError".to_string())
            }
            CreateRequestError::InvalidUri(_) => ErrorResponse::new("InvlaidUri".to_string()),
            CreateRequestError::Custom(e) => ErrorResponse::new(e.to_string()),
        },
        ClientRequestError::HelixRequestGetError(e) => match e {
            HelixRequestGetError::Error {
                status, message, ..
            } => ErrorResponse::with_status(message, status),
            HelixRequestGetError::Utf8Error(_, _, _) => {
                ErrorResponse::new("UTF8-Error".to_string())
            }
            HelixRequestGetError::DeserializeError(_, _, _, _) => {
                ErrorResponse::new("DeserializeError".to_string())
            }
            HelixRequestGetError::InvalidResponse { status, reason, .. } => {
                ErrorResponse::with_status(reason.to_string(), status)
            }
            HelixRequestGetError::InvalidUri(_) => {
                ErrorResponse::new("DeserializeError".to_string())
            }
        },
        ClientRequestError::HelixRequestPutError(e) => match e {
            HelixRequestPutError::Error {
                status, message, ..
            } => ErrorResponse::with_status(message, status),
            HelixRequestPutError::Utf8Error(_, _, _) => {
                ErrorResponse::new("UTF8-Error".to_string())
            }
            HelixRequestPutError::DeserializeError(_, _, _, _) => {
                ErrorResponse::new("DeserializeError".to_string())
            }
        },
        ClientRequestError::HelixRequestPostError(e) => match e {
            HelixRequestPostError::Error {
                status, message, ..
            } => ErrorResponse::with_status(message, status),
            HelixRequestPostError::Utf8Error(_, _, _) => {
                ErrorResponse::new("UTF8-Error".to_string())
            }
            HelixRequestPostError::DeserializeError(_, _, _, _) => {
                ErrorResponse::new("DeserializeError".to_string())
            }
            HelixRequestPostError::InvalidResponse { status, reason, .. } => {
                ErrorResponse::with_status(reason.to_string(), status)
            }
        },
        ClientRequestError::HelixRequestPatchError(e) => match e {
            HelixRequestPatchError::Error {
                status, message, ..
            } => ErrorResponse::with_status(message, status),
            HelixRequestPatchError::Utf8Error(_, _, _) => {
                ErrorResponse::new("UTF8-Error".to_string())
            }
            HelixRequestPatchError::DeserializeError(_, _, _, _) => {
                ErrorResponse::new("DeserializeError".to_string())
            }
            HelixRequestPatchError::InvalidResponse { status, reason, .. } => {
                ErrorResponse::with_status(reason.to_string(), status)
            }
        },
        ClientRequestError::HelixRequestDeleteError(e) => match e {
            HelixRequestDeleteError::Error {
                status, message, ..
            } => ErrorResponse::with_status(message, status),
            HelixRequestDeleteError::Utf8Error(_, _, _) => {
                ErrorResponse::new("UTF8-Error".to_string())
            }
        },
        ClientRequestError::Custom(e) => ErrorResponse::new(e.to_string()),
    }
    .into()
}
