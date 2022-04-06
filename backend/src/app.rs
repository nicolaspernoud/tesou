use std::collections::HashMap;

use actix_web::error::{self};
use actix_web::{dev::ServiceRequest, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use tokio::sync::Mutex;

pub struct AppConfig {
    pub bearer_token: String,
    pub open_cell_id_api_key: String,
    pub user_last_update: Mutex<HashMap<i32, i64>>,
}

impl AppConfig {
    pub fn new(token: String, api_key: String) -> Self {
        AppConfig {
            bearer_token: token,
            open_cell_id_api_key: api_key,
            user_last_update: Mutex::new(HashMap::new()),
        }
    }
}

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    let app_config = req
        .app_data::<actix_web::web::Data<AppConfig>>()
        .expect("Could not get token configuration");
    if app_config.bearer_token == credentials.token() {
        Ok(req)
    } else {
        Err(error::ErrorUnauthorized("Wrong token!"))
    }
}

#[macro_export]
macro_rules! create_app {
    ($pool:expr, $app_data:expr) => {{
        use crate::models::{position, user};
        use actix_cors::Cors;
        use actix_web::{error, middleware, web, web::Data, App, HttpResponse};
        use actix_web_httpauth::middleware::HttpAuthentication;

        App::new()
            .app_data(Data::new($pool.clone()))
            .app_data(
                web::JsonConfig::default()
                    .limit(4096)
                    .error_handler(|err, _req| {
                        error::InternalError::from_response(err, HttpResponse::Conflict().finish())
                            .into()
                    }),
            )
            .app_data(Data::clone($app_data))
            .wrap(Cors::permissive())
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api/users")
                    .wrap(HttpAuthentication::bearer(crate::app::validator))
                    .service(user::read_all)
                    .service(user::read)
                    .service(user::create)
                    .service(user::update)
                    .service(user::delete_all)
                    .service(user::delete),
            )
            .service(
                web::scope("/api/positions")
                    .wrap(HttpAuthentication::bearer(crate::app::validator))
                    .service(position::read_filter)
                    .service(position::read)
                    .service(position::create)
                    .service(position::update)
                    .service(position::delete_all)
                    .service(position::delete)
                    .service(position::create_from_cid),
            )
            .service(actix_files::Files::new("/", "./web").index_file("index.html"))
    }};
}
