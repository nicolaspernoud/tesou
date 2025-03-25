use std::{ops::Add, time::Duration};

use crate::{
    app::AppConfig,
    create_app,
    models::{
        position::Position,
        user::{NewUser, User},
    },
    positions_handler::CLIENT_TIMEOUT,
    positions_server::PositionsServerHandle,
};
use actix_web::web::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use tokio::time::sleep;

macro_rules! next_text_message {
    ($connection:expr) => {{
        let response = loop {
            let response = $connection.next().await.unwrap().unwrap();
            let response = match response {
                awc::ws::Frame::Text(t) => std::str::from_utf8(&t).unwrap().to_string(),
                awc::ws::Frame::Ping(_) => {
                    continue;
                }
                awc::ws::Frame::Close(_) => panic!("Connection closed"),
                _ => panic!("Not a text frame"),
            };
            break response;
        };
        response
    }};
}

pub async fn position_ws_test(
    pool: &r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::SqliteConnection>>,
    app_config: &actix_web::web::Data<AppConfig>,
    position_server_handle: &PositionsServerHandle,
) {
    let pool = pool.clone();
    let app_config = app_config.clone();
    let position_server_handle = position_server_handle.clone();
    let app = actix_test::start(move || create_app!(&pool, &app_config, &position_server_handle));

    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { std::env::set_var("HEARTBEAT_INTERVAL", "3") };

    // Create an user
    let user_id = create_user(&app).await;

    // Check that using the wrong token gives an unauthorized error on websockets endpoint
    let mut resp = app
        .get(format!("/api/positions/ws?user_id={user_id}&token=0102"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
    let body = resp.body().await.unwrap();
    let body = std::str::from_utf8(&body).unwrap().to_string();
    assert_eq!(body, "Wrong token!");

    // Open a websocket connection and wait for receiving a position update
    let (_resp, mut connection) = awc::Client::new()
        .ws(app.url(&format!("/api/positions/ws?user_id={user_id}&token=0101")))
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
        r#"[{{"user_id":{},"latitude":45.74846,"longitude":4.84671,"source":"GPS","battery_level":50,"sport_mode":false}}]"#,
        user_id
    )).await.unwrap().json::<Position>().await.unwrap();

    let response = next_text_message!(connection);
    assert!(response.contains("latitude"));

    // Create a position for another user
    let other_user_id = create_user(&app).await;
    app.post("/api/positions").bearer_auth("0101").content_type("application/json").send_body(format!(
            r#"[{{"user_id":{},"latitude":45.74846,"longitude":4.84671,"source":"GPS","battery_level":50,"sport_mode":false}}]"#,
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
        .ws(app.url(&format!("/api/positions/ws?user_id={user_id}&token=0101")))
        .connect()
        .await
        .unwrap();
    // Create a position
    app.post("/api/positions").bearer_auth("0101").content_type("application/json").send_body(format!(
        r#"[{{"user_id":{},"latitude":45.12345,"longitude":4.84671,"source":"GPS","battery_level":50,"sport_mode":false}}]"#,
        user_id
    )).await.unwrap().json::<Position>().await.unwrap();

    // Check that two clients are connected
    test_connected(&app, 2).await;

    // Check that both connexions get the new position
    let response = next_text_message!(connection);
    assert!(response.contains("12345"));

    let response = next_text_message!(connection2);
    assert!(response.contains("12345"));

    // Wait for connexions timeout
    sleep(CLIENT_TIMEOUT.add(Duration::from_secs(2))).await;

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
        &app.get(format!("/api/token?user_id={user_id}"))
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
    let share_token = urlencoding::encode(&share_token);
    let (_resp, mut connection) = awc::Client::new()
        .ws(app.url(&format!(
            "/api/positions/ws?user_id={user_id}&token={share_token}"
        )))
        .connect()
        .await
        .unwrap();

    connection
        .send(awc::ws::Message::Text("Echo with share token".into()))
        .await
        .unwrap();

    let response = next_text_message!(connection);
    assert_eq!(response, "Echo with share token");
}

async fn create_user(app: &actix_test::TestServer) -> i32 {
    app.post("/api/users")
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
        .id
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
