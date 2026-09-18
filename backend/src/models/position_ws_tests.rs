use std::{ops::Add, time::Duration};

use crate::{
    app::{AppConfig, create_app},
    models::user::{NewUser, User},
    positions_handler::CLIENT_TIMEOUT,
    positions_server::PositionsServerHandle,
};
use futures::{SinkExt as _, StreamExt as _};
use tokio::time::sleep;

macro_rules! next_text_message {
    ($connection:expr) => {{
        let response = loop {
            let response = $connection.next().await.unwrap().unwrap();
            let response = match response {
                tokio_tungstenite::tungstenite::Message::Text(t) => t.to_string(),
                tokio_tungstenite::tungstenite::Message::Ping(_) => {
                    continue;
                }
                tokio_tungstenite::tungstenite::Message::Close(_) => panic!("Connection closed"),
                _ => panic!("Not a text frame"),
            };
            break response;
        };
        response
    }};
}

pub async fn position_ws_test(
    pool: &r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::SqliteConnection>>,
    app_config: &std::sync::Arc<AppConfig>,
    position_server_handle: &PositionsServerHandle,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app_router = create_app(pool, app_config, position_server_handle);
    tokio::spawn(async move {
        axum::serve(listener, app_router).await.unwrap();
    });

    let server_url = format!("http://{}", addr);
    let ws_url = format!("ws://{}", addr);
    let client = reqwest::Client::new();

    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { std::env::set_var("HEARTBEAT_INTERVAL", "3") };

    // Create an user
    let user_id = create_user(&client, &server_url).await;

    // Check that using the wrong token gives an unauthorized error on websockets endpoint
    let ws_wrong_token_endpoint = format!("{ws_url}/api/positions/ws?user_id={user_id}&token=0102");
    let res = tokio_tungstenite::connect_async(&ws_wrong_token_endpoint).await;
    match res {
        Err(tokio_tungstenite::tungstenite::Error::Http(resp)) => {
            assert_eq!(resp.status(), 401);
            let body = resp
                .body()
                .as_ref()
                .map(|b| String::from_utf8_lossy(b).to_string())
                .unwrap_or_default();
            assert_eq!(body, "Wrong token!");
        }
        _ => panic!("Expected HTTP 401 Unauthorized error"),
    }

    // Open a websocket connection and wait for receiving a position update
    let ws_endpoint = format!("{ws_url}/api/positions/ws?user_id={user_id}&token=0101");
    let (mut connection, _) = tokio_tungstenite::connect_async(&ws_endpoint)
        .await
        .unwrap();

    connection
        .send(tokio_tungstenite::tungstenite::Message::Text("Echo".into()))
        .await
        .unwrap();

    let response = connection.next().await.unwrap().unwrap();
    assert_eq!(
        response,
        tokio_tungstenite::tungstenite::Message::Text("Echo".into())
    );

    // Create a position
    let res = client
        .post(format!("{server_url}/api/positions"))
        .bearer_auth("0101")
        .header("content-type", "application/json")
        .body(format!(
            r#"[{{"user_id":{},"latitude":45.74846,"longitude":4.84671,"source":"GPS","battery_level":50,"sport_mode":false}}]"#,
            user_id
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    let response = next_text_message!(connection);
    assert!(response.contains("latitude"));

    // Create a position for another user
    let other_user_id = create_user(&client, &server_url).await;
    client
        .post(format!("{server_url}/api/positions"))
        .bearer_auth("0101")
        .header("content-type", "application/json")
        .body(format!(
            r#"[{{"user_id":{},"latitude":45.74846,"longitude":4.84671,"source":"GPS","battery_level":50,"sport_mode":false}}]"#,
            other_user_id
        ))
        .send()
        .await
        .unwrap();

    // Check that we did not get a position
    let response = connection.next().await.unwrap().unwrap();
    assert!(matches!(
        response,
        tokio_tungstenite::tungstenite::Message::Ping(_)
    ));

    // Check that one client is connected
    test_connected(&client, &server_url, 1).await;

    sleep(Duration::from_secs(2)).await; // To allow for position deduplication

    // Connect another websocket client
    let (mut connection2, _) = tokio_tungstenite::connect_async(&ws_endpoint)
        .await
        .unwrap();

    // Create a position
    client
        .post(format!("{server_url}/api/positions"))
        .bearer_auth("0101")
        .header("content-type", "application/json")
        .body(format!(
            r#"[{{"user_id":{},"latitude":45.12345,"longitude":4.84671,"source":"GPS","battery_level":50,"sport_mode":false}}]"#,
            user_id
        ))
        .send()
        .await
        .unwrap();

    // Check that two clients are connected
    test_connected(&client, &server_url, 2).await;

    // Check that both connexions get the new position
    let response = next_text_message!(connection);
    assert!(response.contains("12345"));

    let response = next_text_message!(connection2);
    assert!(response.contains("12345"));

    // Wait for connexions timeout
    sleep(CLIENT_TIMEOUT.add(Duration::from_secs(3))).await;

    // Check that connexions were closed
    while connection.next().await.is_some() {}
    while connection2.next().await.is_some() {}

    sleep(Duration::from_millis(200)).await;

    // Check that no clients are connected anymore
    test_connected(&client, &server_url, 0).await;

    client
        .delete(format!("{server_url}/api/positions"))
        .bearer_auth("0101")
        .send()
        .await
        .unwrap();

    // Get a share token
    let share_token = client
        .get(format!("{server_url}/api/token?user_id={user_id}"))
        .bearer_auth("0101")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let share_token = urlencoding::encode(&share_token);
    let ws_share_endpoint =
        format!("{ws_url}/api/positions/ws?user_id={user_id}&token={share_token}");
    let (mut connection, _) = tokio_tungstenite::connect_async(&ws_share_endpoint)
        .await
        .unwrap();

    connection
        .send(tokio_tungstenite::tungstenite::Message::Text(
            "Echo with share token".into(),
        ))
        .await
        .unwrap();

    let response = next_text_message!(connection);
    assert_eq!(response, "Echo with share token");
}

async fn create_user(client: &reqwest::Client, server_url: &str) -> i32 {
    client
        .post(format!("{server_url}/api/users"))
        .bearer_auth("0101")
        .json(&NewUser {
            name: "user".to_owned(),
            surname: "user".to_owned(),
        })
        .send()
        .await
        .unwrap()
        .json::<User>()
        .await
        .unwrap()
        .id
}

async fn test_connected(client: &reqwest::Client, server_url: &str, nb: usize) {
    let body = client
        .get(format!("{server_url}/api/positions/ws_count"))
        .bearer_auth("0101")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(body, nb.to_string());
}
