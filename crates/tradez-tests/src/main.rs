use std::time::Duration;

use rand::{Rng, seq::SliceRandom};

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
                    tradez_client| {
            tradez_client.faucet_usdc(1000000000);
            tradez_client.faucet_xtz(1000000000);
            let mut rand = rand::thread_rng();
            let mut price_values = Vec::new();
            for _ in 0..14 {
                let price: u64 = rand.gen_range(900000..=1100000);
                price_values.push(price);
            }
            // Is multiplied by 2
            let number_of_orders = 500;
            loop {
                octez_client.bake_l1_blocks(1);
                for _ in 0..number_of_orders {
                    // Place buy orders of a size between 1000000 and 10000000 at a price between 900000 and 1100000 ( 10 different possibles prices )
                    let size: u64 = rand.gen_range(1000000..=10000000);
                    let price: u64 = *price_values.choose(&mut rand).unwrap();
                    tradez_client.buy(size, price);
                    // Place sell orders of a size between 1000000 and 10000000 at a price between 900000 and 1100000
                    let size: u64 = rand.gen_range(1000000..=10000000);
                    let price: u64 = *price_values.choose(&mut rand).unwrap();
                    tradez_client.sell(size, price);
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        },
    )
    .await;
}
