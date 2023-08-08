use std::{
    env,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};

use actix::prelude::*;
use actix_web::{
    web::{self},
    Error, HttpRequest, HttpResponse, Responder,
};
use actix_web_actors::ws;
use log::{debug, info};

use super::position::Position;

static HEARTBEAT_INTERVAL: OnceLock<Duration> = OnceLock::new();
pub static CLIENT_TIMEOUT: OnceLock<Duration> = OnceLock::new();

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub Position);

#[derive(Debug)]
pub struct PositionConnexion {
    pub addr: Addr<PositionWebSocket>,
    pub id: usize,
    pub user_id: i32,
}

pub struct PositionWebSocket {
    id: usize,
    // Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT)  otherwise we drop connection.
    hb: Instant,
    connexions: Arc<Mutex<Vec<PositionConnexion>>>,
}

impl PositionWebSocket {
    fn new(id: usize, connexions: Arc<Mutex<Vec<PositionConnexion>>>) -> Self {
        Self {
            id,
            hb: Instant::now(),
            connexions,
        }
    }

    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        let conns = self.connexions.clone();
        let id = self.id;
        let heartbeat_interval = HEARTBEAT_INTERVAL
            .get_or_init(|| {
                Duration::from_secs(
                    env::var("HEARTBEAT_INTERVAL")
                        .unwrap_or("60".to_owned())
                        .parse::<u64>()
                        .unwrap_or(60),
                )
            })
            .to_owned();
        let client_timeout = CLIENT_TIMEOUT
            .get_or_init(|| heartbeat_interval * 2)
            .to_owned();
        ctx.run_interval(heartbeat_interval, move |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > client_timeout {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor and remove from connexion list
                ctx.stop();
                ctx.close(None);
                conns.lock().unwrap().retain(|e| e.id != id);

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }

    fn remove_from_connexions(&mut self) {
        debug!(
            "removing websocket connexion with id {} from active connexions",
            { self.id }
        );
        self.connexions.lock().unwrap().retain(|e| e.id != self.id);
    }
}

impl Actor for PositionWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

impl Handler<Message> for PositionWebSocket {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        match serde_json::to_string(&msg.0) {
            Ok(v) => {
                debug!("sent position to websocket connexions: {v}");
                ctx.text(v);
            }
            Err(_) => ctx.text("error serializing position to json"),
        }
    }
}

/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PositionWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // process websocket messages
        debug!("websocket message received: {msg:?}");
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
                self.remove_from_connexions();
            }
            _ => {
                ctx.close(None);
                ctx.stop();
                self.remove_from_connexions();
            }
        }
    }
}

pub struct WebSocketsState {
    pub index: Mutex<usize>,
    pub ws_actors: Arc<Mutex<Vec<PositionConnexion>>>,
}

/// WebSocket handshake and start `PositionWebSocket` actor.
pub async fn connect(
    req: HttpRequest,
    stream: web::Payload,
    ws_data: web::Data<WebSocketsState>,
    path: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let user_id = path.into_inner();
    let mut id = ws_data.index.lock().unwrap();
    *id += 1;
    let actor = PositionWebSocket::new(*id, ws_data.ws_actors.clone());
    match ws::WsResponseBuilder::new(actor, &req, stream).start_with_addr() {
        Ok((addr, resp)) => {
            info!("new websocket connexion with id: {id}, for user: {user_id}");
            let conn = PositionConnexion {
                addr,
                id: *id,
                user_id,
            };
            ws_data.ws_actors.lock().unwrap().push(conn);
            debug!("ws_actors are now: {:?}", ws_data.ws_actors);
            Ok(resp)
        }
        Err(e) => Err(e),
    }
}

pub async fn count(ws_data: web::Data<WebSocketsState>) -> impl Responder {
    ws_data.ws_actors.lock().unwrap().len().to_string()
}
