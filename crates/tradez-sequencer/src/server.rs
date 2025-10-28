use alloy_primitives::hex::FromHex;
use jsonrpsee::{core::RpcResult, server::ServerBuilder, types::ErrorObject};
use rlp::Encodable;
use tradez_kernel::{account::Account, kernel_loop};
use tradez_types::{
    KernelMessage, SignedInput, address::Address, api::TradezRpcServer, currencies::Currencies, orderbook::OrderBook, position::{APIOrder, CancelOrder, Faucet, Price, Qty}
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
        let inputs = vec![
            SignedInput::new(KernelMessage::PlaceOrder(api_order), signature)
                .rlp_bytes()
                .to_vec(),
        ];
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

    async fn cancel_order(&self, params: CancelOrder, signature: Vec<u8>) -> RpcResult<String> {
        let inputs = vec![
            SignedInput::new(KernelMessage::CancelOrder(params), signature)
                .rlp_bytes()
                .to_vec(),
        ];
        let mut host = SequencerHost::new(inputs.clone(), self.data_dir.clone());
        println!("Executing cancel order in native kernel...");
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
        Ok(String::from("Cancel request received"))
    }

    async fn faucet(&self, params: Faucet, signature: Vec<u8>) -> RpcResult<String> {
        let inputs = vec![
            SignedInput::new(KernelMessage::Faucet(params), signature)
                .rlp_bytes()
                .to_vec(),
        ];
        let mut host = SequencerHost::new(inputs.clone(), self.data_dir.clone());
        println!("Executing faucet in native kernel...");
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
        Ok(String::from("Faucet request received"))
    }

    async fn get_balances(&self, address: String) -> RpcResult<Vec<(Currencies, u64)>> {
        let addr = Address::from_hex(&address).map_err(|e| {
            ErrorObject::owned::<()>(-32000, format!("Failed to decode address: {:?}", e), None)
        })?;
        let mut host = SequencerHost::new(vec![], self.data_dir.clone());
        let account = Account::load(&mut host, &addr)
            .map_err(|e| ErrorObject::owned::<()>(-32000, format!("Failed to load account: {:?}", e), None))?
            .unwrap_or_else(|| Account::new(addr));
        let balances: Vec<(tradez_types::currencies::Currencies, u64)> = account
            .balances
            .into_iter()
            .collect();
        Ok(balances)
    }

    async fn get_orderbook_state(&self) ->  RpcResult<(Vec<(Price, Qty)>, Vec<(Price, Qty)>)> {
        let mut host = SequencerHost::new(vec![], self.data_dir.clone());
        let orderbook = OrderBook::load(&mut host)
            .map_err(|e| ErrorObject::owned::<()>(-32000, format!("Failed to load orderbook: {:?}", e), None))?;
        let mut bids = Vec::new();
        for (price, levels) in orderbook.bids.iter().rev() {
            let total_qty: Qty = levels.iter().map(|level| level.qty).sum();
            bids.push((*price, total_qty));
        }
        let mut asks = Vec::new();
        for (price, levels) in orderbook.asks.iter() {
            let total_qty: Qty = levels.iter().map(|level| level.qty).sum();
            asks.push((*price, total_qty));
        }
        Ok((bids, asks))
    }
}

pub async fn launch_server(
    rpc_port: u16,
    smart_rollup_addr: String,
    data_dir: String,
) -> std::io::Result<()> {
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
