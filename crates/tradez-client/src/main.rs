use clap::{Parser, Subcommand};
use jsonrpc_core_client::{RpcChannel, RpcResult, TypedClient};

pub mod wallet;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// URL of the sequencer server
    #[arg(short, long, default_value_t = String::from("http://localhost:8545"))]
    url: String,

    #[clap(subcommand)]
    app: AppSubcommand,
}

#[derive(Subcommand, Debug)]
#[command(author, version, about, long_about = None)]
enum AppSubcommand {
    /// Wallet management commands
    Wallet(Wallet),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Wallet {
    /// Directory path to the wallet
    #[arg(short, long, default_value_t = String::from("wallets"))]
    dirpath: String,

    /// Name of the wallet
    #[arg(short, long, default_value_t = String::from("test_wallet"))]
    name: String,

    #[arg(short, long, default_value_t = String::from("test"))]
    password: String,

    #[clap(subcommand)]
    command: WalletCommand,
}

#[derive(Subcommand, Debug)]
enum WalletCommand {
    /// Create a new wallet
    Create {},
    /// Get positions of a wallet
    GetPositions {},
    /// Balance of a wallet
    Balance {},
    /// Open a new position
    OpenPosition {
        /// Side of the position (0 = buy, 1 = sell)
        #[arg(short, long)]
        side: u8,
        /// Size of the position
        #[arg(short, long)]
        size: u64,
        /// Price of the position
        #[arg(short, long)]
        price: u64,
    },
    /// Close an existing position
    ClosePosition {
        /// ID of the position to close
        #[arg(short, long)]
        position_id: u64,
    },
}

#[derive(Clone)]
struct TestClient(TypedClient);

impl From<RpcChannel> for TestClient {
    fn from(channel: RpcChannel) -> Self {
        TestClient(channel.into())
    }
}

impl TestClient {
    fn open_position(
        &self,
        side: u8,
        size: u64,
        price: u64,
        signature: &str,
    ) -> impl std::future::Future<Output = RpcResult<String>> {
        self.0.call_method(
            "send_order",
            "String",
            (side, size, price, signature.to_string()),
        )
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let server_url = args.url;
    match args.app {
        AppSubcommand::Wallet(wallet_cmd) => {
            let wallet =
                wallet::load_wallet(&wallet_cmd.dirpath, &wallet_cmd.name, wallet_cmd.password);
            let jsonrpc_client: TestClient =
                jsonrpc_core_client::transports::http::connect(&server_url)
                    .await
                    .unwrap();

            match wallet_cmd.command {
                WalletCommand::Create {} => {
                    println!(
                        "Wallet created: {}/{}",
                        &wallet_cmd.dirpath.to_string(),
                        wallet_cmd.name
                    );
                }
                WalletCommand::GetPositions {} => {
                    println!("Fetching positions for wallet: {}", wallet_cmd.name);
                    // Implement fetching positions logic here
                }
                WalletCommand::Balance {} => {
                    println!("Fetching balance for wallet: {}", wallet_cmd.name);
                    // Implement balance fetching logic here
                }
                WalletCommand::OpenPosition { side, size, price } => {
                    println!(
                        "Opening position with side: {}, size: {}, price: {} for wallet: {}",
                        side, size, price, wallet_cmd.name
                    );
                    let signature = "dummy_signature"; // Replace with actual signature logic
                    let result = jsonrpc_client
                        .open_position(side, size, price, signature)
                        .await
                        .expect("RPC call failed");
                    println!("Result from server: {}", result);
                }
                WalletCommand::ClosePosition { position_id } => {
                    println!(
                        "Closing position with ID: {} for wallet: {}",
                        position_id, wallet_cmd.name
                    );
                    // Implement close position logic here
                }
            }
        }
    }
}
