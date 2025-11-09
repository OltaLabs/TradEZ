use std::process::{Command, Stdio};
use std::thread::JoinHandle;

use tempfile::TempDir;
use tradez_octez::logging::spawn_prefixed_logger;

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
    pub stdout_handle: Option<JoinHandle<()>>,
    pub stderr_handle: Option<JoinHandle<()>>,
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
            command.stdout(Stdio::piped()).stderr(Stdio::piped());
        } else {
            command.stdout(Stdio::null()).stderr(Stdio::null());
        }
        if config.print_commands {
            println!("> {:?}", command);
        }
        let mut child = command.spawn().expect("Failed to start sequencer");
        let (stdout_handle, stderr_handle) = if config.verbose {
            (
                child
                    .stdout
                    .take()
                    .map(|stdout| spawn_prefixed_logger(stdout, "tradez-sequencer", false)),
                child
                    .stderr
                    .take()
                    .map(|stderr| spawn_prefixed_logger(stderr, "tradez-sequencer", true)),
            )
        } else {
            (None, None)
        };
        Sequencer {
            child,
            rpc_port,
            data_dir,
            stdout_handle,
            stderr_handle,
        }
    }

    pub fn stop(&mut self) {
        let still_running = match self.child.try_wait() {
            Ok(Some(_)) => false,
            Ok(None) => true,
            Err(_) => true,
        };
        if still_running {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        if let Some(handle) = self.stdout_handle.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.stderr_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Sequencer {
    fn drop(&mut self) {
        self.stop();
    }
}
