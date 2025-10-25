use std::collections::{BTreeMap, VecDeque};

use crate::{
    address::Address,
    error::TradezError,
    position::{OrdType, Order, Price, Qty, Side, Ts},
};
use rlp::{Decodable, Encodable};
use tezos_smart_rollup::host::{Runtime, RuntimeError};
use tezos_smart_rollup_host::path::RefPath;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Placed {
        id: u64,
        side: Side,
        price: Price,
        qty: Qty,
    },
    Trade {
        maker_id: u64,
        taker_id: u64,
        price: Price,
        qty: Qty,
    },
    Done {
        id: u64,
    }, // ordre entièrement exécuté
    Cancelled {
        id: u64,
        reason: &'static str,
    },
}

pub type SideLadder = BTreeMap<Price, VecDeque<Order>>;

#[derive(Default, Debug)]
pub struct OrderBook {
    // asks : prix croissant ; bids : prix croissant (on itère à rebours pour best bid)
    asks: SideLadder,
    bids: SideLadder,
    next_id: u64,
}

impl Encodable for OrderBook {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_list(3);
        // Asks
        s.begin_list(self.asks.len());
        for (price, queue) in &self.asks {
            s.begin_list(2);
            s.append(price);
            s.begin_list(queue.len());
            for order in queue {
                s.append(order);
            }
        }
        // Bids
        s.begin_list(self.bids.len());
        for (price, queue) in &self.bids {
            s.begin_list(2);
            s.append(price);
            s.begin_list(queue.len());
            for order in queue {
                s.append(order);
            }
        }
        s.append(&self.next_id);
    }
}

impl Decodable for OrderBook {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let mut ob = OrderBook::default();
        let mut it = rlp.iter();
        let asks_rlp = it.next().ok_or(rlp::DecoderError::RlpIncorrectListLen)?;
        for ask in asks_rlp.iter() {
            let mut ask_it = ask.iter();
            let price: Price = ask_it
                .next()
                .ok_or(rlp::DecoderError::RlpIncorrectListLen)?
                .as_val()?;
            let orders_rlp = ask_it
                .next()
                .ok_or(rlp::DecoderError::RlpIncorrectListLen)?;
            let mut queue = VecDeque::new();
            for order_rlp in orders_rlp.iter() {
                let order: Order = Decodable::decode(&order_rlp)?;
                queue.push_back(order);
            }
            ob.asks.insert(price, queue);
        }

        let bids_rlp = it.next().ok_or(rlp::DecoderError::RlpIncorrectListLen)?;
        for bid in bids_rlp.iter() {
            let mut bid_it = bid.iter();
            let price: Price = bid_it
                .next()
                .ok_or(rlp::DecoderError::RlpIncorrectListLen)?
                .as_val()?;
            let orders_rlp = bid_it
                .next()
                .ok_or(rlp::DecoderError::RlpIncorrectListLen)?;
            let mut queue = VecDeque::new();
            for order_rlp in orders_rlp.iter() {
                let order: Order = Decodable::decode(&order_rlp)?;
                queue.push_back(order);
            }
            ob.bids.insert(price, queue);
        }
        ob.next_id = it
            .next()
            .ok_or(rlp::DecoderError::RlpIncorrectListLen)?
            .as_val()?;

        Ok(ob)
    }
}

pub const ORDER_BOOK_STR_PATH: &str = "/tradez/order_book";
pub const ORDER_BOOK_PATH: RefPath = RefPath::assert_from(b"/tradez/order_book");

