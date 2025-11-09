use std::sync::Arc;

use alloy_primitives::hex::FromHex;
use hyper::Method;
use jsonrpsee::{
    PendingSubscriptionSink, SubscriptionSink,
    core::{RpcResult, SubscriptionResult},
    server::ServerBuilder,
    types::ErrorObject,
};
use rlp::Encodable;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tradez_kernel::{account::Account, kernel_loop};
use tradez_types::{
    KernelMessage, SignedInput,
    address::Address,
    api::TradezRpcServer,
    currencies::Currencies,
    orderbook::OrderBook,
    position::{APIOrder, CancelOrder, Faucet, Price, Qty, Side, UserOrder},
};

use crate::host::SequencerHost;

pub struct TradezRpcImpl {
    pub smart_rollup_node_client: tradez_octez::smart_rollup_node::SmartRollupClient,
    pub host: Arc<Mutex<SequencerHost>>,
    pub subscribers: Arc<Mutex<Vec<SubscriptionSink>>>,
}

#[async_trait::async_trait]
impl TradezRpcServer for TradezRpcImpl {
    async fn send_order(&self, api_order: APIOrder, signature: Vec<u8>) -> RpcResult<String> {
        let inputs = vec![
            SignedInput::new(KernelMessage::PlaceOrder(api_order), signature)
                .rlp_bytes()
                .to_vec(),
        ];
        {
            let mut host = self.host.lock().await;
            host.add_inputs(inputs.clone());
            kernel_loop(&mut *host);
            let orderbook = OrderBook::load(&mut *host).unwrap();
            let mut bids = Vec::new();
            for (price, levels) in orderbook.bids.iter().rev() {
                let total_qty: Qty = levels.iter().map(|level| level.remaining).sum();
                bids.push((*price, total_qty));
            }
            let mut asks = Vec::new();
            for (price, levels) in orderbook.asks.iter() {
                let total_qty: Qty = levels.iter().map(|level| level.remaining).sum();
                asks.push((*price, total_qty));
            }
            let mut subscribers = self.subscribers.lock().await;
            subscribers.retain(|subscriber| !subscriber.is_closed());
            for subscriber in subscribers.iter() {
                match subscriber.method_name() {
                    "subscribeEvent" => {
                        for event in &std::mem::take(&mut host.event_to_notify) {
                            subscriber
                                .send(serde_json::value::to_raw_value(event).unwrap())
                                .await
                                .unwrap();
                        }
                    }
                    "subscribeOrderBookState" => {
                        subscriber
                            .send(
                                serde_json::value::to_raw_value(&(bids.clone(), asks.clone()))
                                    .unwrap(),
                            )
                            .await
                            .unwrap();
                    }
                    _ => {}
                }
            }
        }
        if let Err(e) = self
            .smart_rollup_node_client
            .inject_inbox_messages(inputs)
            .await
        {
            println!("Failed to inject inbox message: {:?}", e);
        }
        Ok(String::from("Order received"))
    }

    async fn cancel_order(&self, params: CancelOrder, signature: Vec<u8>) -> RpcResult<String> {
        let inputs = vec![
            SignedInput::new(KernelMessage::CancelOrder(params), signature)
                .rlp_bytes()
                .to_vec(),
        ];
        {
            let mut host = self.host.lock().await;
            host.add_inputs(inputs.clone());
            kernel_loop(&mut *host);
        }
        if let Err(e) = self
            .smart_rollup_node_client
            .inject_inbox_messages(inputs)
            .await
        {
            println!("Failed to inject inbox message: {:?}", e);
        }
        Ok(String::from("Cancel request received"))
    }

    async fn faucet(&self, params: Faucet, signature: Vec<u8>) -> RpcResult<String> {
        let inputs = vec![
            SignedInput::new(KernelMessage::Faucet(params), signature)
                .rlp_bytes()
                .to_vec(),
        ];
        {
            let mut host = self.host.lock().await;
            host.add_inputs(inputs.clone());
            kernel_loop(&mut *host);
        }
        if let Err(e) = self
            .smart_rollup_node_client
            .inject_inbox_messages(inputs)
            .await
        {
            println!("Failed to inject inbox message: {:?}", e);
        }
        Ok(String::from("Faucet request received"))
    }

