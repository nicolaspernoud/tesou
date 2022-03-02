use actix_web::error::BlockingError;
use actix_web::error::ResponseError;
use actix_web::HttpResponse;

#[derive(Debug)]
pub enum ServerError {
    R2D2,
    Blocking,
    Other(String),
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::R2D2 => write!(f, "R2D2 error"),
            ServerError::Blocking => write!(f, "Blocking error"),
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

impl From<BlockingError> for ServerError {
    fn from(_: BlockingError) -> ServerError {
        ServerError::Blocking
    }
}
