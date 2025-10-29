extern crate alloc;

use alloy_primitives::Signature;
use rlp::Encodable;
use tezos_smart_rollup::inbox::InboxMessage;
use tezos_smart_rollup::michelson::MichelsonBytes;
use tezos_smart_rollup::prelude::*;
use tradez_types::{
    KernelMessage, SignedInput,
    address::Address,
    currencies::Currencies,
    orderbook::{Event, OrderBook},
    position::{Side, UserOrder},
};

use crate::account::Account;

pub mod account;

const DECIMALS: u128 = 1_000_000;

fn quote_value(qty: u64, price: u64) -> Option<u64> {
    let product = (qty as u128).checked_mul(price as u128)?;
    let value = product / DECIMALS;
    if value > u64::MAX as u128 {
        None
    } else {
        Some(value as u64)
    }
}

fn trading_fee(amount: u64) -> u64 {
    if amount == 0 {
        0
    } else {
        let proportional_fee = amount / 1000;
        proportional_fee.max(1)
    }
}

fn amount_with_fee(amount: u64) -> Option<u64> {
    let fee = trading_fee(amount);
    amount.checked_add(fee)
}

//TODO: Fix all unwraps and verify all computations
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
                        host.write_debug(&format!("Account before placing order: {:?}\n", account));
                        // TODO: Fix computations and unwraps
                        match order.side {
                            Side::Ask => {
                                let balance = account.balances.entry(Currencies::XTZ).or_insert(0);
                                let Some(total_xtz) = amount_with_fee(order.size) else {
                                    host.write_debug(
                                        "Failed to compute total XTZ with fee for ask order\n",
                                    );
                                    return;
                                };
                                if let Some(updated_balance) = (*balance).checked_sub(total_xtz) {
                                    *balance = updated_balance;
                                } else {
                                    host.write_debug("Insufficient XTZ balance for ask order\n");
                                    return;
                                }
                            }
                            Side::Bid => {
                                let balance = account.balances.entry(Currencies::USDC).or_insert(0);
                                let Some(required_usdc) = quote_value(order.size, order.price)
                                else {
                                    host.write_debug(
                                        "Failed to compute notional value for bid order\n",
                                    );
                                    return;
                                };
                                let Some(total_usdc) = amount_with_fee(required_usdc) else {
                                    host.write_debug(
                                        "Failed to compute total USDC with fee for bid order\n",
                                    );
                                    return;
                                };
                                if let Some(updated_balance) = (*balance).checked_sub(total_usdc) {
                                    *balance = updated_balance;
                                } else {
                                    host.write_debug("Insufficient USDC balance for bid order\n");
                                    return;
                                }
                            }
                        };
                        account
                            .orders
                            .insert(orderbook.next_id, UserOrder::from(order));
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
                            match event {
                                Event::Trade {
                                    maker_id,
                                    maker_user,
                                    taker_id,
                                    taker_user,
                                    price,
                                    qty,
                                } => {
                                    // Update balances for maker and taker
                                    let mut maker_account = Account::load(host, &maker_user)
                                        .unwrap()
                                        .unwrap_or(Account::new(maker_user));
                                    let mut taker_account = Account::load(host, &taker_user)
                                        .unwrap()
                                        .unwrap_or(Account::new(taker_user));
                                    let Some(trade_value) = quote_value(qty, price) else {
                                        host.write_debug(
                                            "Failed to compute trade notional value\n",
                                        );
                                        continue;
                                    };

                                    let Some(maker_order_snapshot) =
                                        maker_account.orders.get(&maker_id).cloned()
                                    else {
                                        host.write_debug(
                                            "Maker order not found while processing trade\n",
                                        );
                                        continue;
                                    };
                                    let Some(taker_order_snapshot) =
                                        taker_account.orders.get(&taker_id).cloned()
                                    else {
                                        host.write_debug(
                                            "Taker order not found while processing trade\n",
                                        );
                                        continue;
                                    };

                                    let Some(maker_remaining) =
                                        maker_order_snapshot.remaining.checked_sub(qty)
                                    else {
                                        host.write_debug(
                                            "Trade quantity exceeds maker remaining\n",
                                        );
                                        continue;
                                    };
                                    let Some(taker_remaining) =
                                        taker_order_snapshot.remaining.checked_sub(qty)
                                    else {
                                        host.write_debug(
                                            "Trade quantity exceeds taker remaining\n",
                                        );
                                        continue;
                                    };

                                    {
                                        if let Some(maker_order) =
                                            maker_account.orders.get_mut(&maker_id)
                                        {
                                            maker_order.remaining = maker_remaining;
                                        }
                                    }
                                    if maker_remaining == 0 {
                                        maker_account.orders.remove(&maker_id);
                                    }
                                    {
                                        if let Some(taker_order) =
                                            taker_account.orders.get_mut(&taker_id)
                                        {
                                            taker_order.remaining = taker_remaining;
                                        }
                                    }
                                    if taker_remaining == 0 {
                                        taker_account.orders.remove(&taker_id);
                                    }

                                    // Apply maker side effects
                                    match maker_order_snapshot.side {
                                        Side::Ask => {
                                            let maker_usdc_balance = maker_account
                                                .balances
                                                .entry(Currencies::USDC)
                                                .or_insert(0);
                                            *maker_usdc_balance = maker_usdc_balance
                                                .checked_add(trade_value)
                                                .unwrap();
                                        }
                                        Side::Bid => {
                                            let maker_xtz_balance = maker_account
                                                .balances
                                                .entry(Currencies::XTZ)
                                                .or_insert(0);
                                            *maker_xtz_balance =
                                                maker_xtz_balance.checked_add(qty).unwrap();
                                        }
                                    }

                                    // Apply taker side effects
                                    match taker_order_snapshot.side {
                                        Side::Bid => {
                                            let taker_xtz_balance = taker_account
                                                .balances
                                                .entry(Currencies::XTZ)
                                                .or_insert(0);
                                            *taker_xtz_balance =
                                                taker_xtz_balance.checked_add(qty).unwrap();

                                            if let Some(reserved_value) =
                                                quote_value(qty, taker_order_snapshot.price)
                                            {
                                                if reserved_value >= trade_value {
                                                    let refund = reserved_value - trade_value;
                                                    if refund > 0 {
                                                        let taker_usdc_balance = taker_account
                                                            .balances
                                                            .entry(Currencies::USDC)
                                                            .or_insert(0);
                                                        *taker_usdc_balance = taker_usdc_balance
                                                            .checked_add(refund)
                                                            .unwrap();
                                                    }
                                                } else {
                                                    host.write_debug("Reserved value lower than trade value for taker bid\n");
                                                }
                                            } else {
                                                host.write_debug("Failed to compute reserved value for taker bid\n");
                                            }
                                        }
                                        Side::Ask => {
                                            let taker_usdc_balance = taker_account
                                                .balances
                                                .entry(Currencies::USDC)
                                                .or_insert(0);
                                            *taker_usdc_balance = taker_usdc_balance
                                                .checked_add(trade_value)
                                                .unwrap();
                                        }
                                    }

                                    // Save updated accounts
                                    maker_account.save(host).unwrap();
                                    taker_account.save(host).unwrap();
                                }
                                Event::Done { id, user } => {
                                    host.write_debug(&format!(
                                        "Order done: ID={}, User={:?}\n",
                                        id, user
                                    ));
                                    // Remove order from user's account
                                    let mut user_account = Account::load(host, &user)
                                        .unwrap()
                                        .unwrap_or(Account::new(user));
                                    user_account.orders.remove(&id);
                                    user_account.save(host).unwrap();
                                }
                                _ => {}
                            }
                        }
                    }
                    KernelMessage::CancelOrder(cancel_order) => {
                        host.write_debug("Received Cancel Order\n");
                        let signature = Signature::from_raw(&signature).unwrap();
                        let caller = Address::from(
                            signature
                                .recover_address_from_msg(cancel_order.rlp_bytes())
                                .unwrap(),
                        );
                        let mut account = Account::load(host, &caller)
                            .unwrap()
                            .unwrap_or(Account::new(caller));
                        host.write_debug(&format!(
                            "Account before cancelling order: {:?}\n",
                            account
                        ));
                        if let Some(user_order) = account.orders.remove(&cancel_order.order_id) {
                            // Refund the remaining amount to the user's balance
                            match user_order.side {
                                Side::Ask => {
                                    let balance =
                                        account.balances.entry(Currencies::XTZ).or_insert(0);
                                    *balance = balance.checked_add(user_order.remaining).unwrap();
                                }
                                Side::Bid => {
                                    let balance =
                                        account.balances.entry(Currencies::USDC).or_insert(0);
                                    if let Some(refund) =
                                        quote_value(user_order.remaining, user_order.price)
                                    {
                                        *balance = balance.checked_add(refund).unwrap();
                                    } else {
                                        host.write_debug("Overflow while computing refund for cancelled bid order\n");
                                    }
                                }
                            }
                            account.save(host).unwrap();
                            let mut events = vec![];
                            orderbook.cancel(user_order.side, cancel_order.order_id, &mut events);
                            host.write_debug(&format!(
                                "Cancelled order ID: {} for address: {:?}\n",
                                cancel_order.order_id, caller
                            ));
                            for event in events {
                                host.write_debug(&format!("Order book event: {:?}\n", event));
                            }
                        } else {
                            host.write_debug(&format!(
                                "Order ID: {} not found for address: {:?}\n",
                                cancel_order.order_id, caller
                            ));
                        }
                    }
                    KernelMessage::Faucet(faucet) => {
                        let signature = Signature::from_raw(&signature).unwrap();
                        let caller = Address::from(
                            signature
                                .recover_address_from_msg(faucet.rlp_bytes())
                                .unwrap(),
                        );
                        host.write_debug(&format!(
                            "Faucet request: user={:?}, amount={} currency={:?}\n",
                            caller, faucet.amount, faucet.currency
                        ));
                        let mut account = Account::load(host, &caller)
                            .unwrap()
                            .unwrap_or(Account::new(caller));
                        match faucet.currency {
                            Currencies::XTZ => {
                                let balance = account.balances.entry(Currencies::XTZ).or_insert(0);
                                *balance = balance.checked_add(faucet.amount).unwrap();
                            }
                            Currencies::USDC => {
                                let balance = account.balances.entry(Currencies::USDC).or_insert(0);
                                *balance = balance.checked_add(faucet.amount).unwrap();
                            }
                        };
                        host.write_debug(&format!("Account after faucet: {:?}\n", account));
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
