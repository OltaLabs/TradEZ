extern crate alloc;

use alloy_primitives::Signature;
use rlp::Encodable;
use tezos_smart_rollup::inbox::InboxMessage;
use tezos_smart_rollup::michelson::MichelsonBytes;
use tezos_smart_rollup::prelude::*;
use tradez_types::{
    KernelMessage, SignedInput, address::Address, currencies::Currencies, orderbook::{Event, OrderBook}, position::Side
};

use crate::account::Account;

pub mod account;

//TODO: Fix all unwraps
fn handle_message(host: &mut impl Runtime, msg: impl AsRef<[u8]>) {
    let mut orderbook = OrderBook::load(host).unwrap();
    if let Some((_, msg)) = InboxMessage::<MichelsonBytes>::parse(msg.as_ref()).ok() {
        match msg {
            InboxMessage::External(data) => {
                let SignedInput { message, signature }: SignedInput<KernelMessage> =
                    rlp::decode(data).unwrap();
                match message {
                    KernelMessage::PlaceOrder(order) => {
                        host.write_debug(&format!(
                            "Received Order: side={}, size={}, price={}\n",
                            order.side, order.size, order.price
                        ));
                        let signature = Signature::from_raw(&signature).unwrap();
                        let caller = Address::from(
                            signature
                                .recover_address_from_msg(order.rlp_bytes())
                                .unwrap(),
                        );
                        let mut account = Account::load(host, &caller)
                            .unwrap()
                            .unwrap_or(Account::new(caller));
                        // TODO: Check nonce with account nonce (can't do now because of tests)
                        account.nonce = account.nonce.checked_add(1).unwrap();
                        host.write_debug(&format!(
                            "Account before placing order: {:?}\n",
                            account
                        ));
                        match order.side {
                            Side::Ask => {
                                let balance = account.balances.entry(Currencies::XTZ).or_insert(0);
                                *balance = balance.checked_sub(order.size).unwrap();
                                // Fees 0.1%
                                let fee = std::cmp::min(1, order.size.checked_div(1000).unwrap());
                                *balance = balance.checked_sub(fee).unwrap();
                            }
                            Side::Bid => {
                                let balance = account.balances.entry(Currencies::USDC).or_insert(0);
                                *balance = balance
                                    .checked_sub(order.size.checked_mul(order.price).unwrap())
                                    .unwrap();
                                // Fees 0.1%
                                let fee = std::cmp::min(1, order.size.checked_div(1000).unwrap());
                                *balance = balance.checked_sub(fee).unwrap();
                            }
                        };
                        account.save(host).unwrap();
                        let mut events = vec![];
                        host.write_debug(&format!("Place order for address: {:?}\n", caller));
                        orderbook.place_limit(
                            caller,
                            order.side,
                            order.price,
                            order.size,
                            //TODO: Fix
                            order.nonce,
                            &mut events,
                        );
                        for event in events {
                            host.write_debug(&format!("Order book event: {:?}\n", event));
                            if let Event::Trade {
                                maker_id: _,
                                maker_user,
                                taker_id: _,
                                taker_user,
                                price,
                                qty,
                            } = event
                            {
                                // Update balances for maker and taker
                                let mut maker_account = Account::load(host, &maker_user)
                                    .unwrap()
                                    .unwrap_or(Account::new(maker_user));
                                let mut taker_account = Account::load(host, &taker_user)
                                    .unwrap()
                                    .unwrap_or(Account::new(taker_user));
                                // Maker sold qty of XTZ, receives USDC
                                let maker_xtz_balance =
                                    maker_account.balances.entry(Currencies::XTZ).or_insert(0);
                                *maker_xtz_balance = maker_xtz_balance.checked_sub(qty).unwrap();
                                let maker_usdc_balance =
                                    maker_account.balances.entry(Currencies::USDC).or_insert(0);
                                *maker_usdc_balance = maker_usdc_balance
                                    .checked_add(qty.checked_mul(price).unwrap())
                                    .unwrap();
                                // Taker bought qty of XTZ, pays USDC
                                let taker_xtz_balance =
                                    taker_account.balances.entry(Currencies::XTZ).or_insert(0);
                                *taker_xtz_balance = taker_xtz_balance.checked_add(qty).unwrap();
                                let taker_usdc_balance =
                                    taker_account.balances.entry(Currencies::USDC).or_insert(0);
                                *taker_usdc_balance = taker_usdc_balance
                                    .checked_sub(qty.checked_mul(price).unwrap())
                                    .unwrap();
                                // Save updated accounts
                                maker_account.save(host).unwrap();
                                taker_account.save(host).unwrap();
                            }
                        }
                    }
                    KernelMessage::CancelOrder(_cancel_order) => {
                        host.write_debug("Received Cancel Order\n");
                        // Implement cancel order logic here
                    }
                    KernelMessage::Faucet(faucet) => {
                        let signature = Signature::from_raw(&signature).unwrap();
                        let caller = Address::from(
                            signature
                                .recover_address_from_msg(faucet.rlp_bytes())
                                .unwrap(),
                        );
                        host.write_debug(&format!(
                            "Faucet request: user={:?}, amount={}\n",
                            caller, faucet.amount
                        ));
                        let mut account = Account::load(host, &caller)
                            .unwrap()
                            .unwrap_or(Account::new(caller));
                        let balance = account.balances.entry(Currencies::XTZ).or_insert(0);
                        *balance = balance.checked_add(faucet.amount).unwrap();
                        account.save(host).unwrap();
                    }
                }
            }
            InboxMessage::Internal(_) => {}
        }
    }
    orderbook.save(host).unwrap();
}

// TODO: Have a persistent order book for the sequencer native execution
#[entrypoint::main]
pub fn kernel_loop<Host: tezos_smart_rollup_host::runtime::Runtime>(host: &mut Host) {
    host.write_debug("Kernel started.\n");
    while let Some(msg) = host.read_input().unwrap() {
        handle_message(host, msg);
    }
}
