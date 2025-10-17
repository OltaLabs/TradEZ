use serde::{Deserialize, Serialize};
use tradez_types::position::Order;

#[derive(Deserialize, Serialize, Debug)]
pub struct Message {
    pub action: OrderAction,
    pub signature: Vec<u8>
}

#[derive(Deserialize, Serialize, Debug)]
pub enum OrderAction {
    Open(Order),
    Close(u64),
}