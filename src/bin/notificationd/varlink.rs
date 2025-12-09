use std::{env, io, path::Path};
use varlink::*;
use notificationd::levitating_notificationd::{self, *};
use tracing::{error, warn, info, debug, trace};

use crate::server::ServerHandle;

struct VarlinkClientHandles {
    login: String,
    consume: bool,
    server: String,
}

struct VarlinkHandles {
    server: Option<ServerHandle>,
    //client: Option<VarlinkClientHandles>,
}

impl VarlinkInterface for VarlinkHandles {
    fn status(&self, call: &mut dyn Call_Status) -> varlink::Result<()> {
        let server = self.server.as_ref().map(|sh| {
            ServerStatus {
                bind: self.server.as_ref().unwrap().bind.to_string(),
                connections: sh.clients_len() as i64,
                persistent: sh.has_db()
            }
        });
        return  call.reply(server, None);
    }
    fn who(&self, call: &mut dyn Call_Who) -> varlink::Result<()> {
        if let Some(sh) = &self.server {
            let v = sh.who().iter().map(|(login, socket, consume)| WhoClient {
                login: login.to_string(),
                consume: *consume,
                address: socket.to_string(),
            }).collect();
            return call.reply(v);
        } else {
            return call.reply(vec![]);
        }
    }
}

pub fn init(server: Option<ServerHandle>) -> io::Result<()> {
    let interface = levitating_notificationd::new(Box::new(VarlinkHandles {server}));
    let service = varlink::VarlinkService::new("levitating", "notificationd.service", env!("CARGO_PKG_VERSION"), "https//github.com/LevitatingBusinessMan/notificationd", vec![Box::new(interface)]);
    let config = varlink::ListenConfig::default();
    let thread = std::thread::Builder::new().name(String::from("varlink"));
    let address = levitating_notificationd::address(nix::unistd::Uid::current());
    let address_clone = address.clone();

    thread.spawn(move || {
        let res = varlink::listen(service, &address, &config);
        error!("listener quit: {res:?}");
    })?;

    info!("Varlink initialized on {}", address_clone);

    Ok(())
}
