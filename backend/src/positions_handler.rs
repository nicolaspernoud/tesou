use std::{
    env,
    sync::LazyLock,
    time::{Duration, Instant},
};

use axum::{
    extract::{
        ws::{WebSocketUpgrade},
        RawQuery, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures_util::{SinkExt as _, StreamExt as _};
use tokio::{sync::mpsc, time::interval};

use crate::{
    app::{check_share_token, query_string_to_hashmap, AppState},
    positions_server::{PositionsServerHandle, UserId},
};

// how often heartbeat pings are sent
static HEARTBEAT_INTERVAL: LazyLock<Duration> = LazyLock::new(|| {
    Duration::from_secs(
        env::var("HEARTBEAT_INTERVAL")
            .unwrap_or("60".to_owned())
            .parse::<u64>()
            .unwrap_or(60),
    )
});

// how long before lack of client response causes a timeout
pub static CLIENT_TIMEOUT: LazyLock<Duration> = LazyLock::new(|| *HEARTBEAT_INTERVAL * 2);

// Echo text & binary messages received from the client, respond to ping messages, and monitor connection health to detect network issues and free up resources.
pub async fn positions_ws(
    positions_server: PositionsServerHandle,
    socket: axum::extract::ws::WebSocket,
    user_id: UserId,
) {
    log::info!("new endpoint connection");

    let mut last_heartbeat = Instant::now();
    let mut interval = interval(*HEARTBEAT_INTERVAL);
    interval.reset();

    let (conn_tx, mut conn_rx) = mpsc::unbounded_channel();

    let conn_id = positions_server.connect(conn_tx, user_id).await;

    let (mut ws_tx, mut ws_rx) = socket.split();

    loop {
        tokio::select! {
            // commands & messages received from client
            msg = ws_rx.next() => {
                match msg {
                    Some(Ok(axum::extract::ws::Message::Ping(bytes))) => {
                        last_heartbeat = Instant::now();
                        let _ = ws_tx.send(axum::extract::ws::Message::Pong(bytes)).await;
                    }
                    Some(Ok(axum::extract::ws::Message::Pong(_))) => {
                        last_heartbeat = Instant::now();
                    }
                    Some(Ok(axum::extract::ws::Message::Text(text))) => {
                        process_text_msg(&positions_server, &text, user_id).await;
                    }
                    Some(Ok(axum::extract::ws::Message::Binary(_))) => {
                        log::warn!("unexpected binary message");
                    }
                    Some(Ok(axum::extract::ws::Message::Close(_))) => {
                        break;
                    }
                    Some(Err(err)) => {
                        log::error!("{}", err);
                        break;
                    }
                    None => {
                        break;
                    }
                }
            }

            // positions messages received
            positions_msg = conn_rx.recv() => {
                match positions_msg {
                    Some(msg) => {
                        if ws_tx.send(axum::extract::ws::Message::Text(msg.into())).await.is_err() {
                            break;
                        }
                    }
                    None => {
                        unreachable!(
                            "all connection message senders were dropped; positions server may have panicked"
                        );
                    }
                }
            }

            // heartbeat internal tick
            _ = interval.tick() => {
                if Instant::now().duration_since(last_heartbeat) > *CLIENT_TIMEOUT {
                    log::info!(
                        "client has not sent heartbeat in over {:?}; disconnecting",
                        *CLIENT_TIMEOUT
                    );
                    break;
                }

                let _ = ws_tx.send(axum::extract::ws::Message::Ping(Vec::new().into())).await;
            }
        }
    }

    positions_server.disconnect(conn_id);
}

async fn process_text_msg(
    positions_server: &PositionsServerHandle,
    text: &str,
    user_id: UserId,
) {
    positions_server.send_message(user_id, text).await
}

pub async fn positions_ws_handler(
    ws: WebSocketUpgrade,
    RawQuery(raw_query): RawQuery,
    State(state): State<AppState>,
) -> Response {
    let query_str = raw_query.unwrap_or_default();
    let params = query_string_to_hashmap(&query_str);

    let user_id = match params.get("user_id") {
        Some(uid_str) => match uid_str.parse::<u16>() {
            Ok(uid) => uid,
            Err(_) => {
                return (StatusCode::BAD_REQUEST, "the user_id must be a number").into_response();
            }
        },
        None => {
            return (StatusCode::BAD_REQUEST, "no user_id must in query").into_response();
        }
    };

    let query_token = match params.get("token") {
        Some(t) => t,
        None => {
            return (StatusCode::UNAUTHORIZED, "could not parse query").into_response();
        }
    };

    let query_token = urlencoding::decode(query_token).unwrap_or_default();
    let ref_token = &state.config.bearer_token;

    if query_token != *ref_token {
        let query_user_id = params.get("user_id");
        let (ok, reason) = check_share_token(&query_token, ref_token, query_user_id);
        if !ok {
            return (StatusCode::UNAUTHORIZED, reason).into_response();
        }
    }

    ws.on_upgrade(move |socket| positions_ws(state.positions_server, socket, user_id))
}

pub async fn count(State(state): State<AppState>) -> String {
    state.positions_server.count().await.to_string()
}
