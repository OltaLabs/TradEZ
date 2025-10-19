use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use jsonrpc_http_server::ServerBuilder;
use serde::Deserialize;
use tradez_types::position::Order;

#[derive(Debug, Deserialize)]
pub struct SendOrder {
    pub order: Order,
    pub signature: String,
}

#[derive(Debug, Deserialize)]
pub struct CancelOrder {
    pub order_id: u64,
    pub signature: String,
}

#[rpc(server)]
pub trait TradezRpc {
    #[rpc(name = "send_order")]
    fn send_order(&self, params: SendOrder) -> Result<String>;

    #[rpc(name = "cancel_order")]
    fn cancel_order(&self, params: CancelOrder) -> Result<String>;
}

pub struct TradezRpcImpl;

impl TradezRpc for TradezRpcImpl {
    fn send_order(&self, params: SendOrder) -> Result<String> {
        println!("Received order: {:?}", params.order);
        println!("With signature: {}", params.signature);
        Ok(String::from("Order received"))
    }

    fn cancel_order(&self, params: CancelOrder) -> Result<String> {
        println!("Received cancel request for order ID: {}", params.order_id);
        println!("With signature: {}", params.signature);
        Ok(String::from("Cancel request received"))
    }
}

pub fn launch_server() -> std::io::Result<()> {
    println!("Starting TradEZ JSON-RPC server...");

    let rpc_impl = TradezRpcImpl;

    let mut io = jsonrpc_core::IoHandler::new();
    io.extend_with(rpc_impl.to_delegate());

    let server = ServerBuilder::new(io)
        .threads(4)
        .start_http(&"127.0.0.1:8545".parse().unwrap())?;

    println!("JSON-RPC server running on http://127.0.0.1:8545");
    println!("Available methods:");
    println!("  - send_order");
    println!("  - cancel_order");

    server.wait();
    Ok(())
}
