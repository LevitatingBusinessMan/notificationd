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
	// Show connected clients
	Who,
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
    let uid = if cli.user {
      nix::unistd::getuid()  
    } else {
        nix::unistd::Uid::from_raw(0)
    };
    let addr = levitating_notificationd::address(uid);
    match cli.command {
        Command::Status => {
            let mut client = connect(&addr)?;
            let status = client.status().call()?;
            println!("Local socket: {addr}");
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
        },
        Command::Who => {
            let mut client = connect(&addr)?;
            let who = client.who().call()?;
            println!("Connected clients:");
            for c in who.clients {
                println!("{} {} {}", c.login, if c.consume { "CONSUME" } else { &" ".repeat("CONSUME".len()) }, c.address);
            }
        },
    }
    Ok(())
}

fn connect(addr: &str) -> anyhow::Result<levitating_notificationd::VarlinkClient> {
    let conn = Connection::with_address(addr).context(format!("failed connecting to {addr}"))?;
    Ok(levitating_notificationd::VarlinkClient::new(conn))
}
