use clap::Parser;

mod host;
mod server;

#[derive(Parser)]
pub struct Args {
    #[clap(long, default_value_t = 8545)]
    pub rpc_port: u16,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    server::launch_server(args.rpc_port)?;
    Ok(())
}
