use anyhow;
use std::io;
use std::io::BufRead;
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

use client::ClientHandle;

mod client;
mod database;

pub static NOTIFICATION_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn next_id() -> usize {
    NOTIFICATION_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

pub fn set_id(id: usize) {
    NOTIFICATION_COUNTER.store(id, std::sync::atomic::Ordering::Relaxed);
}

pub struct ServerState {
    pub(self) clients: Vec<ClientHandle>,
    pub(self) db: Option<rusqlite::Connection>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            clients: vec![],
            db: None,
        }
    }
}

#[derive(Clone)]
pub struct ServerHandle {
    /// Address this server is bound at
    pub bind: Arc<String>,
    /// Mutable state of the server
    pub(self) state: Arc<Mutex<ServerState>>,
}

impl ServerHandle {
    pub fn new(addr: String, state: ServerState) -> Self {
        Self {
            bind: Arc::new(addr),
            state: Arc::new(Mutex::new(state)),
        }
    }
    pub fn add_client(&self, handle: ClientHandle) {
        self.state.lock().unwrap().clients.push(handle);
    }
    pub fn broadcast(&self, msg: String) {
        let state = self.state.lock().unwrap();
        for c in &state.clients {
            let _ = c.write(&msg);
        }
    }
    pub fn broadcast_notification(&self, msg: String) -> u32 {
        let state = self.state.lock().unwrap();
        let mut n = 0;
        for c in &state.clients {
            if c.state.lock().unwrap().consume == true {
                if c.write(&msg).is_ok() {
                    n += 1;
                }
            }
        }
        n
    }
    pub(self) fn listen_incoming(&self, listener: TcpListener) -> io::Result<()> {
        loop {
            let (stream, _peer) = listener.accept()?;
            self.add_client(ClientHandle::new(stream, self.clone())?);
        }
    }
    pub fn clients_len(&self) -> usize {
        self.state.lock().unwrap().clients.len()
    }
    pub fn has_db(&self) -> bool {
        self.state.lock().unwrap().db.is_some()
    }
    pub fn who(&self) -> Vec<(String, std::net::SocketAddr, bool)> {
        let mut v = vec![];
        for client in &self.state.lock().unwrap().clients {
            let state = client.state.lock().unwrap();
            if let Some(login) = &state.name {
                v.push((login.to_string(), client.peer, state.consume));
            }
        }
        return v;
    }
}

pub fn main(bind: String) -> anyhow::Result<()> {
    let mut server_state = ServerState::new();
    let persistence = true;
    let db_path = "/tmp/notificationd.sqlite3";
    server_state.db = if persistence {
        let mut db = rusqlite::Connection::open(db_path)?;
        database::setup_database(&mut db)?;
        tracing::info!("Opened database {db_path}");
        Some(db)
    } else {
        None
    };

    let listener = TcpListener::bind(&bind)?;
    tracing::info!("Listening on {}", bind);

    let server_handle = ServerHandle::new(bind, server_state);

    crate::varlink::init(Some(server_handle.clone()))?;

    if sd_notify::booted()? {
        sd_notify::notify(false, &[sd_notify::NotifyState::Ready])?;
    }

    Ok(server_handle.listen_incoming(listener)?)
}
