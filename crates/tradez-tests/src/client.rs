use std::process::Command;

use tempfile::TempDir;

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
    pub fn new(config: ClientConfig, rpc_port: u16) -> Self {
        let wallet_dir = TempDir::with_suffix("tradez_client").unwrap();
        let rpc_url = format!("http://localhost:{}", rpc_port);
        Command::new("../../target/release/tradez-client")
            .arg("--url")
            .arg(&rpc_url)
            .arg("wallet")
            .arg("--dirpath")
            .arg(wallet_dir.path())
            .arg("create")
            .spawn()
            .expect("Failed to initialize client")
            .wait()
            .expect("Failed to wait for client init command");
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
        if self.config.verbose {
            command.stdout(std::process::Stdio::inherit());
            command.stderr(std::process::Stdio::inherit());
        } else {
            command.stdout(std::process::Stdio::piped());
            command.stderr(std::process::Stdio::piped());
        }
        if self.config.print_commands {
            println!("> {:?}", command);
        }
        command
            .spawn()
            .expect("Failed to spawn client open-position command")
            .wait()
            .expect("Failed to wait for client open-position command");
    }
}
