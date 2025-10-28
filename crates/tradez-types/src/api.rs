use jsonrpsee::{core::RpcResult, proc_macros::rpc};

use crate::{
    currencies::Currencies,
    position::{APIOrder, CancelOrder, Faucet, Price, Qty, UserOrder},
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
}
