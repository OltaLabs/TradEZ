use std::collections::{BTreeMap, VecDeque};

use alloy_primitives::Address;
use tradez_types::position::{OrdType, Order, Price, Qty, Side, Ts};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Placed { id: u64, side: Side, price: Price, qty: Qty },
    Trade  { maker_id: u64, taker_id: u64, price: Price, qty: Qty },
    Done   { id: u64 },                       // ordre entièrement exécuté
    Cancelled { id: u64, reason: &'static str },
}

#[derive(Default)]
pub struct OrderBook {
    // asks : prix croissant ; bids : prix croissant (on itère à rebours pour best bid)
    asks: BTreeMap<Price, VecDeque<Order>>,
    bids: BTreeMap<Price, VecDeque<Order>>,
    next_id: u64,
}

impl OrderBook {
    pub fn new() -> Self { Self::default() }

    pub fn best_bid(&self) -> Option<Price> { self.bids.keys().next_back().copied() }
    pub fn best_ask(&self) -> Option<Price> { self.asks.keys().next().copied() }
    pub fn is_empty(&self) -> bool { self.bids.is_empty() && self.asks.is_empty() }

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
        if side == Side::Bid { assert!(price > 0, "bid price must be > 0"); }
        let id = self.alloc_id();
        let mut taker = Order { id, user, side, ord_type: OrdType::Limit, price, qty, remaining: qty, ts };
        out.push(Event::Placed { id, side, price, qty });

        self.match_incoming(&mut taker, out);

        if taker.remaining > 0 {
            let book = match side { Side::Bid => &mut self.bids, Side::Ask => &mut self.asks };
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
        let mut taker = Order { id, user, side, ord_type: OrdType::Market, price: 0, qty, remaining: qty, ts };
        out.push(Event::Placed { id, side, price: 0, qty });

        self.match_incoming(&mut taker, out);

        if taker.remaining > 0 {
            out.push(Event::Cancelled { id, reason: "unfilled_market" });
        } else {
            out.push(Event::Done { id });
        }
        id
    }

    /// Annule un ordre par id sur un côté donné. Retourne true si trouvé.
    pub fn cancel(&mut self, side: Side, id: u64, out: &mut Vec<Event>) -> bool {
        let ladder = match side { Side::Bid => &mut self.bids, Side::Ask => &mut self.asks };
        let mut removed = false;
        let mut to_prune: Option<Price> = None;

        for (price, queue) in ladder.iter_mut() {
            if let Some(pos) = queue.iter().position(|o| o.id == id) {
                queue.remove(pos);
                removed = true;
                if queue.is_empty() { to_prune = Some(*price); }
                break;
            }
        }
        if let Some(p) = to_prune { ladder.remove(&p); }
        if removed { out.push(Event::Cancelled { id, reason: "by_user" }); }
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
            if taker.remaining == 0 { break; }
            let best_ask_price = match self.asks.keys().next().copied() { Some(p) => p, None => break };
            // LIMIT bid doit croiser : bid_price >= best_ask
            if matches!(taker.ord_type, OrdType::Limit) && taker.price < best_ask_price { break; }

            let mut queue = self.asks.remove(&best_ask_price).expect("exists");
            while taker.remaining > 0 {
                let Some(mut maker) = queue.pop_front() else { break; };
                let exec_qty = taker.remaining.min(maker.remaining);
                taker.remaining -= exec_qty;
                maker.remaining -= exec_qty;
                out.push(Event::Trade { maker_id: maker.id, taker_id: taker.id, price: best_ask_price, qty: exec_qty });

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
            if taker.remaining == 0 { break; }
            let best_bid_price = match self.bids.keys().next_back().copied() { Some(p) => p, None => break };
            // LIMIT ask doit croiser : ask_price <= best_bid
            if matches!(taker.ord_type, OrdType::Limit) && taker.price > best_bid_price { break; }

            let mut queue = self.bids.remove(&best_bid_price).expect("exists");
            while taker.remaining > 0 {
                let Some(mut maker) = queue.pop_front() else { break; };
                let exec_qty = taker.remaining.min(maker.remaining);
                taker.remaining -= exec_qty;
                maker.remaining -= exec_qty;
                out.push(Event::Trade { maker_id: maker.id, taker_id: taker.id, price: best_bid_price, qty: exec_qty });

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

    fn alloc_id(&mut self) -> u64 { let id = self.next_id; self.next_id += 1; id }
}

/* === Démo & tests ======================================================= */

#[cfg(test)]
mod tests {
    use super::*;

    fn uid(n: u8) -> Address { let mut a=[0u8;20]; a[0]=n; Address::from(a) }

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
    }
}