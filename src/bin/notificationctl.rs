use std::sync::Arc;
use std::sync::RwLock;

use anyhow::Context;
use clap::Parser;
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
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Status => {
            let mut client = connect()?;
            let status = client.status().call()?;
            println!("Connections: {}", status.connections);
        }
    }
    Ok(())
}

fn connect() -> anyhow::Result<levitating_notificationd::VarlinkClient> {
    let addr: &'static str = "unix:/run/user/1000/levitating.notificationd";
    let conn = Connection::with_address(addr).context(format!("failed connecting to {addr}"))?;
    Ok(levitating_notificationd::VarlinkClient::new(conn))
}
