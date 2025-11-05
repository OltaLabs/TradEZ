use openport::pick_unused_port;
use std::{
    process::{Command, Stdio},
    thread,
};
use tempfile::TempDir;

pub struct L1Node {
    tx: std::sync::mpsc::Sender<()>,
    pub rpc_port: u16,
    thread: Option<thread::JoinHandle<()>>,
}

pub struct L1NodeConfig {
    pub print_commands: bool,
    pub verbose: bool,
}

impl L1Node {
    pub fn launch(config: L1NodeConfig) -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        let net_port =
            pick_unused_port(15000..16000).expect("Failed to pick unused port for L1 node net");
        let metrics_port = pick_unused_port((net_port + 1)..17000)
            .expect("Failed to pick unused port for L1 node metrics");
        let rpc_port = pick_unused_port((metrics_port + 1)..18000)
            .expect("Failed to pick unused port for L1 node rpc");
        let node_data_dir = TempDir::with_suffix("tradez_l1_node")
            .expect("Failed to create temp dir for L1 node data");
        let thread = thread::spawn(move || {
            let mut command = Command::new("octez-node");
            command
                .arg("identity")
                .arg("generate")
                .arg("--data-dir")
                .arg(node_data_dir.path())
                .arg("0");
            if config.verbose {
                command.stdout(Stdio::inherit());
                command.stderr(Stdio::inherit());
            } else {
                command.stdout(Stdio::piped());
                command.stderr(Stdio::piped());
            }
            if config.print_commands {
                println!("> {:?}", command);
            }
            command
                .spawn()
                .expect("Failed to generate L1 node identity")
                .wait()
                .expect("Failed to wait for octez-node identity generate command");
            // Initialize the node configuration
            let mut command = Command::new("octez-node");
            command
                .arg("config")
                .arg("init")
                .arg("--data-dir")
                .arg(node_data_dir.path())
                .arg("--no-bootstrap-peers")
                .arg("--history-mode")
                .arg("archive")
                .arg("--synchronisation-threshold")
                .arg("0")
                .arg("--network")
                .arg("sandbox")
                .arg("--expected-pow")
                .arg("0");

            if config.verbose {
                command.stdout(Stdio::inherit());
                command.stderr(Stdio::inherit());
            } else {
                command.stdout(Stdio::piped());
                command.stderr(Stdio::piped());
            }

            if config.print_commands {
                println!("> {:?}", command);
            }
            command
                .spawn()
                .expect("Failed to initialize L1 node configuration")
                .wait()
                .expect("Failed to wait for octez-node config init command");
            let mut command = Command::new("octez-node");
            command
                .arg("run")
                .arg("--data-dir")
                .arg(node_data_dir.path())
                .arg("--net-addr")
                .arg(format!("127.0.0.1:{}", net_port))
                .arg("--metrics-addr")
                .arg(format!("127.0.0.1:{}", metrics_port))
                .arg("--rpc-addr")
                .arg(format!("127.0.0.1:{}", rpc_port))
                .arg("--max-active-rpc-connections")
                .arg("500")
                .arg("--allow-all-rpc")
                .arg(format!("127.0.0.1:{}", rpc_port));
            if config.verbose {
                command.stdout(Stdio::inherit());
                command.stderr(Stdio::inherit());
            } else {
                command.stdout(Stdio::piped());
                command.stderr(Stdio::piped());
            }
            if config.print_commands {
                println!("> {:?}", command);
            }
            let mut child = command.spawn().expect("Failed to start L1 node");
            match rx.recv() {
                Ok(_) | Err(std::sync::mpsc::RecvError) => {
                    // Stop the node
                    child.kill().expect("Failed to kill L1 node process");
                }
            }
        });

        L1Node {
            tx,
            rpc_port,
            thread: Some(thread),
        }
    }
    pub fn stop(&mut self) {
        self.tx
            .send(())
            .expect("Failed to send stop signal to L1 node");
        if let Some(thread) = self.thread.take() {
            thread.join().expect("Failed to stop L1 node");
        }
    }
}

impl Drop for L1Node {
    fn drop(&mut self) {
        self.stop();
    }
}
