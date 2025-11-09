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
    position::{APIOrder, CancelOrder, Faucet, Side},
};

use crate::account::Account;

pub mod account;

const DECIMALS: u128 = 1_000_000;

fn opposite_side(side: Side) -> Side {
    match side {
        Side::Bid => Side::Ask,
        Side::Ask => Side::Bid,
    }
}

fn get_or_load_account<'a>(
    host: &mut impl Runtime,
    cache: &'a mut Vec<(Address, Account)>,
    user: Address,
) -> &'a mut Account {
    if let Some(pos) = cache.iter().position(|(address, _)| *address == user) {
        return &mut cache[pos].1;
    }
    let account = Account::load(host, &user)
        .unwrap()
        .unwrap_or(Account::new(user));
    cache.push((user, account));
    let len = cache.len();
    &mut cache[len - 1].1
}

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

fn handle_trade_event(
    host: &mut impl Runtime,
    accounts: &mut Vec<(Address, Account)>,
    taker_side: Side,
    order_price: u64,
    maker_user: Address,
    taker_user: Address,
    qty: u64,
    trade_value: u64,
) -> u64 {
    let maker_side = opposite_side(taker_side);
    let taker_fee = match taker_side {
        Side::Ask => trading_fee(qty),
        Side::Bid => trading_fee(trade_value),
    };

    if maker_user == taker_user {
        let account = get_or_load_account(host, accounts, maker_user);

        match maker_side {
            Side::Ask => {
                let maker_usdc_balance = account.balances.entry(Currencies::USDC).or_insert(0);
                *maker_usdc_balance = maker_usdc_balance.checked_add(trade_value).unwrap();
            }
            Side::Bid => {
                let maker_xtz_balance = account.balances.entry(Currencies::XTZ).or_insert(0);
                *maker_xtz_balance = maker_xtz_balance.checked_add(qty).unwrap();
            }
        }

        match taker_side {
            Side::Bid => {
                let taker_xtz_balance = account.balances.entry(Currencies::XTZ).or_insert(0);
                *taker_xtz_balance = taker_xtz_balance.checked_add(qty).unwrap();

                if taker_fee > 0 {
                    let taker_usdc_balance = account.balances.entry(Currencies::USDC).or_insert(0);
                    if let Some(updated) = taker_usdc_balance.checked_sub(taker_fee) {
                        *taker_usdc_balance = updated;
                    } else {
                        host.write_debug("Taker bid insufficient USDC for fee\n");
                    }
                }

                if let Some(reserved_value) = quote_value(qty, order_price) {
                    if reserved_value >= trade_value {
                        let refund = reserved_value - trade_value;
                        if refund > 0 {
                            let taker_usdc_balance =
                                account.balances.entry(Currencies::USDC).or_insert(0);
                            *taker_usdc_balance = taker_usdc_balance.checked_add(refund).unwrap();
                        }
                    } else {
                        host.write_debug("Reserved value lower than trade value for taker bid\n");
                    }
                } else {
                    host.write_debug("Failed to compute reserved value for taker bid\n");
                }
            }
            Side::Ask => {
                if taker_fee > 0 {
                    let taker_xtz_balance = account.balances.entry(Currencies::XTZ).or_insert(0);
                    if let Some(updated) = taker_xtz_balance.checked_sub(taker_fee) {
                        *taker_xtz_balance = updated;
                    } else {
                        host.write_debug("Taker ask insufficient XTZ for fee\n");
                    }
                }
                let taker_usdc_balance = account.balances.entry(Currencies::USDC).or_insert(0);
                *taker_usdc_balance = taker_usdc_balance.checked_add(trade_value).unwrap();
            }
        }
    } else {
        let maker_account = get_or_load_account(host, accounts, maker_user);
        match maker_side {
            Side::Ask => {
                let maker_usdc_balance =
                    maker_account.balances.entry(Currencies::USDC).or_insert(0);
                *maker_usdc_balance = maker_usdc_balance.checked_add(trade_value).unwrap();
            }
            Side::Bid => {
                let maker_xtz_balance = maker_account.balances.entry(Currencies::XTZ).or_insert(0);
                *maker_xtz_balance = maker_xtz_balance.checked_add(qty).unwrap();
            }
        }

        let taker_account = get_or_load_account(host, accounts, taker_user);

        match taker_side {
            Side::Bid => {
                let taker_xtz_balance = taker_account.balances.entry(Currencies::XTZ).or_insert(0);
                *taker_xtz_balance = taker_xtz_balance.checked_add(qty).unwrap();

                if taker_fee > 0 {
                    let taker_usdc_balance =
                        taker_account.balances.entry(Currencies::USDC).or_insert(0);
                    if let Some(updated) = taker_usdc_balance.checked_sub(taker_fee) {
                        *taker_usdc_balance = updated;
                    } else {
                        host.write_debug("Taker bid insufficient USDC for fee\n");
                    }
                }

                if let Some(reserved_value) = quote_value(qty, order_price) {
                    if reserved_value >= trade_value {
                        let refund = reserved_value - trade_value;
                        if refund > 0 {
                            let taker_usdc_balance =
                                taker_account.balances.entry(Currencies::USDC).or_insert(0);
                            *taker_usdc_balance = taker_usdc_balance.checked_add(refund).unwrap();
                        }
                    } else {
                        host.write_debug("Reserved value lower than trade value for taker bid\n");
                    }
                } else {
                    host.write_debug("Failed to compute reserved value for taker bid\n");
                }
            }
            Side::Ask => {
                if taker_fee > 0 {
                    let taker_xtz_balance =
                        taker_account.balances.entry(Currencies::XTZ).or_insert(0);
                    if let Some(updated) = taker_xtz_balance.checked_sub(taker_fee) {
                        *taker_xtz_balance = updated;
                    } else {
                        host.write_debug("Taker ask insufficient XTZ for fee\n");
                    }
                }
                let taker_usdc_balance =
                    taker_account.balances.entry(Currencies::USDC).or_insert(0);
                *taker_usdc_balance = taker_usdc_balance.checked_add(trade_value).unwrap();
            }
        }
    }

    qty
}

