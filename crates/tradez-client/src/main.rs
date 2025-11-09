use clap::{Parser, Subcommand};
use jsonrpsee::http_client::HttpClientBuilder;
use rlp::Encodable;
use tradez_types::{
    api::TradezRpcClient,
    position::{APIOrder, CancelOrder, Faucet},
};

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
    /// Get state infos
    Get(GetInfos),
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
    /// Faucet tokens to a wallet
    Faucet {
        /// Amount to faucet
        #[arg(short, long)]
        amount: u64,

        /// Currency to faucet (0 = USDC, 1 = XTZ)
        #[arg(short, long, default_value_t = 0u8)]
        currency: u8,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct GetInfos {
    #[clap(subcommand)]
    command: GetInfosCommand,
}

#[derive(Subcommand, Debug)]
enum GetInfosCommand {
    /// Get orderbook state
    OrderbookState {},
    /// Get balances of an address
    Balances {
        /// Address to get balances for
        /// Hexadecimal string representation of the address
        #[arg(short, long)]
        address: String,
    },
    /// Get orders of an address
    Orders {
        /// Address to get orders for
        /// Hexadecimal string representation of the address
        #[arg(short, long)]
        address: String,
    },
    /// Get the trade history
    History {},
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let server_url = args.url;
    let client = HttpClientBuilder::new().build(server_url).unwrap();
    match args.app {
        AppSubcommand::Wallet(wallet_cmd) => {
            let wallet = wallet::Wallet::load_wallet(
                &wallet_cmd.dirpath,
                &wallet_cmd.name,
                wallet_cmd.password,
            );

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
                    let api_order = APIOrder {
                        side: if side == 0 {
                            tradez_types::position::Side::Bid
                        } else {
                            tradez_types::position::Side::Ask
                        },
                        // TODO: Fix
                        nonce: 0,
                        size,
                        price,
                    };
                    let signature = wallet.sign_message(&api_order.rlp_bytes()).unwrap();
                    let result = TradezRpcClient::send_order(&client, api_order, signature)
                        .await
                        .unwrap();
                    println!("Result from server: {}", result);
                }
                WalletCommand::ClosePosition { position_id } => {
                    println!(
                        "Closing position with ID: {} for wallet: {}",
                        position_id, wallet_cmd.name
                    );
                    let cancel_order = CancelOrder {
                        order_id: position_id,
                    };
                    let signature = wallet.sign_message(&cancel_order.rlp_bytes()).unwrap();
                    let result = TradezRpcClient::cancel_order(&client, cancel_order, signature)
                        .await
                        .unwrap();
                    println!("Result from server: {}", result);
                    // Implement close position logic here
                }
                WalletCommand::Faucet { amount, currency } => {
                    println!(
                        "Requesting faucet of amount: {} for wallet: {}",
                        amount, wallet_cmd.name
                    );
                    let faucet = Faucet {
                        amount,
                        currency: match currency {
                            0 => tradez_types::currencies::Currencies::USDC,
                            1 => tradez_types::currencies::Currencies::XTZ,
                            _ => {
                                println!("Invalid currency specified. Defaulting to USDC.");
                                tradez_types::currencies::Currencies::USDC
                            }
                        },
                    };
                    let signature = wallet.sign_message(&faucet.rlp_bytes()).unwrap();
                    let result = TradezRpcClient::faucet(&client, faucet, signature)
                        .await
                        .unwrap();
                    println!("Result from server: {}", result);
                }
            }
        }
        AppSubcommand::Get(get_cmd) => match get_cmd.command {
            GetInfosCommand::OrderbookState {} => {
                println!("Fetching orderbook state...");
                let (bids, asks) = TradezRpcClient::get_orderbook_state(&client).await.unwrap();
                println!("Bids: {:?}", bids);
                println!("Asks: {:?}", asks);
            }
            GetInfosCommand::Balances { address } => {
                println!("Fetching balances for address: {}", address);
                let balances = TradezRpcClient::get_balances(&client, address)
                    .await
                    .unwrap();
                println!("Balances: {:?}", balances);
            }
            GetInfosCommand::Orders { address } => {
                println!("Fetching orders for address: {}", address);
                let orders = TradezRpcClient::get_orders(&client, address).await.unwrap();
                println!("Orders: {:?}", orders);
            }
            GetInfosCommand::History {} => {
                println!("Fetching trade history...");
                let history = TradezRpcClient::get_history(&client).await.unwrap();
                println!("History: {:?}", history);
            }
        },
    }
}
