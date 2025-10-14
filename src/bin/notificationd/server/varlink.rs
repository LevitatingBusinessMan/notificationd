use std::{env, io};
use varlink::*;
use notificationd::levitating_notificationd::{self, *};
use tracing::{error, warn, info, debug, trace};

use crate::server::ServerHandle;

struct LevitatingNotificationd {
    server: ServerHandle,
}

pub const SOCKET_NAME: &'static str = "levitating.notificationd";

impl VarlinkInterface for LevitatingNotificationd {
    fn status(&self, call: &mut dyn Call_Status) -> varlink::Result<()> {
        let state = self.server.state.lock().unwrap();
        return call.reply(state.clients.len() as i64);
    }
}

fn addres() -> String {
    let dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
        let uid: nix::unistd::Uid = nix::unistd::getuid();
        if uid.is_root() {
            "/run/".to_owned()
        } else {
            format!("/run/user/{}", uid.as_raw())
        }
    });
    format!("unix:{dir}/{SOCKET_NAME}")
}

pub fn init(server: ServerHandle) -> io::Result<()> {
    let interface = levitating_notificationd::new(Box::new(LevitatingNotificationd {server}));
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
