use std::fmt::Display;

use rlp::{Decodable, Encodable, RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

use crate::{address::Address, currencies::Currencies};

pub type Price = u64; // microUSDC par XTZ (1e6)
pub type Qty = u64; // microXTZ (1e6)

#[derive(
    Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable, Default, PartialEq, Eq, Clone, Copy,
)]
pub struct APIOrder {
    pub side: Side,
    pub size: Qty,
    pub price: Price,
    pub nonce: u64,
}

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable, PartialEq, Eq)]
pub struct CancelOrder {
    pub order_id: u64,
}

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable, PartialEq, Eq)]
pub struct Faucet {
    pub amount: Qty,
    pub currency: Currencies,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, RlpDecodable, RlpEncodable)]
pub struct Order {
    pub id: u64,
    pub user: Address,
    pub side: Side,
    pub ord_type: OrdType,
    pub price: Price,   // ignoré si Market
    pub qty: Qty,       // quantité initiale
    pub remaining: Qty, // quantité restante
    pub nonce: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, RlpDecodable, RlpEncodable)]
pub struct UserOrder {
    pub side: Side,
    pub ord_type: OrdType,
    pub price: Price,   // ignoré si Market
    pub qty: Qty,       // quantité initiale
    pub remaining: Qty, // quantité restante
    pub nonce: u64,
}

impl From<APIOrder> for UserOrder {
    fn from(api_order: APIOrder) -> Self {
        UserOrder {
            side: api_order.side,
            ord_type: OrdType::Limit,
            price: api_order.price,
            qty: api_order.size,
            remaining: api_order.size,
            nonce: api_order.nonce,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum Side {
    #[default]
    Bid,
    Ask,
}

impl Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Bid => write!(f, "Bid"),
            Side::Ask => write!(f, "Ask"),
        }
    }
}

impl Encodable for Side {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        match self {
            Side::Bid => s.append_internal(&0u8),
            Side::Ask => s.append_internal(&1u8),
        };
    }
}

impl Decodable for Side {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let value: u8 = rlp.as_val()?;
        match value {
            0 => Ok(Side::Bid),
            1 => Ok(Side::Ask),
            _ => Err(rlp::DecoderError::Custom("Invalid Side value")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum OrdType {
    Limit,
    Market,
}

impl Encodable for OrdType {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        match self {
            OrdType::Limit => s.append_internal(&0u8),
            OrdType::Market => s.append_internal(&1u8),
        };
    }
}

impl Decodable for OrdType {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let value: u8 = rlp.as_val()?;
        match value {
            0 => Ok(OrdType::Limit),
            1 => Ok(OrdType::Market),
            _ => Err(rlp::DecoderError::Custom("Invalid OrdType value")),
        }
    }
}
