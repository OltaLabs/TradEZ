extern crate alloc;

use tezos_smart_rollup::inbox::InboxMessage;
use tezos_smart_rollup::michelson::MichelsonBytes;
use tezos_smart_rollup::prelude::*;
use tradez_types::position::APIOrder;

mod inbox;
mod orderbook;

fn handle_message(host: &mut impl Runtime, msg: impl AsRef<[u8]>) {
    if let Some((_, msg)) = InboxMessage::<MichelsonBytes>::parse(msg.as_ref()).ok() {
        match msg {
            InboxMessage::External(data) => {
                // let (message, _): (inbox::Message, usize) =
                //     bincode::serde::decode_from_slice(data, bincode::config::standard()).unwrap();
                // match message.action {
                //     inbox::OrderAction::Open(order) => {
                //         debug_msg!(host, "Received Open Order: {:?}\n", order);
                //     }
                //     inbox::OrderAction::Close(order_id) => {
                //         debug_msg!(host, "Received Close Order for ID: {}\n", order_id);
                //     }
                // }
                let order: APIOrder = rlp::decode(data).unwrap();
                host.write_debug(&format!(
                    "Received Order: side={}, size={}, price={}\n",
                    order.side, order.size, order.price
                ));
            }
            InboxMessage::Internal(_) => {}
        }
    }
}

#[entrypoint::main]
pub fn kernel_loop<Host: tezos_smart_rollup_host::runtime::Runtime>(host: &mut Host) {
    host.write_debug("Kernel started.\n");
    while let Some(msg) = host.read_input().unwrap() {
        handle_message(host, msg);
    }
}
