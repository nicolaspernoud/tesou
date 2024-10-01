//! A multi-user positions server.

use std::{
    collections::{HashMap, HashSet},
    io,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use rand::{thread_rng, Rng as _};
use tokio::sync::{mpsc, oneshot};

// connection ID
pub type ConnId = usize;

// user ID
pub type UserId = u16;

// message sent to a user/client
pub type Msg = String;

// a command received by the [`PositionsServer`]
#[derive(Debug)]
enum Command {
    Connect {
        user: UserId,
        conn_tx: mpsc::UnboundedSender<Msg>,
        res_tx: oneshot::Sender<ConnId>,
    },

    Disconnect {
        conn: ConnId,
    },

    Message {
        msg: Msg,
        user_id: UserId,
        res_tx: oneshot::Sender<()>,
    },

    Count {
        res_tx: oneshot::Sender<usize>,
    },
}

// a multi-user positions server
// call and spawn [`run`](Self::run) to start processing commands.
#[derive(Debug)]
pub struct PositionsServer {
    // map of connection IDs to their message receivers.
    sessions: HashMap<ConnId, mpsc::UnboundedSender<Msg>>,

    // map of user id to participant IDs listening to that user positions updates
    users: HashMap<UserId, HashSet<ConnId>>,

    // tracks total number of historical connections established.
    visitor_count: Arc<AtomicUsize>,

    // command receiver
    cmd_rx: mpsc::UnboundedReceiver<Command>,
}

impl PositionsServer {
    pub fn new() -> (Self, PositionsServerHandle) {
        // create empty server
        let users = HashMap::with_capacity(4);

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        (
            Self {
                sessions: HashMap::new(),
                users,
                visitor_count: Arc::new(AtomicUsize::new(0)),
                cmd_rx,
            },
            PositionsServerHandle { cmd_tx },
        )
    }

    async fn send_message(&self, user: &UserId, msg: impl Into<Msg>) {
        if let Some(sessions) = self.users.get(user) {
            let msg = msg.into();
            for conn_id in sessions {
                if let Some(tx) = self.sessions.get(conn_id) {
                    // errors if client disconnected abruptly and hasn't been timed-out yet
                    let _ = tx.send(msg.clone());
                }
            }
        }
    }

    //Register new session and assign unique ID to this session
    async fn connect(&mut self, tx: mpsc::UnboundedSender<Msg>, user_id: UserId) -> ConnId {
        log::info!("endpoint connected");

        // register session with random connection ID
        let id = thread_rng().gen::<ConnId>();
        self.sessions.insert(id, tx);

        // Join the endpoints listening to the target user
        self.users.entry(user_id).or_default().insert(id);

        self.visitor_count.fetch_add(1, Ordering::SeqCst);

        // send id back
        id
    }

    //Unregister connection from user map and broadcast disconnection message.
    async fn disconnect(&mut self, conn_id: ConnId) {
        log::info!("endpoint disconnected");

        // remove sender
        self.sessions.remove(&conn_id);
        // remove session from all users
        for (_, sessions) in &mut self.users {
            sessions.remove(&conn_id);
        }

        self.visitor_count.fetch_sub(1, Ordering::SeqCst);
    }

    pub async fn run(mut self) -> io::Result<()> {
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                Command::Connect {
                    conn_tx,
                    user,
                    res_tx,
                } => {
                    let conn_id = self.connect(conn_tx, user).await;
                    let _ = res_tx.send(conn_id);
                }

                Command::Disconnect { conn } => {
                    self.disconnect(conn).await;
                }

                Command::Message {
                    user_id,
                    msg,
                    res_tx,
                } => {
                    self.send_message(&user_id, msg).await;
                    let _ = res_tx.send(());
                }

                Command::Count { res_tx } => {
                    let count = self.visitor_count.load(Ordering::SeqCst);
                    let _ = res_tx.send(count);
                }
            }
        }

        Ok(())
    }
}

// handle and command sender for positions server
#[derive(Debug, Clone)]
pub struct PositionsServerHandle {
    cmd_tx: mpsc::UnboundedSender<Command>,
}

impl PositionsServerHandle {
    //Register client message sender and obtain connection ID.
    pub async fn connect(&self, conn_tx: mpsc::UnboundedSender<Msg>, user: UserId) -> ConnId {
        let (res_tx, res_rx) = oneshot::channel();

        // unwrap: positions server should not have been dropped
        self.cmd_tx
            .send(Command::Connect {
                conn_tx,
                res_tx,
                user,
            })
            .unwrap();

        // unwrap: positions server does not drop out response channel
        res_rx.await.unwrap()
    }

    // broadcast message
    pub async fn send_message(&self, user_id: UserId, msg: impl Into<Msg>) {
        let (res_tx, res_rx) = oneshot::channel();

        // unwrap: positions server should not have been dropped
        self.cmd_tx
            .send(Command::Message {
                user_id,
                msg: msg.into(),
                res_tx,
            })
            .unwrap();

        // unwrap: positions server does not drop our response channel
        res_rx.await.unwrap();
    }

    // unregister message sender and broadcast disconnection message to current user
    pub fn disconnect(&self, conn: ConnId) {
        // unwrap: positions server should not have been dropped
        self.cmd_tx.send(Command::Disconnect { conn }).unwrap();
    }

    pub async fn count(&self) -> usize {
        let (res_tx, res_rx) = oneshot::channel();

        // unwrap: positions server should not have been dropped
        self.cmd_tx.send(Command::Count { res_tx }).unwrap();

        // unwrap: positions server does not drop our response channel
        res_rx.await.unwrap()
    }
}
