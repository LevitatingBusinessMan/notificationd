use std::{env, io, path::Path};
use varlink::*;
use notificationd::levitating_notificationd::{self, *};
use tracing::{error, warn, info, debug, trace};

use crate::server::ServerHandle;

struct VarlinkClientInfo {
    login: String,
    consume: bool,
    server: String,
}

struct VarlinkHandles {
    server: Option<ServerHandle>,
    client: Option<VarlinkClientInfo>,
}

pub const SOCKET_NAME: &'static str = "levitating.notificationd";

impl VarlinkInterface for VarlinkHandles {
    fn status(&self, call: &mut dyn Call_Status) -> varlink::Result<()> {
        todo!()
        // let state = self.server.state.lock().unwrap();
        //return call.reply(state.clients.len() as i64);
    }
}

fn addres() -> String {
    let dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
        let runtime_dir = if Path::new("/run").exists() {
            "/run"
        } else {
            "/var/run"
        };
        let uid: nix::unistd::Uid = nix::unistd::getuid();
        if uid.is_root() {
            runtime_dir.to_owned()
        } else {
            format!("/{runtime_dir}/user/{}", uid.as_raw())
        }
    });
    format!("unix:{dir}/{SOCKET_NAME}")
}

pub fn init(server: Option<ServerHandle>) -> io::Result<()> {
    let interface = levitating_notificationd::new(Box::new(VarlinkHandles {server}));
    let service = varlink::VarlinkService::new("levitating", "notificationd.service", env!("CARGO_PKG_VERSION"), "https//github.com/LevitatingBusinessMan/notificationd", vec![Box::new(interface)]);
    let config = varlink::ListenConfig::default();
    let thread = std::thread::Builder::new().name(String::from("varlink"));

    thread.spawn(move || {
        let res = varlink::listen(service, &addres(), &config);
        error!("listener quit: {res:?}");
    })?;

    info!("Varlink initialized on {}", addres());

    Ok(())
}
