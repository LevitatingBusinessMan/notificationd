use clap::Parser;

mod server;
mod protocol;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value="0.0.0.0:6606")]
    bind: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    Ok(server::main(args.bind)?)
}
