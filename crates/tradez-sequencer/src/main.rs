use clap::Parser;

mod host;
mod server;

#[derive(Parser)]
pub struct Args {
    #[clap(long, default_value_t = 8545)]
    pub rpc_port: u16,

    #[clap(long, default_value_t = String::from("http://localhost:8732"))]
    pub smart_rollup_addr: String,

    #[clap(long, default_value_t = String::from("tradez-sequencer/"))]
    pub data_dir: String,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    server::launch_server(args.rpc_port, args.smart_rollup_addr, args.data_dir).await?;
    Ok(())
}
