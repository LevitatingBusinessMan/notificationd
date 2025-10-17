use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

use anyhow::Context;
use clap::Parser;
use clap::ValueHint;
use notificationd::levitating_notificationd;
use notificationd::levitating_notificationd::VarlinkClientInterface;
use varlink::Connection;

#[derive(clap::Subcommand)]
enum Command {
    Status,
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(long)]
    user: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let addr = if cli.user {
        "unix:/run/user/1000/levitating.notificationd"
    } else {
        "unix:/run/levitating.notificationd"
    };
    match cli.command {
        Command::Status => {
            let mut client = connect(addr)?;
            let status = client.status().call()?;
            if let Some(server) = status.server {
                println!("Mode: server");
                println!("Connections: {}", server.connections);
                println!("Bind: {}", server.bind);
            }
            if let Some(client) = status.client {
                println!("Mode: client");
                println!("Server: {}", client.server);
                println!("Login: {}", client.login);
                println!("Consume: {}", client.consume);
            }
        }
    }
    Ok(())
}

fn connect(addr: &str) -> anyhow::Result<levitating_notificationd::VarlinkClient> {
    let conn = Connection::with_address(addr).context(format!("failed connecting to {addr}"))?;
    Ok(levitating_notificationd::VarlinkClient::new(conn))
}