    async fn get_balances(&self, address: String) -> RpcResult<Vec<(Currencies, u64)>> {
        let addr = Address::from_hex(&address).map_err(|e| {
            ErrorObject::owned::<()>(-32000, format!("Failed to decode address: {:?}", e), None)
        })?;
        let account_result = {
            let mut host = self.host.lock().await;
            Account::load(&mut *host, &addr)
        };
        let account = account_result
            .map_err(|e| {
                ErrorObject::owned::<()>(-32000, format!("Failed to load account: {:?}", e), None)
            })?
            .unwrap_or_else(|| Account::new(addr));
        let balances: Vec<(tradez_types::currencies::Currencies, u64)> =
            account.balances.into_iter().collect();
        Ok(balances)
    }

    async fn get_orders(&self, address: String) -> RpcResult<Vec<(u64, UserOrder)>> {
        let addr = Address::from_hex(&address).map_err(|e| {
            ErrorObject::owned::<()>(-32000, format!("Failed to decode address: {:?}", e), None)
        })?;
        let (account, orderbook) = {
            let mut host = self.host.lock().await;
            let account = Account::load(&mut *host, &addr).map_err(|e| {
                ErrorObject::owned::<()>(-32000, format!("Failed to load account: {:?}", e), None)
            })?;
            let orderbook = OrderBook::load(&mut *host).map_err(|e| {
                ErrorObject::owned::<()>(-32000, format!("Failed to load orderbook: {:?}", e), None)
            })?;
            (account.unwrap_or_else(|| Account::new(addr)), orderbook)
        };

        let mut orders: Vec<(u64, UserOrder)> = Vec::new();
        for id in account.orders {
            if let Some(order) = orderbook.get_order(id) {
                let user_order = UserOrder {
                    side: order.side,
                    ord_type: order.ord_type,
                    price: order.price,
                    qty: order.qty,
                    remaining: order.remaining,
                    nonce: order.nonce,
                };
                orders.push((id, user_order));
            }
        }
        Ok(orders)
    }

    async fn get_orderbook_state(&self) -> RpcResult<(Vec<(Price, Qty)>, Vec<(Price, Qty)>)> {
        let orderbook_result = {
            let mut host = self.host.lock().await;
            OrderBook::load(&mut *host)
        };
        let orderbook = orderbook_result.map_err(|e| {
            ErrorObject::owned::<()>(-32000, format!("Failed to load orderbook: {:?}", e), None)
        })?;
        let mut bids = Vec::new();
        for (price, levels) in orderbook.bids.iter().rev() {
            let total_qty: Qty = levels.iter().map(|level| level.remaining).sum();
            bids.push((*price, total_qty));
        }
        let mut asks = Vec::new();
        for (price, levels) in orderbook.asks.iter() {
            let total_qty: Qty = levels.iter().map(|level| level.remaining).sum();
            asks.push((*price, total_qty));
        }
        Ok((bids, asks))
    }

    async fn get_history(&self) -> RpcResult<Vec<(u128, Qty, Price, Side)>> {
        Ok(self.host.lock().await.read_history())
    }

    async fn subscribe_order_book_state(
        &self,
        pending: PendingSubscriptionSink,
    ) -> SubscriptionResult {
        let sink = pending.accept().await?;
        self.subscribers.lock().await.push(sink);
        Ok(())
    }

    async fn subscribe_event(&self, pending: PendingSubscriptionSink) -> SubscriptionResult {
        let sink = pending.accept().await?;
        self.subscribers.lock().await.push(sink);
        Ok(())
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
        host: Arc::new(Mutex::new(SequencerHost::new(data_dir))),
        subscribers: Arc::new(Mutex::new(Vec::new())),
    };

    let cors = CorsLayer::new()
        .allow_methods([Method::POST])
        .allow_origin(Any)
        .allow_headers([hyper::header::CONTENT_TYPE]);
    let middleware = ServiceBuilder::new().layer(cors);

    let server = ServerBuilder::default()
        .set_http_middleware(middleware)
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
