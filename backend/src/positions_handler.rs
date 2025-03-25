use std::{
    env,
    pin::pin,
    sync::LazyLock,
    time::{Duration, Instant},
};

use actix_web::Error;
use actix_web::{HttpRequest, HttpResponse, web};
use actix_web::{
    Responder,
    error::{self},
};
use actix_ws::AggregatedMessage;
use futures_util::{
    StreamExt as _,
    future::{Either, select},
};
use tokio::{sync::mpsc, task::spawn_local, time::interval};

use crate::{
    app::query_string_to_hashmap,
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
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
    user_id: UserId,
) {
    log::info!("new endpoint connection");

    let mut last_heartbeat = Instant::now();
    let mut interval = interval(*HEARTBEAT_INTERVAL);

    let (conn_tx, mut conn_rx) = mpsc::unbounded_channel();

    // unwrap: positions server is not dropped before the HTTP server
    let conn_id = positions_server.connect(conn_tx, user_id).await;

    let msg_stream = msg_stream
        .max_frame_size(128 * 1024)
        .aggregate_continuations()
        .max_continuation_size(2 * 1024 * 1024);

    let mut msg_stream = pin!(msg_stream);

    let close_reason = loop {
        // most of the futures we process need to be stack-pinned to work with select()
        let tick = pin!(interval.tick());
        let msg_rx = pin!(conn_rx.recv());

        // TODO: nested select is pretty gross for readability on the match
        let messages = pin!(select(msg_stream.next(), msg_rx));

        match select(messages, tick).await {
            // commands & messages received from client
            Either::Left((Either::Left((Some(Ok(msg)), _)), _)) => {
                log::debug!("msg: {msg:?}");

                match msg {
                    AggregatedMessage::Ping(bytes) => {
                        last_heartbeat = Instant::now();
                        session.pong(&bytes).await.unwrap();
                    }

                    AggregatedMessage::Pong(_) => {
                        last_heartbeat = Instant::now();
                    }

                    AggregatedMessage::Text(text) => {
                        process_text_msg(&positions_server, &mut session, &text, user_id).await;
                    }

                    AggregatedMessage::Binary(_bin) => {
                        log::warn!("unexpected binary message");
                    }

                    AggregatedMessage::Close(reason) => break reason,
                }
            }

            // client WebSocket stream error
            Either::Left((Either::Left((Some(Err(err)), _)), _)) => {
                log::error!("{}", err);
                break None;
            }

            // client WebSocket stream ended
            Either::Left((Either::Left((None, _)), _)) => break None,

            // positions messages received
            Either::Left((Either::Right((Some(positions_msg), _)), _)) => {
                session.text(positions_msg).await.unwrap();
            }

            // all connection's message senders were dropped
            Either::Left((Either::Right((None, _)), _)) => unreachable!(
                "all connection message senders were dropped; positions server may have panicked"
            ),

            // heartbeat internal tick
            Either::Right((_inst, _)) => {
                // if no heartbeat ping/pong received recently, close the connection
                if Instant::now().duration_since(last_heartbeat) > *CLIENT_TIMEOUT {
                    log::info!(
                        "client has not sent heartbeat in over {:?}; disconnecting",
                        *CLIENT_TIMEOUT
                    );
                    break None;
                }

                // send heartbeat ping
                let _ = session.ping(b"").await;
            }
        };
    };

    positions_server.disconnect(conn_id);

    // attempt to close connection gracefully
    let _ = session.close(close_reason).await;
}

async fn process_text_msg(
    positions_server: &PositionsServerHandle,
    _session: &mut actix_ws::Session,
    text: &str,
    user_id: UserId,
) {
    positions_server.send_message(user_id, text).await
}

// handshake and start WebSocket handler with heartbeats
pub async fn positions_ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    chat_server: web::Data<PositionsServerHandle>,
) -> Result<HttpResponse, Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

    // get user id from request
    let user_id = query_string_to_hashmap(req.query_string())
        .get("user_id")
        .ok_or(error::ErrorBadRequest("no user_id must in query"))?
        .parse::<u16>()
        .map_err(|_| error::ErrorBadRequest("the user_id must be a number"))?;
    // spawn websocket handler (and don't await it) so that the response is returned immediately
    spawn_local(positions_ws(
        (**chat_server).clone(),
        session,
        msg_stream,
        user_id,
    ));

    Ok(res)
}

pub async fn count(chat_server: web::Data<PositionsServerHandle>) -> impl Responder {
    chat_server.count().await.to_string()
}
