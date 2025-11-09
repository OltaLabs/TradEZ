use jsonrpsee::{
    core::{RpcResult, SubscriptionResult},
    proc_macros::rpc,
};

use crate::{
    currencies::Currencies,
    orderbook::Event,
    position::{APIOrder, CancelOrder, Faucet, Price, Qty, Side, UserOrder},
};

#[rpc(client, server)]
pub trait TradezRpc {
    #[method(name = "send_order")]
    async fn send_order(&self, api_order: APIOrder, signature: Vec<u8>) -> RpcResult<String>;

    #[method(name = "cancel_order")]
    async fn cancel_order(&self, params: CancelOrder, signature: Vec<u8>) -> RpcResult<String>;

    #[method(name = "faucet")]
    async fn faucet(&self, params: Faucet, signature: Vec<u8>) -> RpcResult<String>;

    #[method(name = "get_balances")]
    async fn get_balances(&self, address: String) -> RpcResult<Vec<(Currencies, u64)>>;

    #[method(name = "get_orders")]
    async fn get_orders(&self, address: String) -> RpcResult<Vec<(u64, UserOrder)>>;

    #[method(name = "get_orderbook_state")]
    async fn get_orderbook_state(&self) -> RpcResult<(Vec<(Price, Qty)>, Vec<(Price, Qty)>)>;

    #[method(name = "get_history")]
    async fn get_history(&self) -> RpcResult<Vec<(u128, Qty, Price, Side)>>;

    #[subscription(name = "subscribeOrderBookState", item = (Vec<(Price, Qty)>, Vec<(Price, Qty)>))]
    async fn subscribe_order_book_state(&self) -> SubscriptionResult;

    #[subscription(name = "subscribeEvent", item = Event)]
    async fn subscribe_event(&self) -> SubscriptionResult;
}
