use std::io;
use std::io::BufReader;
use std::io::prelude::*;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::notifications;
use crate::protocol;
use crate::protocol::parser;
use crate::protocol::parser::Message;
use crate::server;
use crate::server::ServerHandle;
use anyhow::anyhow;
use notifications::NotificationDetails;
use zbus::address::transport::Tcp;

pub struct ClientState {
    pub name: Option<String>,
    pub details: NotificationDetails,
    pub consume: bool,
}

impl ClientState {
    pub fn new() -> Self {
        ClientState {
            name: None,
            consume: false,
            details: NotificationDetails::new(),
        }
    }
}

#[derive(Clone)]
pub struct ClientHandle {
    pub peer: SocketAddr,
    pub state: Arc<Mutex<ClientState>>,
    // only used for closing
    stream: Arc<TcpStream>,
    pub server: ServerHandle,
    write_channel: mpsc::Sender<String>,
}

impl ClientHandle {
    pub fn new(stream: TcpStream, server: ServerHandle) -> io::Result<Self> {
        let peer = stream.peer_addr()?;
        let writer = stream.try_clone()?;
        let reader = BufReader::new(stream.try_clone()?);
        let (tx, rx) = mpsc::channel();

        let state = Arc::new(Mutex::new(ClientState::new()));

        let handle = Self {
            peer,
            state,
            server,
            stream: Arc::new(stream),
            write_channel: tx,
        };

        let handle_clone = handle.clone();

        thread::spawn(move || {
            Self::writer(writer, rx).expect("Client handle writer error");
        });

        thread::spawn(move || {
            Self::reader(reader, &handle).expect("Client handle writer error");
        });

        Ok(handle_clone)
    }

    fn reader(reader: BufReader<TcpStream>, handle: &ClientHandle) -> anyhow::Result<()> {
        for line in reader.lines() {
            let line = line?;
            match parser::line(&line, false) {
                Ok((_remaining, msg)) => handle.handle_message(msg)?,
                Err(_e) => handle.write("-ERR PARSE\r\n")?,
            }
        }
        Ok(())
    }

    fn writer(mut writer: TcpStream, rx: mpsc::Receiver<String>) -> anyhow::Result<()> {
        loop {
            let str = rx.recv()?;
            writer.write(str.as_bytes())?;
        }
    }

    pub fn write(&self, msg: &str) -> Result<(), mpsc::SendError<String>> {
        self.write_channel.send(msg.to_owned())
    }

    // to return an error here means to kill the connection
    pub fn handle_message(&self, msg: protocol::parser::Message) -> anyhow::Result<()> {
        println!("{msg:?}");

        if msg.sign.is_some() {
            self.write(&protocol::reply(
                msg.id,
                false,
                "ERR",
                vec!["INVALID_MESSAGE"],
                Some("You can't sent a reply message to a server."),
            ))?;
            return Ok(());
        }

        let cmd = msg.command.to_uppercase();
        match cmd.as_ref() {
            "LOGIN" => {
                let user = msg.arguments.get(0);
                let password = msg.arguments.get(1);

                match user {
                    None => self.write(&protocol::reply(
                        msg.id,
                        false,
                        "LOGIN",
                        vec!["MISSING_ARG"],
                        None,
                    ))?,
                    Some(user) => {
                        let user_ = &mut self.state.lock().unwrap().name;
                        match user_ {
                            Some(user_) => self.write(&protocol::reply(
                                msg.id,
                                false,
                                "LOGIN",
                                vec!["ALREADY_LOGGED_IN"],
                                Some(&format!(
                                    "You are already logged in as {}. Please reconnect.",
                                    user_
                                )),
                            ))?,
                            None => {
                                *user_ = Some(user.to_owned());
                                self.write(&protocol::reply(
                                    msg.id,
                                    true,
                                    "LOGIN",
                                    vec![],
                                    Some(&format!("Welcome {}", user)),
                                ))?;
                                println!("{} logged in as {}", self.peer, user);
                            }
                        };
                    }
                }
            }
            _ => {
                // user needs to be logged in first
                if self.state.lock().unwrap().name.is_none() {
                    self.write(&protocol::reply(
                        msg.id,
                        false,
                        "-ERR",
                        vec!["NO_LOGIN"],
                        Some("Please login first."),
                    ))?
                } else {
                    let user = self.state.lock().unwrap().name.clone().unwrap();
                    match cmd.as_ref() {
                        "TITLE" => match msg.trailing {
                            Some(title) => {
                                self.state.lock().unwrap().details.title = Some(title.clone())
                            }
                            None => self.write(&protocol::reply(
                                msg.id,
                                false,
                                "TITLE",
                                vec!["MISSING_TRAILING"],
                                None,
                            ))?,
                        },
                        "BODY" => match msg.trailing {
                            Some(body) => {
                                self.state.lock().unwrap().details.body = Some(body.clone())
                            }
                            None => self.write(&protocol::reply(
                                msg.id,
                                false,
                                "BODY",
                                vec!["MISSING_TRAILING"],
                                None,
                            ))?,
                        },
                        "SEND" => {
                            let title = self
                                .state
                                .lock()
                                .unwrap()
                                .details
                                .title
                                .clone()
                                .unwrap_or(String::from(""));
                            let body = self.state.lock().unwrap().details.body.clone().unwrap_or(String::from(""));
                            let notification_id = server::NOTIFICATION_COUNTER
                                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            let notify_msg = format!(
                                "NOTIFY_START {} {}\r\nTITLE: {}\r\nBODY: {}\r\nNOTIFY_END {}\r\n",
                                user, notification_id, title, body, notification_id
                            );

                            let n = self.server.broadcast_notification(notify_msg);
                            self.write(&protocol::reply(
                                msg.id,
                                true,
                                "SEND",
                                vec![&n.to_string()],
                                None,
                            ))?
                        }
                        "RESET" => {
                            self.state.lock().unwrap().details = NotificationDetails::new();
                        }
                        "VERSION" => self.write(&protocol::reply(
                            msg.id,
                            true,
                            "VERSION",
                            vec![env!("CARGO_PKG_VERSION")],
                            None,
                        ))?,
                        "CONSUME" => {
                            let on_bind = String::from("on");
                            let arg = msg.arguments.first().unwrap_or(&on_bind);

                            if arg == "on" || arg == "true" {
                                self.state.lock().unwrap().consume = true;
                                self.write(&protocol::reply(
                                    msg.id,
                                    true,
                                    "CONSUME",
                                    vec![arg],
                                    None,
                                ))?
                            } else if arg == "off" || arg == "false" {
                                self.state.lock().unwrap().consume = false;
                                self.write(&protocol::reply(
                                    msg.id,
                                    true,
                                    "CONSUME",
                                    vec![arg],
                                    None,
                                ))?
                            } else {
                                self.write(&protocol::reply(
                                    msg.id,
                                    false,
                                    "CONSUME",
                                    vec!["INVALID_ARG"],
                                    None,
                                ))?
                            }
                        },
                        "QUIT" => {
                            self.stream.shutdown(std::net::Shutdown::Both)?;
                        },
                        _ => self.write(&protocol::reply(
                            msg.id,
                            false,
                            "ERR",
                            vec!["UNKNOWN_CMD"],
                            Some(&format!("I do not know {}", &cmd)),
                        ))?,
                    }
                }
            }
        }
        Ok(())
    }
}
