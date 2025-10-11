use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub struct ClientState {
    pub name: Option<String>,
}

impl ClientState {
    pub fn new() -> Self {
        ClientState {
            name: None,
        }
    }
}

#[derive(Clone)]
pub struct ClientHandle {
    pub peer: SocketAddr,
    pub state: Arc<Mutex<ClientState>>,
    write_channel: mpsc::Sender<String>,
}

impl ClientHandle {
    pub fn new(stream: TcpStream) -> io::Result<Self> {
        let peer = stream.peer_addr()?;
        let writer = stream.try_clone()?;
        let reader = BufReader::new(stream);
        let (tx, rx) = mpsc::channel();

        let state = Arc::new(Mutex::new(ClientState::new()));

        let handle = Self {
            peer,
            state,
            write_channel: tx,
        };

        let handle_clone = handle.clone();

        thread::spawn(move || {
            Self::writer(writer, rx).expect("Client handle writer error:");
        });

        thread::spawn(move || {
            Self::reader(reader, &handle).expect("Client handle writer error:");
        });

        Ok(handle_clone)
    }

    fn reader(reader: BufReader<TcpStream>, handle: &ClientHandle) -> anyhow::Result<()> {
        for line in reader.lines() {
            handle.write(line?)?;
        }
        Ok(())
    }

    fn writer(mut writer: TcpStream, rx: mpsc::Receiver<String>) -> anyhow::Result<()> {
        loop {
            let str = rx.recv()?;
            writer.write(str.as_bytes())?;
        }
    }

    pub fn write(&self, msg: String) -> Result<(), mpsc::SendError<String>> {
        self.write_channel.send(msg)
    }
}
