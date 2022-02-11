use actix_web::error::BlockingError;
use actix_web::error::ResponseError;
use actix_web::HttpResponse;
use log::info;

#[derive(Debug)]
pub enum ServerError {
    R2D2,
    Diesel,
    DieselNotFound,
    DieselDatabaseError(String),
    BlockingCanceled,
    Other(String),
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::R2D2 => write!(f, "R2D2 error"),
            ServerError::Diesel => write!(f, "Diesel error"),
            ServerError::DieselNotFound => write!(f, "Position not found"),
            ServerError::DieselDatabaseError(m) => write!(f, "{}", m),
            ServerError::BlockingCanceled => write!(f, "Blocking error"),
            ServerError::Other(m) => write!(f, "Error: {}", m),
        }
    }
}

impl std::error::Error for ServerError {}

impl ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServerError::R2D2 => HttpResponse::InternalServerError().body("R2D2 error"),
            ServerError::Diesel => HttpResponse::InternalServerError().body("Diesel error"),
            ServerError::DieselNotFound => HttpResponse::NotFound().body("Position not found"),
            ServerError::DieselDatabaseError(m) => HttpResponse::NotFound().body(m),
            ServerError::BlockingCanceled => {
                HttpResponse::InternalServerError().body("Blocking error")
            }
            ServerError::Other(m) => {
                info!("{}", m);
                HttpResponse::NotFound().body(m)
            }
        }
    }
}

impl From<r2d2::Error> for ServerError {
    fn from(_: r2d2::Error) -> ServerError {
        ServerError::R2D2
    }
}

fn server_error_from_diesel_error(err: diesel::result::Error) -> ServerError {
    match err {
        diesel::result::Error::NotFound => ServerError::DieselNotFound,
        diesel::result::Error::DatabaseError(_, info) => {
            ServerError::DieselDatabaseError(info.message().to_string())
        }
        _ => ServerError::Diesel,
    }
}

impl From<diesel::result::Error> for ServerError {
    fn from(err: diesel::result::Error) -> ServerError {
        server_error_from_diesel_error(err)
    }
}

impl From<BlockingError<diesel::result::Error>> for ServerError {
    fn from(err: BlockingError<diesel::result::Error>) -> ServerError {
        match err {
            BlockingError::Error(e) => server_error_from_diesel_error(e),
            BlockingError::Canceled => ServerError::BlockingCanceled,
        }
    }
}
