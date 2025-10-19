use std::path::{Path, PathBuf};

use tempfile::TempDir;

pub struct SmartRollupNode {
    base_dir_path: PathBuf,
    config: SmartRollupNodeConfig,
    child: Option<std::process::Child>,
    data_dir: TempDir,
    l1_rpc_addr: String,
}

pub struct SmartRollupNodeConfig {
    pub print_commands: bool,
    pub verbose: bool,
}

impl SmartRollupNode {
    pub fn new(base_dir_path: &Path, config: SmartRollupNodeConfig, l1_rpc_addr: String) -> Self {
        let data_dir = TempDir::with_suffix("tradez_smart_rollup_node")
            .expect("Failed to create temp dir for smart rollup node data");
        SmartRollupNode {
            base_dir_path: base_dir_path.to_path_buf(),
            config,
            child: None,
            data_dir,
            l1_rpc_addr,
        }
    }

    pub fn pre_images_path(&self) -> PathBuf {
        self.data_dir.path().join("wasm_2_0_0")
    }

    pub fn data_path(&self) -> &Path {
        self.data_dir.path()
    }

    pub fn start(&mut self, operator: &str) {
        let mut command = std::process::Command::new("octez-smart-rollup-node");
        let rpc_port = openport::pick_unused_port(15500..16000)
            .expect("Failed to pick unused port for smart rollup node rpc");
        let metrics_port = openport::pick_unused_port((rpc_port + 1)..17000)
            .expect("Failed to pick unused port for smart rollup node metrics");
        command
            .arg("--endpoint")
            .arg(&self.l1_rpc_addr)
            .arg("--base-dir")
            .arg(self.base_dir_path.as_path())
            .arg("run")
            .arg("operator")
            .arg("for")
            .arg("tradez_rollup")
            .arg("with")
            .arg("operators")
            .arg(operator)
            .arg("--data-dir")
            .arg(self.data_dir.path())
            .arg("--rpc-port")
            .arg(rpc_port.to_string())
            .arg("--rpc-addr")
            .arg("127.0.0.1")
            .arg("--metrics-addr")
            .arg(format!("127.0.0.1:{}", metrics_port))
            .arg("--log-kernel-debug")
            .arg("--log-kernel-debug-file")
            .arg(self.data_path().join("tradez_kernel.debug"));
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
        let child = command
            .spawn()
            .expect("Failed to spawn octez-smart-rollup-node run command");
        self.child = Some(child);
    }

    pub fn stop(&mut self) {
        if let Some(child) = &mut self.child {
            child
                .kill()
                .expect("Failed to kill smart rollup node process");
            child
                .wait()
                .expect("Failed to wait for smart rollup node process to exit");
            self.child = None;
        }
    }
}