impl OrderBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load<Host: Runtime>(host: &mut Host) -> Result<Self, TradezError> {
        match host.store_read_all(&ORDER_BOOK_PATH) {
            Ok(data) => {
                let rlp = rlp::Rlp::new(&data);
                OrderBook::decode(&rlp).map_err(|e| TradezError::DataStoreError(e.to_string()))
            }
            Err(RuntimeError::PathNotFound) => Ok(OrderBook::new()),
            Err(e) => Err(TradezError::DataStoreError(e.to_string())),
        }
    }

    pub fn save<Host: Runtime>(&self, host: &mut Host) -> Result<(), TradezError> {
        let mut stream = rlp::RlpStream::new();
        self.rlp_append(&mut stream);
        let data = stream.out().to_vec();
        host.store_write_all(&ORDER_BOOK_PATH, &data)
            .map_err(|e| TradezError::DataStoreError(e.to_string()))
    }

    pub fn best_bid(&self) -> Option<Price> {
        self.bids.keys().next_back().copied()
    }
    pub fn best_ask(&self) -> Option<Price> {
        self.asks.keys().next().copied()
    }
    pub fn spread(&self) -> Option<Price> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask.saturating_sub(bid)),
            _ => None,
        }
    }
    pub fn price_quantity_at(&self, price: Price) -> Qty {
        let bid_qty = self
            .bids
            .get(&price)
            .map(|queue| queue.iter().map(|o| o.remaining).sum())
            .unwrap_or(0);
        let ask_qty = self
            .asks
            .get(&price)
            .map(|queue| queue.iter().map(|o| o.remaining).sum())
            .unwrap_or(0);
        bid_qty + ask_qty
    }
    pub fn is_empty(&self) -> bool {
        self.bids.is_empty() && self.asks.is_empty()
    }

    /// Place un LIMIT. Retourne l'id de l'ordre. Les événements sont poussés dans `out`.
    pub fn place_limit(
        &mut self,
        user: Address,
        side: Side,
        price: Price,
        qty: Qty,
        ts: Ts,
        out: &mut Vec<Event>,
    ) -> u64 {
        assert!(qty > 0, "qty must be > 0");
        if side == Side::Bid {
            assert!(price > 0, "bid price must be > 0");
        }
        let id = self.alloc_id();
        let mut taker = Order {
            id,
            user,
            side,
            ord_type: OrdType::Limit,
            price,
            qty,
            remaining: qty,
            ts,
        };
        out.push(Event::Placed {
            id,
            side,
            price,
            qty,
        });

        self.match_incoming(&mut taker, out);

        if taker.remaining > 0 {
            let book = match side {
                Side::Bid => &mut self.bids,
                Side::Ask => &mut self.asks,
            };
            book.entry(price).or_default().push_back(taker);
        } else {
            out.push(Event::Done { id });
        }
        id
    }

    /// Place un MARKET. Retourne l'id de l'ordre. Le reliquat non exécuté est annulé.
    pub fn place_market(
        &mut self,
        user: Address,
        side: Side,
        qty: Qty,
        ts: Ts,
        out: &mut Vec<Event>,
    ) -> u64 {
        assert!(qty > 0, "qty must be > 0");
        let id = self.alloc_id();
        let mut taker = Order {
            id,
            user,
            side,
            ord_type: OrdType::Market,
            price: 0,
            qty,
            remaining: qty,
            ts,
        };
        out.push(Event::Placed {
            id,
            side,
            price: 0,
            qty,
        });

        self.match_incoming(&mut taker, out);

        if taker.remaining > 0 {
            out.push(Event::Cancelled {
                id,
                reason: "unfilled_market",
            });
        } else {
            out.push(Event::Done { id });
        }
        id
    }

    /// Annule un ordre par id sur un côté donné. Retourne true si trouvé.
    pub fn cancel(&mut self, side: Side, id: u64, out: &mut Vec<Event>) -> bool {
        let ladder = match side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks,
        };
        let mut removed = false;
        let mut to_prune: Option<Price> = None;

        for (price, queue) in ladder.iter_mut() {
            if let Some(pos) = queue.iter().position(|o| o.id == id) {
                queue.remove(pos);
                removed = true;
                if queue.is_empty() {
                    to_prune = Some(*price);
                }
                break;
            }
        }
        if let Some(p) = to_prune {
            ladder.remove(&p);
        }
        if removed {
            out.push(Event::Cancelled {
                id,
                reason: "by_user",
            });
        }
        removed
    }

    /* === Matching interne (déterministe, FIFO intra-niveau) ============== */

    fn match_incoming(&mut self, taker: &mut Order, out: &mut Vec<Event>) {
        match taker.side {
            Side::Bid => self.consume_asks(taker, out),
            Side::Ask => self.consume_bids(taker, out),
        }
    }

    fn consume_asks(&mut self, taker: &mut Order, out: &mut Vec<Event>) {
        loop {
            if taker.remaining == 0 {
                break;
            }
            let best_ask_price = match self.asks.keys().next().copied() {
                Some(p) => p,
                None => break,
            };
            // LIMIT bid doit croiser : bid_price >= best_ask
            if matches!(taker.ord_type, OrdType::Limit) && taker.price < best_ask_price {
                break;
            }

            let mut queue = self.asks.remove(&best_ask_price).expect("exists");
            while taker.remaining > 0 {
                let Some(mut maker) = queue.pop_front() else {
                    break;
                };
                let exec_qty = taker.remaining.min(maker.remaining);
                taker.remaining -= exec_qty;
                maker.remaining -= exec_qty;
                out.push(Event::Trade {
                    maker_id: maker.id,
                    taker_id: taker.id,
                    price: best_ask_price,
                    qty: exec_qty,
                });

                if maker.remaining > 0 {
                    queue.push_front(maker); // FIFO conservé
                    break;
                } else {
                    out.push(Event::Done { id: maker.id });
                }
            }
            if !queue.is_empty() {
                self.asks.insert(best_ask_price, queue);
            }
        }
    }

    fn consume_bids(&mut self, taker: &mut Order, out: &mut Vec<Event>) {
        loop {
            if taker.remaining == 0 {
                break;
            }
            let best_bid_price = match self.bids.keys().next_back().copied() {
                Some(p) => p,
                None => break,
            };
            // LIMIT ask doit croiser : ask_price <= best_bid
            if matches!(taker.ord_type, OrdType::Limit) && taker.price > best_bid_price {
                break;
            }

            let mut queue = self.bids.remove(&best_bid_price).expect("exists");
            while taker.remaining > 0 {
                let Some(mut maker) = queue.pop_front() else {
                    break;
                };
                let exec_qty = taker.remaining.min(maker.remaining);
                taker.remaining -= exec_qty;
                maker.remaining -= exec_qty;
                out.push(Event::Trade {
                    maker_id: maker.id,
                    taker_id: taker.id,
                    price: best_bid_price,
                    qty: exec_qty,
                });

                if maker.remaining > 0 {
                    queue.push_front(maker);
                    break;
                } else {
                    out.push(Event::Done { id: maker.id });
                }
            }
            if !queue.is_empty() {
                self.bids.insert(best_bid_price, queue);
            }
        }
    }

    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

