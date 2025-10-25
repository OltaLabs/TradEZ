use jsonrpsee::{core::RpcResult, server::ServerBuilder};
use rlp::Encodable;
use tradez_kernel::kernel_loop;
use tradez_types::{
    api::TradezRpcServer,
    position::{APIOrder, CancelOrder}, SignedInput,
};

use crate::host::SequencerHost;

pub struct TradezRpcImpl {
    pub smart_rollup_node_client: tradez_octez::smart_rollup_node::SmartRollupClient,
    pub data_dir: String,
}

#[async_trait::async_trait]
impl TradezRpcServer for TradezRpcImpl {
    async fn send_order(&self, api_order: APIOrder, signature: Vec<u8>) -> RpcResult<String> {
        println!(
            "Received order: side={}, size={}, price={}",
            api_order.side, api_order.size, api_order.price
        );
        let inputs = vec![SignedInput::new(
            api_order,
            signature,
        ).rlp_bytes().to_vec()];
        let mut host = SequencerHost::new(inputs.clone(), self.data_dir.clone());
        println!("Executing order in native kernel...");
        kernel_loop(&mut host);
        if let Err(e) = self
            .smart_rollup_node_client
            .inject_inbox_messages(inputs)
            .await
        {
            println!("Failed to inject inbox message: {:?}", e);
        } else {
            println!("Successfully injected inbox message.");
        }
        Ok(String::from("Order received"))
    }

    async fn cancel_order(&self, params: CancelOrder, _signature: Vec<u8>) -> RpcResult<String> {
        println!("Received cancel request for order ID: {}", params.order_id);
        Ok(String::from("Cancel request received"))
    }
}

pub async fn launch_server(rpc_port: u16, smart_rollup_addr: String, data_dir: String) -> std::io::Result<()> {
    println!("Starting TradEZ JSON-RPC server...");

    let rpc_impl = TradezRpcImpl {
        smart_rollup_node_client: tradez_octez::smart_rollup_node::SmartRollupClient::new(
            &smart_rollup_addr,
        ),
        data_dir,
    };

    let server = ServerBuilder::default()
        .build(&format!("127.0.0.1:{}", rpc_port))
        .await?;
    let handle = server.start(TradezRpcServer::into_rpc(rpc_impl));

    println!("JSON-RPC server running on http://127.0.0.1:{}", rpc_port);
    println!("Available methods:");
    println!("  - send_order");
    println!("  - cancel_order");

    handle.stopped().await;
    Ok(())
}
