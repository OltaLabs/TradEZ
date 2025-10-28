use std::time::Duration;

use crate::setup::{TestConfig, tradez_test_wrapper};

mod client;
mod sequencer;
mod setup;

#[tokio::main]
async fn main() {
    let config = TestConfig {
        verbose: true,
        print_commands: false,
    };
    tradez_test_wrapper(
        config,
        async move |_l1_node,
                    octez_client,
                    _smart_rollup_node,
                    _smart_rollup_client,
                    _tradez_sequencer,
                    _tradez_client| {
            loop {
                octez_client.bake_l1_blocks(1);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        },
    )
    .await;
}
