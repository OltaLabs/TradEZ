#[cfg(test)]
mod client;
#[cfg(test)]
mod sequencer;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use rlp::{Decodable, Rlp};
    use tradez_octez::l1_node::L1NodeConfig;
    use tradez_types::orderbook::{OrderBook, ORDER_BOOK_STR_PATH};

    // Here you can write integration tests that use tradez (sequencer + client) and an L1 node + smart rollup node + tezos client
    #[tokio::test]
    async fn basic_flow_works() {
        let mut node = tradez_octez::l1_node::L1Node::launch(L1NodeConfig {
            print_commands: true,
            verbose: true,
        });
        std::thread::sleep(std::time::Duration::from_secs(5));
        let octez_client = tradez_octez::client::Client::new_with_temp_dir(
            tradez_octez::client::ClientConfig {
                print_commands: true,
                verbose: true,
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
        let mut sequencer = crate::sequencer::Sequencer::new(crate::sequencer::SequencerConfig {
            print_commands: true,
            verbose: true,
            smart_rollup_node_address: smart_rollup_node.rpc_addr(),
        });
        let smart_rollup_client = tradez_octez::smart_rollup_node::SmartRollupClient::new(
            &smart_rollup_node.rpc_addr()
        );
        std::thread::sleep(std::time::Duration::from_secs(1));
        let tradez_client = crate::client::Client::new(
            crate::client::ClientConfig {
                print_commands: true,
                verbose: true,
            },
            sequencer.rpc_port,
        );
        tradez_client.buy(10, 1000);
        std::thread::sleep(std::time::Duration::from_secs(3));
        octez_client.bake_l1_blocks(2);
        std::thread::sleep(std::time::Duration::from_secs(3));
        let bytes = smart_rollup_client.get_value(ORDER_BOOK_STR_PATH).await.unwrap().unwrap();
        println!("Raw order book bytes: {:?}", bytes);
        let order_book = OrderBook::decode(&Rlp::new(&bytes)).unwrap();
        println!("Order book after one buy: {:?}", order_book);
        sequencer.stop();
        smart_rollup_node.stop();
        node.stop();
    }
}
