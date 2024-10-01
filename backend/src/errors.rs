use std::time::SystemTimeError;

use actix_web::error::BlockingError;
use actix_web::error::PayloadError;
use actix_web::error::ResponseError;
use actix_web::HttpResponse;

#[derive(Debug)]
pub enum ServerError {
    R2D2,
    Blocking,
    Other(String),
    Diesel,
    DieselNotFound,
    DieselDatabaseError(String),
    Image(String),
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::R2D2 => write!(f, "R2D2 error"),
            ServerError::Blocking => write!(f, "Blocking error"),
            ServerError::Diesel => write!(f, "Diesel error"),
            ServerError::DieselNotFound => write!(f, "Item not found"),
            ServerError::DieselDatabaseError(m) => write!(f, "{}", m),
            ServerError::Image(m) => write!(f, "Image error: {}", m),
            ServerError::Other(m) => write!(f, "Error: {}", m),
        }
    }
}

impl std::error::Error for ServerError {}

impl ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServerError::R2D2 => HttpResponse::InternalServerError().body("R2D2 error"),
            ServerError::Blocking => HttpResponse::InternalServerError().body("Blocking error"),
            ServerError::Diesel => HttpResponse::InternalServerError().body("Diesel error"),
            ServerError::DieselNotFound => HttpResponse::NotFound().body("Item not found"),
            ServerError::DieselDatabaseError(m) => HttpResponse::NotFound().body(m.clone()),
            ServerError::Image(m) => HttpResponse::InternalServerError().body(m.clone()),
            ServerError::Other(m) => {
                log::info!("{}", m);
                HttpResponse::NotFound().body(m.clone())
            }
        }
    }
}

impl From<r2d2::Error> for ServerError {
    fn from(_: r2d2::Error) -> ServerError {
        ServerError::R2D2
    }
}

impl From<diesel::result::Error> for ServerError {
    fn from(err: diesel::result::Error) -> ServerError {
        match err {
            diesel::result::Error::NotFound => ServerError::DieselNotFound,
            diesel::result::Error::DatabaseError(_, info) => {
                ServerError::DieselDatabaseError(info.message().to_string())
            }
            _ => ServerError::Diesel,
        }
    }
}

impl From<BlockingError> for ServerError {
    fn from(_: BlockingError) -> ServerError {
        ServerError::Blocking
    }
}

impl From<std::io::Error> for ServerError {
    fn from(err: std::io::Error) -> ServerError {
        ServerError::Image(err.to_string())
    }
}

impl From<PayloadError> for ServerError {
    fn from(err: PayloadError) -> ServerError {
        ServerError::Image(err.to_string())
    }
}

impl From<SystemTimeError> for ServerError {
    fn from(_: SystemTimeError) -> ServerError {
        ServerError::Other("could not get system time".to_owned())
    }
}

impl From<chacha20poly1305::Error> for ServerError {
    fn from(_: chacha20poly1305::Error) -> ServerError {
        ServerError::Other("could not perform chacha20poly1305 operation".to_owned())
    }
}

impl From<std::num::TryFromIntError> for ServerError {
    fn from(_: std::num::TryFromIntError) -> ServerError {
        ServerError::Other("could not convert value from integer".to_owned())
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(_: serde_json::Error) -> ServerError {
        ServerError::Other("could not marshall the value to json".to_owned())
    }
}
