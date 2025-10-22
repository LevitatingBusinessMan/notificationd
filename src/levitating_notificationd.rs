// you might want to enable rust-analyzer.cargo.loadOutDirsFromCheck
include!(concat!(env!("OUT_DIR"), "/levitating.notificationd.rs"));

pub const SOCKET_NAME: &'static str = "levitating.notificationd";

pub fn address(uid: nix::unistd::Uid) -> String {
    let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
        let runtime_dir = if std::path::Path::new("/run").exists() {
            "/run"
        } else {
            "/var/run"
        };
        // let uid: nix::unistd::Uid = nix::unistd::getuid();
        if uid.is_root() {
            runtime_dir.to_owned()
        } else {
            format!("/{runtime_dir}/user/{}", uid.as_raw())
        }
    });
    format!("unix:{dir}/{SOCKET_NAME}")
}
