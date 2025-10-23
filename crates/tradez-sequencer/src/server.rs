use jsonrpsee::{core::RpcResult, server::ServerBuilder};
use rlp::Encodable;
use tradez_kernel::kernel_loop;
use tradez_types::{
    TradezRpcServer,
    position::{APIOrder, CancelOrder},
};

use crate::host::SequencerHost;

pub struct TradezRpcImpl {
    pub smart_rollup_node_client: tradez_octez::smart_rollup_node::SmartRollupClient,
}

#[async_trait::async_trait]
impl TradezRpcServer for TradezRpcImpl {
    async fn send_order(&self, api_order: APIOrder) -> RpcResult<String> {
        println!(
            "Received order: side={}, size={}, price={}",
            api_order.side, api_order.size, api_order.price
        );
        let rlp_encoded = api_order.rlp_bytes().to_vec();
        let inputs = vec![rlp_encoded.clone()];
        let mut host = SequencerHost {
            inputs: inputs.iter().cloned().collect(),
        };
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
        println!("With signature: {}", api_order.signature);
        Ok(String::from("Order received"))
    }

    async fn cancel_order(&self, params: CancelOrder) -> RpcResult<String> {
        println!("Received cancel request for order ID: {}", params.order_id);
        println!("With signature: {}", params.signature);
        Ok(String::from("Cancel request received"))
    }
}

pub async fn launch_server(rpc_port: u16, smart_rollup_addr: String) -> std::io::Result<()> {
    println!("Starting TradEZ JSON-RPC server...");

    let rpc_impl = TradezRpcImpl {
        smart_rollup_node_client: tradez_octez::smart_rollup_node::SmartRollupClient::new(
            &smart_rollup_addr,
        ),
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
