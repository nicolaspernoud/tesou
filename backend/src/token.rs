use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use base64ct::{Base64, Encoding};
use chacha20poly1305::aead::Generate;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use log::debug;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{app::AppState, errors::ServerError};

#[derive(serde::Deserialize)]
pub struct Info {
    pub user_id: u16,
}

pub async fn get(
    State(state): State<AppState>,
    Query(info): Query<Info>,
) -> Result<Response, ServerError> {
    // Get the current time
    let time = SystemTime::now().duration_since(UNIX_EPOCH)?;
    debug!("Creating token, time = {:?}", time);
    let mut data = Vec::with_capacity(9);
    data.extend_from_slice(&time.as_secs().to_le_bytes());
    data.extend_from_slice(&info.user_id.to_le_bytes());

    // Get the main token
    let token = &state.config.bearer_token;

    // Derive it as a key
    let mut hasher = Sha256::new();
    hasher.update(token);
    let key: [u8; 32] = hasher.finalize().into();

    // Encrypt message
    let cipher = ChaCha20Poly1305::new(&key.into());
    let nonce = Nonce::generate(); // 96-bits; unique per message
    debug!("Creating token, nonce = {:?}", nonce);
    let mut ciphertext = cipher.encrypt(&nonce, data.as_ref())?;
    debug!("Creating token, ciphertext = {:?}", ciphertext);
    // Prepend message with nonce
    let mut ciphered = nonce.to_vec();
    ciphered.append(&mut ciphertext);
    debug!("Creating token, binary token = {:?}", ciphered);
    let encoded = Base64::encode_string(&ciphered);
    debug!("Creating token, base64 token = {:?}", encoded);

    // Respond
    Ok((StatusCode::OK, encoded).into_response())
}

#[cfg(test)]
pub async fn token_test(
    pool: &r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::SqliteConnection>>,
    app_config: &std::sync::Arc<crate::app::AppConfig>,
    position_server_handle: &crate::positions_server::PositionsServerHandle,
) {
    use crate::app::create_app;
    use crate::do_test;
    use axum::http::Method;
    use tower::ServiceExt;

    let app = create_app(pool, app_config, position_server_handle);

    // Get a token
    let share_token = do_test!(
        app,
        Method::GET,
        "/api/token?user_id=1",
        "",
        StatusCode::OK,
        ""
    );

    debug!("share_token = {}", share_token);

    // Try to use the share token with a get method (must pass)
    let req = http::Request::builder()
        .method(Method::GET)
        .uri("/api/users")
        .header("Authorization", format!("Bearer {share_token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Try to use the share token to get the positions for the user it was created for (must pass)
    let req = http::Request::builder()
        .method(Method::GET)
        .uri("/api/positions?user_id=1")
        .header("Authorization", format!("Bearer {share_token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Try to use the share token to get positions without an user id (must fail)
    let req = http::Request::builder()
        .method(Method::GET)
        .uri("/api/positions")
        .header("Authorization", format!("Bearer {share_token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Try to use the share token to get the positions for another user that the one it was created for (must fail)
    let req = http::Request::builder()
        .method(Method::GET)
        .uri("/api/positions?user_id=2")
        .header("Authorization", format!("Bearer {share_token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Try to use a random share token with a get method (must fail)
    let req = http::Request::builder()
        .method(Method::GET)
        .uri("/api/users")
        .header("Authorization", "Bearer 9HwHATU2Ifi7BFvZgbwTwQKsG5DlEIOqlAFRxX+steugCbDp")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Try to use the share token with a method altering the data (must fail)
    let req = http::Request::builder()
        .method(Method::DELETE)
        .uri("/api/users")
        .header("Authorization", format!("Bearer {share_token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Try to use the share token with a get a share token (must fail)
    let req = http::Request::builder()
        .method(Method::GET)
        .uri("/api/token?user_id=1")
        .header("Authorization", format!("Bearer {share_token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
