use std::{ops::Add, time::Duration};

use crate::{
    app::AppConfig,
    create_app,
    models::{
        position::Position,
        position_ws::CLIENT_TIMEOUT,
        user::{NewUser, User},
    },
};
use actix_web::web::Bytes;
use futures_util::{SinkExt as _, StreamExt as _};
use tokio::time::sleep;

use super::position_ws::WebSocketsState;

pub async fn position_ws_test(
    pool: &r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::SqliteConnection>>,
    app_config: &actix_web::web::Data<AppConfig>,
    ws_state: &actix_web::web::Data<WebSocketsState>,
) {
    let pool = pool.clone();
    let app_config = app_config.clone();
    let ws_state = ws_state.clone();
    let app = actix_test::start(move || create_app!(&pool, &app_config, &ws_state));

    std::env::set_var("HEARTBEAT_INTERVAL", "3");

    // Create an user
    let user_id = create_user(&app).await;

    // Check that using the wrong token gives an unauthorized error on websockets endpoint
    let mut resp = app
        .get(&format!("/api/positions/ws/{user_id}?token=0102"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
    let body = resp.body().await.unwrap();
    let body = std::str::from_utf8(&body).unwrap().to_string();
    assert_eq!(body, "Wrong token!");

    // Open a websocket connection and wait for receiving a position update
    let (_resp, mut connection) = awc::Client::new()
        .ws(app.url(&format!("/api/positions/ws/{user_id}?token=0101")))
        .connect()
        .await
        .unwrap();

    connection
        .send(awc::ws::Message::Text("Echo".into()))
        .await
        .unwrap();

    let response = connection.next().await.unwrap().unwrap();
    assert_eq!(response, awc::ws::Frame::Text("Echo".into()));

    // Create a position
    app.post("/api/positions").bearer_auth("0101").content_type("application/json").send_body(format!(
        r#"{{"user_id":{},"latitude":45.74846,"longitude":4.84671,"source":"GPS","battery_level":50,"sport_mode":false}}"#,
        user_id
    )).await.unwrap().json::<Position>().await.unwrap();

    let response = connection.next().await.unwrap().unwrap();
    let response = match response {
        awc::ws::Frame::Text(t) => std::str::from_utf8(&t).unwrap().to_string(),
        _ => panic!("Not a text frame"),
    };
    assert!(response.contains("latitude"));

    // Create a position for another user
    let other_user_id = create_user(&app).await;
    app.post("/api/positions").bearer_auth("0101").content_type("application/json").send_body(format!(
            r#"{{"user_id":{},"latitude":45.74846,"longitude":4.84671,"source":"GPS","battery_level":50,"sport_mode":false}}"#,
            other_user_id
        )).await.unwrap().json::<Position>().await.unwrap();

    // Check that we did not get a position
    let response = connection.next().await.unwrap().unwrap();
    assert_eq!(response, awc::ws::Frame::Ping(Bytes::from("")));

    // Check that one client is connected
    test_connected(&app, 1).await;

    sleep(Duration::from_secs(2)).await; // To allow for position deduplication

    // Connect another websocket client
    let (_resp, mut connection2) = awc::Client::new()
        .ws(app.url(&format!("/api/positions/ws/{user_id}?token=0101")))
        .connect()
        .await
        .unwrap();
    // Create a position
    app.post("/api/positions").bearer_auth("0101").content_type("application/json").send_body(format!(
        r#"{{"user_id":{},"latitude":45.12345,"longitude":4.84671,"source":"GPS","battery_level":50,"sport_mode":false}}"#,
        user_id
    )).await.unwrap().json::<Position>().await.unwrap();

    // Check that two clients are connected
    test_connected(&app, 2).await;

    // Check that both connexions get the new position
    let response = connection.next().await.unwrap().unwrap();
    let response = match response {
        awc::ws::Frame::Text(t) => std::str::from_utf8(&t).unwrap().to_string(),
        _ => panic!("Not a text frame"),
    };
    assert!(response.contains("12345"));

    let response = connection2.next().await.unwrap().unwrap();
    let response = match response {
        awc::ws::Frame::Text(t) => std::str::from_utf8(&t).unwrap().to_string(),
        _ => panic!("Not a text frame"),
    };
    assert!(response.contains("12345"));

    // Wait for connexions timeout
    let client_timeout = CLIENT_TIMEOUT
        .get_or_init(|| Duration::from_secs(0))
        .to_owned();
    sleep(client_timeout.add(Duration::from_secs(2))).await;

    // Check that connexions were closed
    loop {
        if connection.next().await.is_none() {
            break;
        }
    }

    // Check that no clients are connected anymore
    test_connected(&app, 0).await;

    app.delete("/api/positions")
        .bearer_auth("0101")
        .send()
        .await
        .unwrap();

    // Get a share token
    // Get a token
    let share_token = std::str::from_utf8(
        &app.get("/api/token")
            .bearer_auth("0101")
            .send()
            .await
            .unwrap()
            .body()
            .await
            .unwrap(),
    )
    .unwrap()
    .to_string();
    let (_resp, mut connection) = awc::Client::new()
        .ws(app.url(&format!("/api/positions/ws/{user_id}?token={share_token}")))
        .connect()
        .await
        .unwrap();

    connection
        .send(awc::ws::Message::Text("Echo with share token".into()))
        .await
        .unwrap();

    let response = connection.next().await.unwrap().unwrap();
    assert_eq!(
        response,
        awc::ws::Frame::Text("Echo with share token".into())
    );
}

async fn create_user(app: &actix_test::TestServer) -> i32 {
    let user_id = app
        .post("/api/users")
        .bearer_auth("0101")
        .send_json(&NewUser {
            name: "user".to_owned(),
            surname: "user".to_owned(),
        })
        .await
        .unwrap()
        .json::<User>()
        .await
        .unwrap()
        .id;
    user_id
}

async fn test_connected(app: &actix_test::TestServer, nb: usize) {
    let body = std::str::from_utf8(
        &app.get("/api/positions/ws_count")
            .bearer_auth("0101")
            .send()
            .await
            .unwrap()
            .body()
            .await
            .unwrap(),
    )
    .unwrap()
    .to_string();
    assert_eq!(body, nb.to_string());
}
