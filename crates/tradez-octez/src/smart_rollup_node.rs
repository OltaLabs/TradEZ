use std::path::{Path, PathBuf};
use std::thread::JoinHandle;

use serde::Deserialize;
use tempfile::TempDir;

use crate::{error::OctezError, logging::spawn_command, ports::pick_unused_port};

pub struct SmartRollupNode {
    base_dir_path: PathBuf,
    config: SmartRollupNodeConfig,
    child: Option<std::process::Child>,
    data_dir: TempDir,
    l1_rpc_addr: String,
    rpc_port: u16,
    stdout_handle: Option<JoinHandle<()>>,
    stderr_handle: Option<JoinHandle<()>>,
}

pub struct SmartRollupNodeConfig {
    pub print_commands: bool,
    pub verbose: bool,
}

impl SmartRollupNode {
    pub fn new(base_dir_path: &Path, config: SmartRollupNodeConfig, l1_rpc_addr: String) -> Self {
        let data_dir = TempDir::with_suffix("tradez_smart_rollup_node")
            .expect("Failed to create temp dir for smart rollup node data");
        let rpc_port = pick_unused_port();
        SmartRollupNode {
            base_dir_path: base_dir_path.to_path_buf(),
            config,
            rpc_port,
            child: None,
            data_dir,
            l1_rpc_addr,
            stdout_handle: None,
            stderr_handle: None,
        }
    }

    pub fn pre_images_path(&self) -> PathBuf {
        // if they don't exists create it
        let path = self.data_dir.path().join("wasm_2_0_0");
        std::fs::create_dir_all(&path).expect("Failed to create pre-images directory");
        path
    }

    pub fn data_path(&self) -> &Path {
        // if they don't exists create it
        let path = self.data_dir.path();
        std::fs::create_dir_all(path).expect("Failed to create data directory");
        path
    }

    pub fn start(&mut self, operator: &str) {
        let mut command = std::process::Command::new("octez-smart-rollup-node");
        let metrics_port = pick_unused_port();
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
            .arg(self.rpc_port.to_string())
            .arg("--rpc-addr")
            .arg("127.0.0.1")
            .arg("--metrics-addr")
            .arg(format!("127.0.0.1:{}", metrics_port))
            .arg("--log-kernel-debug")
            .arg("--log-kernel-debug-file")
            .arg("tradez_kernel.debug");
        let (child, stdout_handle, stderr_handle) = spawn_command(
            &mut command,
            "smart-rollup-node",
            self.config.verbose,
            self.config.print_commands,
            "Failed to spawn octez-smart-rollup-node run command",
        );
        self.child = Some(child);
        self.stdout_handle = stdout_handle;
        self.stderr_handle = stderr_handle;
    }

    pub fn stop(&mut self) {
        if let Some(child) = &mut self.child {
            let _ = child.kill();
            let _ = child.wait();
            self.child = None;
        }
        if let Some(handle) = self.stdout_handle.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.stderr_handle.take() {
            let _ = handle.join();
        }
    }

    pub fn rpc_addr(&self) -> String {
        format!("http://127.0.0.1:{}", self.rpc_port)
    }
}

impl Drop for SmartRollupNode {
    fn drop(&mut self) {
        self.stop();
    }
}

pub struct SmartRollupClient {
    client: reqwest::Client,
    api_addr: String,
}

#[derive(Debug, Deserialize)]
pub struct ValueError {
    pub kind: String,
    pub id: String,
    pub block: Option<String>,
    pub msg: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ValueResponse {
    Value(String),
    Errors(Vec<ValueError>),
}

impl SmartRollupClient {
    pub fn new(api_addr: &str) -> Self {
        SmartRollupClient {
            api_addr: api_addr.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn inject_inbox_messages(
        &self,
        inbox_message: Vec<Vec<u8>>,
    ) -> Result<(), OctezError> {
        let res = self
            .client
            .post(format!("{}/local/batcher/injection", self.api_addr))
            .json(
                &inbox_message
                    .into_iter()
                    .map(hex::encode)
                    .collect::<Vec<String>>(),
            )
            .send()
            .await
            .unwrap();

        if res.status() == 200 {
            Ok(())
        } else {
            let err_text = res.text().await.unwrap();
            Err(crate::error::OctezError::HttpResponseError(err_text))
        }
    }

    pub async fn get_value(&self, key: &str) -> Result<Option<Vec<u8>>, OctezError> {
        let res = self
            .client
            .get(format!(
                "{}/global/block/head/durable/wasm_2_0_0/value?key={}",
                self.api_addr, key
            ))
            .send()
            .await?;

        if res.status() == 200 || res.status() == 500 {
            let content: Option<ValueResponse> = res.json().await?;
            match content {
                Some(ValueResponse::Value(value)) => {
                    let payload = hex::decode(value)?;
                    Ok(Some(payload))
                }
                Some(ValueResponse::Errors(errors)) => Err(OctezError::HttpResponseError(format!(
                    "Failed to get value of key-value pair: {}. Errors: {:?}",
                    key, errors
                ))),
                None => Ok(None),
            }
        } else {
            Err(OctezError::HttpResponseError(format!(
                "Unhandled response status: {}",
                res.status()
            )))
        }
    }
}
