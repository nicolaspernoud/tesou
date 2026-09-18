use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use base64ct::{Base64, Encoding};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use log::debug;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tower_http::{cors::CorsLayer, services::{ServeDir, ServeFile}};

use crate::{
    db_options::DbPool,
    models::{position, sport_mode, user},
    positions_handler::{count, positions_ws_handler},
    positions_server::PositionsServerHandle,
    token,
};

pub struct AppConfig {
    pub bearer_token: String,
    pub open_cell_id_api_key: Option<String>,
    pub user_last_update: Mutex<HashMap<i32, i64>>,
    pub sport_mode_toggle_users: Mutex<Vec<i32>>,
}

impl AppConfig {
    pub fn new(token: String, api_key: Option<String>) -> Self {
        AppConfig {
            bearer_token: token,
            open_cell_id_api_key: api_key,
            user_last_update: Mutex::new(HashMap::new()),
            sport_mode_toggle_users: Mutex::new(Vec::new()),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub config: Arc<AppConfig>,
    pub positions_server: PositionsServerHandle,
}

const SHARE_TOKEN_DURATION: u64 = 2 * 60 * 60;

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

pub async fn share_validator(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let token = extract_bearer_token(req.headers());
    if let Some(token) = token {
        if state.config.bearer_token == token {
            return next.run(req).await;
        }
    }
    (
        StatusCode::FORBIDDEN,
        "a share token cannot be used to get a share token",
    )
        .into_response()
}

pub async fn validator(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let token = match extract_bearer_token(req.headers()) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    let main_token = &state.config.bearer_token;
    if main_token == token {
        return next.run(req).await;
    }

    if req.method() != Method::GET {
        return (
            StatusCode::FORBIDDEN,
            "share token cannot be use to alter data",
        )
            .into_response();
    }

    let query_str = req.uri().query().unwrap_or("");
    let params = query_string_to_hashmap(query_str);
    let user_id = params.get("user_id");
    let (ok, reason) = check_share_token(token, main_token, user_id);
    if ok {
        next.run(req).await
    } else {
        (StatusCode::FORBIDDEN, reason).into_response()
    }
}

pub fn query_string_to_hashmap(q_string: &str) -> HashMap<String, String> {
    let mut params: HashMap<String, String> = HashMap::new();
    for pair in q_string.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            params.insert(key.to_owned(), value.to_owned());
        }
    }
    params
}

pub fn check_share_token(
    base64_token: &str,
    main_token: &str,
    user_id: Option<&String>,
) -> (bool, &'static str) {
    debug!("Getting token, base64 token = {:?}", base64_token);
    let binary_token = match Base64::decode_vec(base64_token) {
        Ok(val) => val,
        Err(_) => {
            return (false, "could not decode share token as base 64");
        }
    };
    debug!("Getting token, binary token = {:?}", binary_token);
    let mut hasher = Sha256::new();
    hasher.update(main_token);
    let key: [u8; 32] = hasher.finalize().into();
    let cipher = ChaCha20Poly1305::new(&key.into());
    if binary_token.len() < 12 {
        return (false, "Wrong token!");
    }
    let nonce: Nonce = match Nonce::try_from(&binary_token[..12]) {
        Ok(n) => n,
        Err(_) => {
            return (false, "could not decode token into nonce");
        }
    };
    let data = match cipher.decrypt(&nonce, &binary_token[12..]) {
        Ok(val) => val,
        Err(_) => {
            return (false, "could not decipher token data");
        }
    };
    let (time_data, id) = data.split_at(8);
    let time_data: [u8; 8] = match time_data.try_into() {
        Ok(val) => val,
        Err(_) => {
            return (false, "could not extract time from data");
        }
    };
    let token_time = u64::from_le_bytes(time_data);
    let id: [u8; 2] = match id.try_into() {
        Ok(val) => val,
        Err(_) => {
            return (false, "could not extract user id from data");
        }
    };
    let id = u16::from_le_bytes(id);
    let time = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(val) => val.as_secs(),
        Err(_) => {
            return (false, "could not get system time");
        }
    };
    if time > token_time + SHARE_TOKEN_DURATION {
        return (false, "token is expired");
    }
    if let Some(user_id) = user_id.map(|x| x.parse::<u16>().unwrap_or(0))
        && user_id != id
    {
        return (false, "user ids don't match");
    }
    (true, "")
}

pub fn build_router(state: AppState) -> Router {
    let user_routes = Router::new()
        .route("/", get(user::read_all).post(user::create).delete(user::delete_all))
        .route("/{oid}", get(user::read).put(user::update).delete(user::delete))
        .layer(middleware::from_fn_with_state(state.clone(), validator));

    let position_routes = Router::new()
        .route("/ws_count", get(count))
        .route("/", get(position::read_filter).post(position::create).delete(position::delete_all))
        .route("/{oid}", get(position::read).put(position::update).delete(position::delete))
        .route("/cid/{uid}", post(position::create_from_cid))
        .layer(middleware::from_fn_with_state(state.clone(), validator));

    let sport_mode_routes = Router::new()
        .route("/toggle/{user_id}", post(sport_mode::toggle_sport_mode))
        .layer(middleware::from_fn_with_state(state.clone(), validator));

    let token_routes = Router::new()
        .route("/", get(token::get))
        .layer(middleware::from_fn_with_state(state.clone(), share_validator));

    let serve_dir = ServeDir::new("./web").fallback(ServeFile::new("./web/index.html"));

    Router::new()
        .nest("/api/users", user_routes)
        .nest("/api/positions", position_routes)
        .route("/api/positions/ws", get(positions_ws_handler))
        .nest("/api/sport-mode", sport_mode_routes)
        .nest("/api/token", token_routes)
        .fallback_service(serve_dir)
        .layer(CorsLayer::permissive())
        .layer(axum::extract::DefaultBodyLimit::max(4096))
        .with_state(state)
}

pub fn create_app(
    pool: &DbPool,
    app_config: &Arc<AppConfig>,
    positions_server_tx: &PositionsServerHandle,
) -> Router {
    let state = AppState {
        pool: pool.clone(),
        config: app_config.clone(),
        positions_server: positions_server_tx.clone(),
    };

    build_router(state)
}
