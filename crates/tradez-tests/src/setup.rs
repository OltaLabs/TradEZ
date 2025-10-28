use std::path::Path;

use tradez_octez::l1_node::{L1Node, L1NodeConfig};

pub struct TestConfig {
    pub verbose: bool,
    pub print_commands: bool,
}

pub async fn tradez_test_wrapper<F, R>(config: TestConfig, test_fn: F)
where
    R: std::future::Future<Output = ()>,
    F: FnOnce(
        L1Node,
        tradez_octez::client::Client,
        tradez_octez::smart_rollup_node::SmartRollupNode,
        tradez_octez::smart_rollup_node::SmartRollupClient,
        crate::sequencer::Sequencer,
        crate::client::Client,
    ) -> R,
{
    // Setup
    let node = L1Node::launch(L1NodeConfig {
        print_commands: config.print_commands,
        verbose: config.verbose,
    });
    std::thread::sleep(std::time::Duration::from_secs(5));
    let octez_client = tradez_octez::client::Client::new_with_temp_dir(
        tradez_octez::client::ClientConfig {
            print_commands: config.print_commands,
            verbose: config.verbose,
        },
        Some(format!("http://localhost:{}", node.rpc_port)),
    );

    octez_client.import_accounts_from_file(std::path::Path::new("accounts.json"));
    octez_client.activate_protocol(
        std::path::Path::new("sandbox_parameters.json"),
        "PtSeouLouXkxhg39oWzjxDWaCydNfR3RxCUrNe4Q9Ro8BTehcbh",
    );
    let mut smart_rollup_node = tradez_octez::smart_rollup_node::SmartRollupNode::new(
        Path::new(octez_client.data_dir()),
        tradez_octez::smart_rollup_node::SmartRollupNodeConfig {
            print_commands: true,
            verbose: true,
        },
        format!("http://localhost:{}", node.rpc_port),
    );
    tradez_octez::smart_rollup_installer::create_installer(
        Path::new("tradez_kernel.wasm"),
        smart_rollup_node.pre_images_path().as_path(),
        smart_rollup_node
            .data_path()
            .join("tradez_kernel_installer.hex")
            .as_path(),
        tradez_octez::smart_rollup_installer::SmartRollupInstallerConfig {
            print_commands: true,
            verbose: true,
        },
    );
    octez_client.originate_smart_roll_up(
        "tradez_rollup",
        "bootstrap1",
        smart_rollup_node
            .data_path()
            .join("tradez_kernel_installer.hex")
            .as_path(),
    );
    octez_client.bake_l1_blocks(1);
    smart_rollup_node.start("bootstrap1");
    octez_client.bake_l1_blocks(1);
    std::thread::sleep(std::time::Duration::from_secs(2));
    let sequencer = crate::sequencer::Sequencer::new(crate::sequencer::SequencerConfig {
        print_commands: true,
        verbose: true,
        smart_rollup_node_address: smart_rollup_node.rpc_addr(),
    });
    let smart_rollup_client =
        tradez_octez::smart_rollup_node::SmartRollupClient::new(&smart_rollup_node.rpc_addr());
    std::thread::sleep(std::time::Duration::from_secs(1));
    let tradez_client = crate::client::Client::new(
        crate::client::ClientConfig {
            print_commands: true,
            verbose: true,
        },
        sequencer.rpc_port,
    );
    tradez_client.faucet_xtz(1_000_000_000);
    tradez_client.faucet_usdc(1_000_000_000);
    std::thread::sleep(std::time::Duration::from_secs(2));
    octez_client.bake_l1_blocks(1);
    std::thread::sleep(std::time::Duration::from_secs(2));
    test_fn(
        node,
        octez_client,
        smart_rollup_node,
        smart_rollup_client,
        sequencer,
        tradez_client,
    )
    .await;
}
