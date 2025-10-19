use tradez_types::position::Order;

#[derive(Debug)]
pub struct Message {
    pub action: OrderAction,
    pub signature: Vec<u8>,
}

#[derive(Debug)]
pub enum OrderAction {
    Open(Order),
    Close(u64),
}