fn handle_message(host: &mut impl Runtime, msg: impl AsRef<[u8]>) {
    let Some((_, msg)) = InboxMessage::<MichelsonBytes>::parse(msg.as_ref()).ok() else {
        return;
    };

    let mut orderbook = OrderBook::load(host).unwrap();
    if let InboxMessage::External(data) = msg {
        let SignedInput { message, signature }: SignedInput<KernelMessage> =
            rlp::decode(data).unwrap();
        let result = match message {
            KernelMessage::PlaceOrder(order) => {
                process_place_order(host, &mut orderbook, order, &signature)
            }
            KernelMessage::CancelOrder(cancel_order) => {
                process_cancel_order(host, &mut orderbook, cancel_order, &signature)
            }
            KernelMessage::Faucet(faucet) => process_faucet(host, faucet, &signature),
        };

        if result.is_err() {
            return;
        }
    }
    orderbook.save(host).unwrap();
}

fn process_place_order(
    host: &mut impl Runtime,
    orderbook: &mut OrderBook,
    order: APIOrder,
    signature: &[u8],
) -> Result<(), ()> {
    host.write_debug(&format!(
        "Received Order: side={}, size={}, price={}\n",
        order.side, order.size, order.price
    ));
    let signature = Signature::from_raw(signature).map_err(|_| ())?;
    let caller = Address::from(
        signature
            .recover_address_from_msg(order.rlp_bytes())
            .map_err(|_| ())?,
    );

    let caller_account = Account::load(host, &caller)
        .unwrap()
        .unwrap_or(Account::new(caller));
    let mut accounts = vec![(caller, caller_account)];

    {
        let caller_account = get_or_load_account(host, &mut accounts, caller);
        caller_account.nonce = caller_account.nonce.checked_add(1).unwrap();
        match order.side {
            Side::Ask => {
                let balance = caller_account.balances.entry(Currencies::XTZ).or_insert(0);
                let Some(total_xtz) = amount_with_fee(order.size) else {
                    host.write_debug("Failed to compute total XTZ with fee for ask order\n");
                    return Err(());
                };
                if *balance < total_xtz {
                    host.write_debug("Insufficient XTZ balance for ask order\n");
                    return Err(());
                }
                *balance = balance
                    .checked_sub(order.size)
                    .expect("balance >= order.size");
            }
            Side::Bid => {
                let balance = caller_account.balances.entry(Currencies::USDC).or_insert(0);
                let Some(required_usdc) = quote_value(order.size, order.price) else {
                    host.write_debug("Failed to compute notional value for bid order\n");
                    return Err(());
                };
                let Some(total_usdc) = amount_with_fee(required_usdc) else {
                    host.write_debug("Failed to compute total USDC with fee for bid order\n");
                    return Err(());
                };
                if *balance < total_usdc {
                    host.write_debug("Insufficient USDC balance for bid order\n");
                    return Err(());
                }
                *balance = balance
                    .checked_sub(required_usdc)
                    .expect("balance >= required_usdc");
            }
        }
    }

    let mut events = vec![];
    let order_id = orderbook.place_limit(
        caller,
        order.side,
        order.price,
        order.size,
        order.nonce,
        &mut events,
    );

    let mut taker_total_filled: u64 = 0;
    let mut taker_done = false;

    for event in events {
        host.write_output(&event.rlp_bytes()).unwrap();
        match event {
            Event::Placed { .. } => {}
            Event::Trade {
                maker_user,
                taker_user,
                price,
                qty,
                ..
            } => {
                let Some(trade_value) = quote_value(qty, price) else {
                    host.write_debug("Failed to compute trade notional value\n");
                    continue;
                };
                taker_total_filled = taker_total_filled.saturating_add(handle_trade_event(
                    host,
                    &mut accounts,
                    order.side,
                    order.price,
                    maker_user,
                    taker_user,
                    qty,
                    trade_value,
                ));
            }
            Event::Done { id, user } => {
                let account = get_or_load_account(host, &mut accounts, user);
                account.orders.remove(&id);
                if user == caller && id == order_id {
                    taker_done = true;
                }
            }
            Event::Cancelled { .. } => {}
        }
    }

    let caller_account = get_or_load_account(host, &mut accounts, caller);
    if taker_done || taker_total_filled >= order.size {
        caller_account.orders.remove(&order_id);
    } else {
        caller_account.orders.insert(order_id);
    }

    for (_, account) in &accounts {
        account.save(host).unwrap();
    }
    Ok(())
}

