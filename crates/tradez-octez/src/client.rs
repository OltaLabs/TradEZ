use std::path;
use std::process::Command;

use crate::logging::{print_prefixed_lines, run_command};

pub struct Client {
    data_dir: String,
    #[allow(dead_code)]
    temp_dir: Option<tempfile::TempDir>,
    rpc_addr: String,
    config: ClientConfig,
}

pub struct ClientConfig {
    pub print_commands: bool,
    pub verbose: bool,
}

impl Client {
    pub fn new(data_dir: String, config: ClientConfig, rpc_addr: Option<String>) -> Self {
        let rpc_addr = rpc_addr.unwrap_or_else(|| "http://localhost:8732".into());
        Client {
            data_dir,
            config,
            temp_dir: None,
            rpc_addr,
        }
    }

    pub fn new_with_temp_dir(config: ClientConfig, rpc_addr: Option<String>) -> Self {
        let temp_dir = tempfile::TempDir::with_suffix("tradez_octez_client")
            .expect("Failed to create temp dir for client data");
        let rpc_addr = rpc_addr.unwrap_or_else(|| "http://localhost:8732".into());
        Client {
            data_dir: temp_dir.path().to_str().unwrap().to_string(),
            temp_dir: Some(temp_dir),
            config,
            rpc_addr,
        }
    }

    pub fn data_dir(&self) -> &str {
        &self.data_dir
    }

    fn run_octez_command(&self, command: &mut Command, spawn_err: &str, wait_err: &str) {
        run_command(
            command,
            "octez-client",
            self.config.verbose,
            self.config.print_commands,
            spawn_err,
            wait_err,
        );
    }

    pub fn import_account(&self, alias: &str, sk: &str) {
        let mut command = Command::new("octez-client");
        command
            .env("TEZOS_CLIENT_UNSAFE_DISABLE_DISCLAIMER", "Y")
            .arg("--base-dir")
            .arg(&self.data_dir)
            .arg("--endpoint")
            .arg(&self.rpc_addr)
            .arg("import")
            .arg("secret")
            .arg("key")
            .arg(alias)
            .arg(sk);
        self.run_octez_command(
            &mut command,
            "Failed to spawn octez-client import secret key command",
            "Failed to wait for octez-client import secret key command",
        );
    }

    pub fn import_accounts_from_file(&self, path: &path::Path) {
        let accounts = builtin_bootstrap_accounts(path);
        for (alias, _pk, sk, _balance) in accounts {
            self.import_account(&alias, &sk);
        }
    }

    pub fn activate_protocol(&self, file: &path::Path, protocol_hash: &str) {
        let mut command = Command::new("octez-client");
        command
            .env("TEZOS_CLIENT_UNSAFE_DISABLE_DISCLAIMER", "Y")
            .arg("--base-dir")
            .arg(&self.data_dir)
            .arg("--endpoint")
            .arg(&self.rpc_addr)
            .arg("-block")
            .arg("genesis")
            .arg("activate")
            .arg("protocol")
            .arg(protocol_hash)
            .arg("with")
            .arg("fitness")
            .arg("1")
            .arg("and")
            .arg("key")
            .arg("activator")
            .arg("and")
            .arg("parameters")
            .arg(file);
        self.run_octez_command(
            &mut command,
            "Failed to spawn octez-client activate protocol command",
            "Failed to wait for octez-client activate protocol command",
        );
    }

    pub fn get_balance(&self, alias: &str) -> u64 {
        let mut command = Command::new("octez-client");
        command
            .env("TEZOS_CLIENT_UNSAFE_DISABLE_DISCLAIMER", "Y")
            .arg("--base-dir")
            .arg(&self.data_dir)
            .arg("--endpoint")
            .arg(&self.rpc_addr)
            .arg("get")
            .arg("balance")
            .arg("for")
            .arg(alias);
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        if self.config.print_commands {
            println!("> {:?}", command);
        }
        let output = command
            .output()
            .expect("Failed to spawn octez-client get balance command");
        if self.config.verbose {
            print_prefixed_lines(&output.stdout, "octez-client", false);
            print_prefixed_lines(&output.stderr, "octez-client", true);
        }
        let balance_str = String::from_utf8_lossy(&output.stdout);
        balance_str
            .split_whitespace()
            .next()
            .unwrap_or("0")
            .parse::<f64>()
            .expect("Failed to parse balance") as u64
    }

    pub fn originate_smart_roll_up(
        &self,
        rollup_name: &str,
        from_alias: &str,
        kernel_file: &path::Path,
    ) {
        let mut command = Command::new("octez-client");
        command
            .env("TEZOS_CLIENT_UNSAFE_DISABLE_DISCLAIMER", "Y")
            .arg("--base-dir")
            .arg(&self.data_dir)
            .arg("--endpoint")
            .arg(&self.rpc_addr)
            .arg("--wait")
            .arg("none")
            .arg("originate")
            .arg("smart")
            .arg("rollup")
            .arg(rollup_name)
            .arg("from")
            .arg(from_alias)
            .arg("of")
            .arg("kind")
            .arg("wasm_2_0_0")
            .arg("of")
            .arg("type")
            .arg("bytes")
            .arg("with")
            .arg("kernel")
            .arg(format!("file:{}", kernel_file.to_str().unwrap()))
            .arg("--burn-cap")
            .arg("3");
        self.run_octez_command(
            &mut command,
            "Failed to spawn octez-client originate smart rollup command",
            "Failed to wait for octez-client originate smart rollup command",
        );
        self.bake_l1_blocks(1);
    }

    pub fn bake_l1_blocks(&self, count: u32) {
        for _ in 0..count {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let mut command = Command::new("octez-client");
            command
                .env("TEZOS_CLIENT_UNSAFE_DISABLE_DISCLAIMER", "Y")
                .arg("--base-dir")
                .arg(&self.data_dir)
                .arg("--endpoint")
                .arg(&self.rpc_addr)
                .arg("bake")
                .arg("for")
                .arg("--minimal-timestamp");
            self.run_octez_command(
                &mut command,
                "Failed to spawn octez-client bake command",
                "Failed to wait for octez-client bake command",
            );
        }
    }
}

pub fn builtin_bootstrap_accounts(path: &path::Path) -> Vec<(String, String, String, u64)> {
    serde_json::from_slice(&std::fs::read(path).expect("error reading built-in bootstrap accounts"))
        .expect("error parsing built-in bootstrap accounts")
}
