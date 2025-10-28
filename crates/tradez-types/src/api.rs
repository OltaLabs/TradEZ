use jsonrpsee::{core::RpcResult, proc_macros::rpc};

use crate::position::{APIOrder, CancelOrder, Faucet};

#[rpc(client, server)]
pub trait TradezRpc {
    #[method(name = "send_order")]
    async fn send_order(&self, api_order: APIOrder, signature: Vec<u8>) -> RpcResult<String>;

    #[method(name = "cancel_order")]
    async fn cancel_order(&self, params: CancelOrder, signature: Vec<u8>) -> RpcResult<String>;

    #[method(name = "faucet")]
    async fn faucet(&self, params: Faucet, signature: Vec<u8>) -> RpcResult<String>;
}
