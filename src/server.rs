use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use anyhow;

use client::ClientHandle;

mod client;

pub struct ServerState {
    pub(self) clients: Vec<ClientHandle>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            clients: vec![],
        }
    }
}

#[derive(Clone)]
pub struct ServerHandle {
    state: Arc<Mutex<ServerState>>,
}

impl ServerHandle {
    pub fn new(state: ServerState) -> Self {
        Self {
            state: Arc::new(Mutex::new(state))
        }
    }
    pub fn add_client(&self, handle: ClientHandle) {
        self.state.lock().unwrap().clients.push(handle);
    }
    pub fn broadcast(&self, msg: String) {
        let state = self.state.lock().unwrap();
        for c in &state.clients {
            let _ = c.write(msg.clone());
        }
    }
    pub(self) fn listen_incoming(&self, listener: TcpListener) -> io::Result<()> {
        loop {
            let (stream, _peer) = listener.accept()?;
            self.add_client(ClientHandle::new(stream)?);
        }
    }
}

pub fn main(bind: String) -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6606")?;
    println!("Listening on {}", bind);
    
    let server_handle = ServerHandle::new(ServerState::new());
    server_handle.listen_incoming(listener)
}
