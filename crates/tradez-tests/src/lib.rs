#[cfg(test)]
mod client;
#[cfg(test)]
mod sequencer;

#[cfg(test)]
mod setup;

#[cfg(test)]
mod tests {
    use rlp::{Decodable, Rlp};
    use tradez_types::orderbook::{ORDER_BOOK_STR_PATH, OrderBook};

    use crate::setup::{TestConfig, tradez_test_wrapper};

    // Here you can write integration tests that use tradez (sequencer + client) and an L1 node + smart rollup node + tezos client
    #[tokio::test]
    async fn basic_flow_works() {
        let config = TestConfig {
            verbose: true,
            print_commands: true,
        };
        tradez_test_wrapper(
            config,
            async move |_l1_node,
                        octez_client,
                        _smart_rollup_node,
                        smart_rollup_client,
                        _tradez_sequencer,
                        tradez_client| {
                tradez_client.buy(10, 1000);
                std::thread::sleep(std::time::Duration::from_secs(2));
                octez_client.bake_l1_blocks(2);
                std::thread::sleep(std::time::Duration::from_secs(2));
                let bytes = smart_rollup_client
                    .get_value(ORDER_BOOK_STR_PATH)
                    .await
                    .unwrap()
                    .unwrap();
                let order_book = OrderBook::decode(&Rlp::new(&bytes)).unwrap();
                assert_eq!(order_book.best_bid(), Some(1000));
            },
        )
        .await
    }

    #[tokio::test]
    async fn basic_buy_sell() {
        let config = TestConfig {
            verbose: true,
            print_commands: true,
        };
        tradez_test_wrapper(
            config,
            async move |_l1_node,
                        octez_client,
                        _smart_rollup_node,
                        smart_rollup_client,
                        _tradez_sequencer,
                        tradez_client| {
                tradez_client.buy(10, 1000);
                std::thread::sleep(std::time::Duration::from_secs(1));
                octez_client.bake_l1_blocks(2);
                std::thread::sleep(std::time::Duration::from_secs(2));
                tradez_client.sell(5, 900);
                std::thread::sleep(std::time::Duration::from_secs(1));
                octez_client.bake_l1_blocks(2);
                std::thread::sleep(std::time::Duration::from_secs(2));
                let bytes = smart_rollup_client
                    .get_value(ORDER_BOOK_STR_PATH)
                    .await
                    .unwrap()
                    .unwrap();
                let order_book = OrderBook::decode(&Rlp::new(&bytes)).unwrap();
                assert_eq!(order_book.price_quantity_at(1000), 5);
            },
        )
        .await
    }
}