/* === Démo & tests ======================================================= */

#[cfg(test)]
mod tests {
    use super::*;

    fn uid(n: u8) -> Address {
        let mut a = [0u8; 20];
        a[0] = n;
        Address::from(a)
    }

    fn compare_orderbooks(ob1: &OrderBook, ob2: &OrderBook) {
        assert_eq!(ob1.next_id, ob2.next_id);
        assert_eq!(ob1.asks.len(), ob2.asks.len());
        for (price, queue1) in &ob1.asks {
            let queue2 = ob2.asks.get(price).expect("ask price level exists");
            assert_eq!(queue1.len(), queue2.len());
            for (o1, o2) in queue1.iter().zip(queue2.iter()) {
                assert_eq!(o1, o2);
            }
        }
        assert_eq!(ob1.bids.len(), ob2.bids.len());
        for (price, queue1) in &ob1.bids {
            let queue2 = ob2.bids.get(price).expect("bid price level exists");
            assert_eq!(queue1.len(), queue2.len());
            for (o1, o2) in queue1.iter().zip(queue2.iter()) {
                assert_eq!(o1, o2);
            }
        }
    }

    #[test]
    fn limit_then_market_flow() {
        let mut ob = OrderBook::new();
        let mut ev = vec![];

        // Place deux asks : 3.50 et 3.60 (1 XTZ chacune)
        let _a1 = ob.place_limit(uid(1), Side::Ask, 3_500_000, 1_000_000, 1, &mut ev);
        let _a2 = ob.place_limit(uid(2), Side::Ask, 3_600_000, 1_000_000, 2, &mut ev);

        assert_eq!(ob.best_ask(), Some(3_500_000));
        assert!(ob.best_bid().is_none());

        // Market BID 1.5 XTZ → consomme 1.0 @3.50 puis 0.5 @3.60
        let _m = ob.place_market(uid(9), Side::Bid, 1_500_000, 3, &mut ev);

        // Il doit rester 0.5 XTZ à 3.60 en ask
        assert_eq!(ob.best_ask(), Some(3_600_000));

        // Cancel de l'ask restant
        // (dans une vraie intégration, on garderait l'id ; ici on le retrouve depuis le niveau)
        let mut cancelled = false;
        if let Some(q) = ob.asks.get(&3_600_000) {
            let id = q.front().unwrap().id;
            cancelled = ob.cancel(Side::Ask, id, &mut ev);
        }
        assert!(cancelled);

        // Le book doit maintenant être vide
        assert!(ob.best_ask().is_none());
        assert!(ob.is_empty() || ob.best_bid().is_none());
    }

    #[test]
    fn limit_crossing_immediate_exec() {
        let mut ob = OrderBook::new();
        let mut ev = vec![];

        // Best bid à 3.40
        let _b1 = ob.place_limit(uid(1), Side::Bid, 3_400_000, 1_000_000, 1, &mut ev);
        // On poste un ask limit "crossé" à 3.30 → doit s'exécuter immédiatement à 3.40 (maker price)
        let _a = ob.place_limit(uid(2), Side::Ask, 3_300_000, 700_000, 2, &mut ev);

        // Best bid a diminué à 3.40 avec 0.3 XTZ restant
        assert_eq!(ob.best_bid(), Some(3_400_000));
        let q = ob.bids.get(&3_400_000).unwrap();
        assert_eq!(q.front().unwrap().remaining, 300_000);

        let bytes = {
            let mut stream = rlp::RlpStream::new();
            ob.rlp_append(&mut stream);
            stream.out().to_vec()
        };
        let ob2 = OrderBook::decode(&rlp::Rlp::new(&bytes)).unwrap();
        compare_orderbooks(&ob, &ob2);
    }
}
