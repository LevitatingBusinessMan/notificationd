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
    #[arg(long, value_hint=ValueHint::Url)]
    address: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let addr = cli.address.unwrap_or("unix:/run/user/1000/levitating.notificationd".to_owned());
    match cli.command {
        Command::Status => {
            let mut client = connect(&addr)?;
            let status = client.status().call()?;
            println!("Connections: {}", status.connections);
        }
    }
    Ok(())
}

fn connect(addr: &str) -> anyhow::Result<levitating_notificationd::VarlinkClient> {
    let conn = Connection::with_address(addr).context(format!("failed connecting to {addr}"))?;
    Ok(levitating_notificationd::VarlinkClient::new(conn))
}
