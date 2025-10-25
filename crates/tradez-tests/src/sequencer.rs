use std::process::Command;

use tempfile::TempDir;

pub struct SequencerConfig {
    pub print_commands: bool,
    pub verbose: bool,
    pub smart_rollup_node_address: String,
}

pub struct Sequencer {
    child: std::process::Child,
    pub rpc_port: u16,
    #[allow(dead_code)]
    pub data_dir: TempDir,
}

impl Sequencer {
    pub fn new(config: SequencerConfig) -> Self {
        let rpc_port = openport::pick_unused_port(19000..20000).expect("Failed to find free port");
        let data_dir = TempDir::with_suffix("tradez_sequencer")
            .expect("Failed to create temp dir for sequencer data");
        let mut command = Command::new("../../target/release/tradez-sequencer");
        command
            .arg("--rpc-port")
            .arg(rpc_port.to_string())
            .arg("--smart-rollup-addr")
            .arg(config.smart_rollup_node_address)
            .arg("--data-dir")
            .arg(data_dir.path());
        if config.verbose {
            command.stdout(std::process::Stdio::inherit());
            command.stderr(std::process::Stdio::inherit());
        } else {
            command.stdout(std::process::Stdio::piped());
            command.stderr(std::process::Stdio::piped());
        }
        if config.print_commands {
            println!("> {:?}", command);
        }
        let child = command.spawn().expect("Failed to start sequencer");
        Sequencer {
            child,
            rpc_port,
            data_dir,
        }
    }

    pub fn stop(&mut self) {
        self.child.kill().expect("Failed to kill sequencer process");
        self.child
            .wait()
            .expect("Failed to wait for sequencer process to exit");
    }
}

impl Drop for Sequencer {
    fn drop(&mut self) {
        self.stop();
    }
}
