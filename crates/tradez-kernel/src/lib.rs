extern crate alloc;

use alloy_primitives::{Signature, B512};
use tezos_smart_rollup::inbox::InboxMessage;
use tezos_smart_rollup::michelson::MichelsonBytes;
use tezos_smart_rollup::prelude::*;
use tradez_types::{address::Address, orderbook::OrderBook, position::APIOrder, SignedInput};
use rlp::Encodable;

fn handle_message(host: &mut impl Runtime, msg: impl AsRef<[u8]>) {
    if let Some((_, msg)) = InboxMessage::<MichelsonBytes>::parse(msg.as_ref()).ok() {
        match msg {
            InboxMessage::External(data) => {
                let SignedInput { message: order, signature }: SignedInput<APIOrder> = rlp::decode(data).unwrap();
                host.write_debug(&format!(
                    "Received Order: side={}, size={}, price={}\n",
                    order.side, order.size, order.price
                ));
                let signature = Signature::from_raw(&signature).unwrap();
                let caller = signature.recover_from_msg(order.rlp_bytes()).unwrap();
                host.write_debug(&format!("Order placed by address: {:?}\n", B512::from_slice(&caller.to_encoded_point(false).as_bytes()[1..])));
                let mut orderbook = OrderBook::load(host).unwrap();
                let mut events = vec![];
                orderbook.place_limit(
                    Address::ZERO,
                    order.side,
                    order.price,
                    order.size,
                    order.ts,
                    &mut events,
                );
                orderbook.save(host).unwrap();
                for event in events {
                    host.write_debug(&format!("Order book event: {:?}\n", event));
                }
            }
            InboxMessage::Internal(_) => {}
        }
    }
}

// TODO: Have a persistent order book for the sequencer native execution
#[entrypoint::main]
pub fn kernel_loop<Host: tezos_smart_rollup_host::runtime::Runtime>(host: &mut Host) {
    host.write_debug("Kernel started.\n");
    while let Some(msg) = host.read_input().unwrap() {
        handle_message(host, msg);
    }
}
