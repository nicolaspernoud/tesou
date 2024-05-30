use actix_web::{get, web, HttpResponse, Responder, Result};
use base64ct::{Base64, Encoding};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305,
};
use log::debug;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{app::AppConfig, errors::ServerError};

#[derive(serde::Deserialize)]
pub struct Info {
    pub user_id: u16,
}

#[get("")]
pub async fn get(
    cfg: web::Data<AppConfig>,
    info: web::Query<Info>,
) -> Result<impl Responder, ServerError> {
    // Get the current time
    let time = SystemTime::now().duration_since(UNIX_EPOCH)?;
    debug!("Creating token, time = {:?}", time);
    let mut data = Vec::with_capacity(9);
    data.extend_from_slice(&time.as_secs().to_le_bytes());
    data.extend_from_slice(&info.user_id.to_le_bytes());

    // Get the main token
    let token = &cfg.bearer_token;

    // Derive it as a key
    let mut hasher = Sha256::new();
    hasher.update(token);
    let key: [u8; 32] = hasher.finalize().into();

    // Encrypt message
    let cipher = ChaCha20Poly1305::new(&key.into());
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng); // 96-bits; unique per message
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
    Ok(HttpResponse::Ok().body(encoded))
}

#[cfg(test)]
pub async fn token_test(
    pool: &r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::SqliteConnection>>,
    app_config: &actix_web::web::Data<AppConfig>,
    ws_state: &actix_web::web::Data<crate::models::position_ws::WebSocketsState>,
) {
    use crate::do_test;
    use actix_web::{
        http::{Method, StatusCode},
        test,
    };

    let mut app = test::init_service(crate::create_app!(pool, app_config, ws_state)).await;

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
    let req = test::TestRequest::get()
        .insert_header(("Authorization", format!("Bearer {share_token}")))
        .uri("/api/users")
        .to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 200);

    // Try to use the share token to get the positions for the user it was created for (must pass)
    let req = test::TestRequest::get()
        .insert_header(("Authorization", format!("Bearer {share_token}")))
        .uri("/api/positions?user_id=1")
        .to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 200);

    // Try to use the share token to get positions without an user id (must fail)
    let req = test::TestRequest::get()
        .insert_header(("Authorization", format!("Bearer {share_token}")))
        .uri("/api/positions")
        .to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 404);

    // Try to use the share token to get the positions for another user that the one it was created for (must fail)
    let req = test::TestRequest::get()
        .insert_header(("Authorization", format!("Bearer {share_token}")))
        .uri("/api/positions?user_id=2")
        .to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 403);

    // Try to use a random share token with a get method (must fail)
    let req = test::TestRequest::get()
        .insert_header((
            "Authorization",
            "Bearer 9HwHATU2Ifi7BFvZgbwTwQKsG5DlEIOqlAFRxX+steugCbDp",
        ))
        .uri("/api/users")
        .to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 403);

    // Try to use the share token with a method altering the data (must fail)
    let req = test::TestRequest::delete()
        .insert_header(("Authorization", format!("Bearer {share_token}")))
        .uri("/api/users")
        .to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 403);

    // Try to use the share token with a get a share token (must fail)
    let req = test::TestRequest::get()
        .insert_header(("Authorization", format!("Bearer {share_token}")))
        .uri("/api/token?user_id=1")
        .to_request();
    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), 403);
}
