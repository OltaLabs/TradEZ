use alloy_primitives::Address;
use serde::{Deserialize, Serialize};

pub type Price = u64; // microUSDC par XTZ (1e6)
pub type Qty = u64; // microXTZ (1e6)
pub type Ts = u64; // timestamp fourni par l'input (déterministe)

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Order {
    pub id: u64,
    pub user: Address,
    pub side: Side,
    pub ord_type: OrdType,
    pub price: Price,   // ignoré si Market
    pub qty: Qty,       // quantité initiale
    pub remaining: Qty, // quantité restante
    pub ts: Ts,         // pour FIFO intra-niveau
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum OrdType {
    Limit,
    Market,
}
