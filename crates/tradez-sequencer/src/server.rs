use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use jsonrpc_http_server::ServerBuilder;
use rlp::Encodable;
use serde::Deserialize;
use tradez_kernel::kernel_loop;

use crate::host::SequencerHost;

#[derive(Debug, Deserialize)]
pub struct CancelOrder {
    pub order_id: u64,
    pub signature: String,
}

#[rpc(server)]
pub trait TradezRpc {
    #[rpc(name = "send_order")]
    fn send_order(&self, side: u8, size: u64, price: u64, signature: String) -> Result<String>;

    #[rpc(name = "cancel_order")]
    fn cancel_order(&self, params: CancelOrder) -> Result<String>;
}

pub struct TradezRpcImpl;

impl TradezRpc for TradezRpcImpl {
    fn send_order(&self, side: u8, size: u64, price: u64, signature: String) -> Result<String> {
        println!(
            "Received order: side={}, size={}, price={}",
            side, size, price
        );
        let encoded_order = tradez_types::position::APIOrder {
            side,
            size,
            price,
            signature: signature.clone(),
        };
        let rlp_encoded = encoded_order.rlp_bytes().to_vec();
        let mut host = SequencerHost {
            inputs: vec![rlp_encoded].into(),
        };
        kernel_loop(&mut host);
        println!("With signature: {}", signature);
        Ok(String::from("Order received"))
    }

    fn cancel_order(&self, params: CancelOrder) -> Result<String> {
        println!("Received cancel request for order ID: {}", params.order_id);
        println!("With signature: {}", params.signature);
        Ok(String::from("Cancel request received"))
    }
}

pub fn launch_server(rpc_port: u16) -> std::io::Result<()> {
    println!("Starting TradEZ JSON-RPC server...");

    let rpc_impl = TradezRpcImpl;

    let mut io = jsonrpc_core::IoHandler::new();
    io.extend_with(rpc_impl.to_delegate());

    let server = ServerBuilder::new(io)
        .threads(4)
        .start_http(&format!("127.0.0.1:{}", rpc_port).parse().unwrap())?;

    println!("JSON-RPC server running on http://127.0.0.1:{}", rpc_port);
    println!("Available methods:");
    println!("  - send_order");
    println!("  - cancel_order");

    server.wait();
    Ok(())
}
