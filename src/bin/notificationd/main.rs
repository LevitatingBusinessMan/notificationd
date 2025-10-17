use anyhow::Context;
use clap::Parser;

mod client;
mod protocol;
mod server;
mod logging;
mod varlink;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "0.0.0.0:6606")]
    bind: String,
    #[arg(short, long)]
    client: Option<String>,
}

fn main() -> anyhow::Result<()> {
    logging::init().expect("Failed to initialize logging");
    let args = Args::parse();
    if let Some(server) = args.client {
        Ok(client::main(server)?)
    } else {
        Ok(server::main(args.bind)?)
    }
}
