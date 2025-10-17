use clap::{Parser, Subcommand};

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
    Create {
    },
    /// Get positions of a wallet
    GetPositions {
    },
    /// Balance of a wallet
    Balance {
    },
    /// Open a new position
    OpenPosition {
        /// Side of the position (0 = long, 1 = short)
        #[arg(short, long)]
        side: u8,
    },
    /// Close an existing position
    ClosePosition {
        /// ID of the position to close
        #[arg(short, long)]
        position_id: u64,
    },
}

pub fn main() {
    let args = Args::parse();
    
    let server_url = args.url;
    match args.app {
        AppSubcommand::Wallet(wallet_cmd) => {
            let wallet = wallet::load_wallet(&wallet_cmd.dirpath, &wallet_cmd.name, wallet_cmd.password);
            match wallet_cmd.command {
                WalletCommand::Create {} => {
                    println!("Wallet created: {}", wallet_cmd.name);
                }
                WalletCommand::GetPositions {} => {
                    println!("Fetching positions for wallet: {}", wallet_cmd.name);
                    // Implement fetching positions logic here
                }
                WalletCommand::Balance {} => {
                    println!("Fetching balance for wallet: {}", wallet_cmd.name);
                    // Implement balance fetching logic here
                }
                WalletCommand::OpenPosition { side } => {
                    println!("Opening position with side: {} for wallet: {}", side, wallet_cmd.name);
                    // Implement open position logic here
                }
                WalletCommand::ClosePosition { position_id } => {
                    println!("Closing position with ID: {} for wallet: {}", position_id, wallet_cmd.name);
                    // Implement close position logic here
                }
            }
        }
    }
}