fn process_cancel_order(
    host: &mut impl Runtime,
    orderbook: &mut OrderBook,
    cancel_order: CancelOrder,
    signature: &[u8],
) -> Result<(), ()> {
    host.write_debug("Received Cancel Order\n");
    let signature = Signature::from_raw(signature).map_err(|_| ())?;
    let caller = Address::from(
        signature
            .recover_address_from_msg(cancel_order.rlp_bytes())
            .map_err(|_| ())?,
    );
    let mut account = Account::load(host, &caller)
        .unwrap()
        .unwrap_or(Account::new(caller));
    if !account.orders.contains(&cancel_order.order_id) {
        host.write_debug("Order not found in account during cancel\n");
        return Err(());
    }

    if let Some(order) = orderbook.get_order(cancel_order.order_id) {
        match order.side {
            Side::Ask => {
                let balance = account.balances.entry(Currencies::XTZ).or_insert(0);
                *balance = balance.checked_add(order.remaining).unwrap();
            }
            Side::Bid => {
                if let Some(refund) = quote_value(order.remaining, order.price) {
                    let balance = account.balances.entry(Currencies::USDC).or_insert(0);
                    *balance = balance.checked_add(refund).unwrap();
                } else {
                    host.write_debug("Failed to compute refund for cancelled bid order\n");
                }
            }
        }
        account.orders.remove(&cancel_order.order_id);
        account.save(host).unwrap();

        let mut events = vec![];
        orderbook.cancel(order.side, cancel_order.order_id, caller, &mut events);
        for event in events {
            host.write_output(&event.rlp_bytes()).unwrap();
            host.write_debug(&format!("Order book event: {:?}\n", event));
        }
        Ok(())
    } else {
        host.write_debug("Order not found in orderbook during cancel\n");
        Err(())
    }
}

fn process_faucet(host: &mut impl Runtime, faucet: Faucet, signature: &[u8]) -> Result<(), ()> {
    let signature = Signature::from_raw(signature).map_err(|_| ())?;
    let caller = Address::from(
        signature
            .recover_address_from_msg(faucet.rlp_bytes())
            .map_err(|_| ())?,
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
    }
    account.save(host).unwrap();
    Ok(())
}

#[entrypoint::main]
pub fn kernel_loop<Host: tezos_smart_rollup_host::runtime::Runtime>(host: &mut Host) {
    host.write_debug("Kernel started.\n");
    while let Some(msg) = host.read_input().unwrap() {
        handle_message(host, msg);
    }
}
