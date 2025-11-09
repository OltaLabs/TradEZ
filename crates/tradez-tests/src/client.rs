#![allow(dead_code)]
use std::process::{Command, Stdio};

use tempfile::TempDir;
use tradez_octez::logging::{print_prefixed_lines, run_command};

pub struct Client {
    rpc_url: String,
    wallet_dir: TempDir,
    config: ClientConfig,
}

pub struct ClientConfig {
    pub verbose: bool,
    pub print_commands: bool,
}

impl Client {
    fn configure_stdio(config: &ClientConfig, command: &mut Command) {
        if config.verbose {
            command.stdout(Stdio::piped()).stderr(Stdio::piped());
        } else {
            command.stdout(Stdio::null()).stderr(Stdio::null());
        }
    }

    pub fn new(config: ClientConfig, rpc_port: u16) -> Self {
        let wallet_dir = TempDir::with_suffix("tradez_client").unwrap();
        let rpc_url = format!("http://localhost:{}", rpc_port);
        let mut command = Command::new("../../target/release/tradez-client");
        command
            .arg("--url")
            .arg(&rpc_url)
            .arg("wallet")
            .arg("--dirpath")
            .arg(wallet_dir.path())
            .arg("create");
        run_command(
            &mut command,
            "tradez-client",
            config.verbose,
            config.print_commands,
            "Failed to spawn tradez-client command",
            "Failed to wait for tradez-client command",
        );
        Client {
            rpc_url,
            wallet_dir,
            config,
        }
    }

    pub fn buy(&self, size: u64, price: u64) {
        let mut command = Command::new("../../target/release/tradez-client");
        command
            .arg("--url")
            .arg(&self.rpc_url)
            .arg("wallet")
            .arg("--dirpath")
            .arg(self.wallet_dir.path())
            .arg("open-position")
            .arg("--side")
            .arg("0")
            .arg("--size")
            .arg(size.to_string())
            .arg("--price")
            .arg(price.to_string());
        run_command(
            &mut command,
            "tradez-client",
            self.config.verbose,
            self.config.print_commands,
            "Failed to spawn client open-position command",
            "Failed to wait for client open-position command",
        );
    }

    pub fn sell(&self, size: u64, price: u64) {
        let mut command = Command::new("../../target/release/tradez-client");
        command
            .arg("--url")
            .arg(&self.rpc_url)
            .arg("wallet")
            .arg("--dirpath")
            .arg(self.wallet_dir.path())
            .arg("open-position")
            .arg("--side")
            .arg("1")
            .arg("--size")
            .arg(size.to_string())
            .arg("--price")
            .arg(price.to_string());
        run_command(
            &mut command,
            "tradez-client",
            self.config.verbose,
            self.config.print_commands,
            "Failed to spawn client open-position command",
            "Failed to wait for client open-position command",
        );
    }

    pub fn faucet_usdc(&self, amount: u64) {
        let mut command = Command::new("../../target/release/tradez-client");
        command
            .arg("--url")
            .arg(&self.rpc_url)
            .arg("wallet")
            .arg("--dirpath")
            .arg(self.wallet_dir.path())
            .arg("faucet")
            .arg("--amount")
            .arg(amount.to_string())
            .arg("--currency")
            .arg("0");
        run_command(
            &mut command,
            "tradez-client",
            self.config.verbose,
            self.config.print_commands,
            "Failed to spawn client faucet command",
            "Failed to wait for client faucet command",
        );
    }

    pub fn faucet_xtz(&self, amount: u64) {
        let mut command = Command::new("../../target/release/tradez-client");
        command
            .arg("--url")
            .arg(&self.rpc_url)
            .arg("wallet")
            .arg("--dirpath")
            .arg(self.wallet_dir.path())
            .arg("faucet")
            .arg("--amount")
            .arg(amount.to_string())
            .arg("--currency")
            .arg("1");
        run_command(
            &mut command,
            "tradez-client",
            self.config.verbose,
            self.config.print_commands,
            "Failed to spawn client faucet command",
            "Failed to wait for client faucet command",
        );
    }

    pub fn get_balances(&self, address: String) -> String {
        let mut command = Command::new("../../target/release/tradez-client");
        command
            .arg("--url")
            .arg(&self.rpc_url)
            .arg("get")
            .arg("balances")
            .arg("--address")
            .arg(address);
        if self.config.print_commands {
            println!("> {:?}", command);
        }
        let output = command
            .output()
            .expect("Failed to execute client get balances command");
        if self.config.verbose {
            print_prefixed_lines(&output.stdout, "tradez-client", false);
            print_prefixed_lines(&output.stderr, "tradez-client", true);
        }
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    pub fn get_orders(&self, address: String) -> String {
        let mut command = Command::new("../../target/release/tradez-client");
        command
            .arg("--url")
            .arg(&self.rpc_url)
            .arg("get")
            .arg("orders")
            .arg("--address")
            .arg(address);
        if self.config.print_commands {
            println!("> {:?}", command);
        }
        let output = command
            .output()
            .expect("Failed to execute client get orders command");
        if self.config.verbose {
            print_prefixed_lines(&output.stdout, "tradez-client", false);
            print_prefixed_lines(&output.stderr, "tradez-client", true);
        }
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    pub fn get_orderbook_state(&self) -> String {
        let mut command = Command::new("../../target/release/tradez-client");
        command
            .arg("--url")
            .arg(&self.rpc_url)
            .arg("get")
            .arg("orderbook-state");
        if self.config.print_commands {
            println!("> {:?}", command);
        }
        let output = command
            .output()
            .expect("Failed to execute client get orderbook state command");
        if self.config.verbose {
            print_prefixed_lines(&output.stdout, "tradez-client", false);
            print_prefixed_lines(&output.stderr, "tradez-client", true);
        }
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    pub fn get_history(&self) -> String {
        let mut command = Command::new("../../target/release/tradez-client");
        command
            .arg("--url")
            .arg(&self.rpc_url)
            .arg("get")
            .arg("history");
        if self.config.print_commands {
            println!("> {:?}", command);
        }
        let output = command
            .output()
            .expect("Failed to execute client get history command");
        if self.config.verbose {
            print_prefixed_lines(&output.stdout, "tradez-client", false);
            print_prefixed_lines(&output.stderr, "tradez-client", true);
        }
        String::from_utf8_lossy(&output.stdout).to_string()
    }
}
