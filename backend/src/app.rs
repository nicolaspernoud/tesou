use std::collections::HashMap;

use actix_web::error::{ErrorForbidden, ErrorUnauthorized};
use actix_web::http::Method;
use actix_web::{dev::ServiceRequest, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use base64ct::{Base64, Encoding};
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::consts::U12;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305,
};
use log::debug;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
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

const SHARE_TOKEN_DURATION: u64 = 2 * 60 * 60;

pub async fn share_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let app_config = req
        .app_data::<actix_web::web::Data<AppConfig>>()
        .expect("Could not get token configuration");
    if app_config.bearer_token == credentials.token() {
        Ok(req)
    } else {
        return Err((
            ErrorForbidden("a share token cannot be used to get a share token"),
            req,
        ));
    }
}

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let app_config = req
        .app_data::<actix_web::web::Data<AppConfig>>()
        .expect("Could not get token configuration");
    if app_config.bearer_token == credentials.token() {
        Ok(req)
    } else {
        // CHECK THE METHOD (GET ONLY ACCEPTED)
        if req.method() != Method::GET {
            return Err((
                ErrorForbidden("share token cannot be use to alter data"),
                req,
            ));
        }
        // TRY TO DECRYPT THE TOKEN
        // Get the token as base64
        let base64_token = credentials.token();
        debug!("Getting token, base64 token = {:?}", base64_token);
        // Convert to &[u8]
        let binary_token = match Base64::decode_vec(&base64_token) {
            Ok(val) => val,
            Err(_) => {
                return Err((
                    ErrorUnauthorized("could not decode share token as base 64"),
                    req,
                ));
            }
        };
        debug!("Getting token, binary token = {:?}", binary_token);
        // Get the main token
        let token = &app_config.bearer_token;
        // Derive it as a key
        let mut hasher = Sha256::new();
        hasher.update(token);
        let key: [u8; 32] = hasher.finalize().into();
        // Decipher the value of the token
        let cipher = ChaCha20Poly1305::new(&key.into());
        if binary_token.len() < 12 {
            return Err((ErrorUnauthorized("Wrong token!"), req));
        }
        let nonce: GenericArray<_, U12> = GenericArray::clone_from_slice(&binary_token[..12]);
        let data = match cipher.decrypt(&nonce, &binary_token[12..]) {
            Ok(val) => val,
            Err(_) => {
                return Err((ErrorUnauthorized("could not decipher token data"), req));
            }
        };
        // Convert it to a duration (since unix epoch, little endian)
        let data: [u8; 8] = match data.try_into() {
            Ok(val) => val,
            Err(_) => {
                return Err((ErrorUnauthorized("could not convert data to time"), req));
            }
        };
        let token_time = u64::from_le_bytes(data);
        // Get the current time
        let time = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(val) => val.as_secs(),
            Err(_) => {
                return Err((ErrorUnauthorized("could not get system time"), req));
            }
        };
        // Check that no more than 2 hours passed since the creation of the token
        if time > token_time + SHARE_TOKEN_DURATION {
            return Err((ErrorUnauthorized("token is expired"), req));
        }

        Ok(req)
    }
}

#[macro_export]
macro_rules! create_app {
    ($pool:expr, $app_config:expr, $ws_state:expr) => {{
        use crate::models::position_ws::{connect, count};
        use crate::models::{position, user};
        use crate::token;
        use actix_cors::Cors;
        use actix_web::dev::Service;
        use actix_web::{error::InternalError, middleware, web, web::Data, App, HttpResponse};
        use actix_web_httpauth::middleware::HttpAuthentication;

        App::new()
            .app_data(Data::new($pool.clone()))
            .app_data(
                web::JsonConfig::default()
                    .limit(4096)
                    .error_handler(|err, _req| {
                        InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
                    }),
            )
            .app_data(Data::clone($app_config))
            .app_data(Data::clone($ws_state))
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
                web::resource("/api/positions/ws/{user_id}")
                    .route(web::get().to(connect))
                    .wrap_fn(|req, srv| {
                        let reference_token = req
                            .app_data::<web::Data<AppConfig>>()
                            .map(|data| &data.bearer_token);
                        if let Some(ref_token) = reference_token {
                            if req.query_string().contains(&format!("token={}", ref_token)) {
                                return srv.call(req);
                            }
                        }
                        return Box::pin(async {
                            Err(actix_web::error::ErrorUnauthorized("Wrong token!"))
                        });
                    }),
            )
            .service(
                web::scope("/api/positions")
                    .wrap(HttpAuthentication::bearer(crate::app::validator))
                    .route("/ws_count", web::get().to(count))
                    .service(position::read_filter)
                    .service(position::read)
                    .service(position::create)
                    .service(position::update)
                    .service(position::delete_all)
                    .service(position::delete)
                    .service(position::create_from_cid),
            )
            .service(
                web::scope("/api/token")
                    .wrap(HttpAuthentication::bearer(crate::app::share_validator))
                    .service(token::get),
            )
            .service(actix_files::Files::new("/", "./web").index_file("index.html"))
    }};
}
