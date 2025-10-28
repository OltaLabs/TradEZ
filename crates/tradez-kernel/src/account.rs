use std::collections::HashMap;

use rlp::{Decodable, Encodable};
use tezos_smart_rollup::host::{Runtime, RuntimeError};
use tezos_smart_rollup_host::path::{RefPath, concat};
use tradez_types::{address::Address, currencies::Currencies, error::TradezError, position::UserOrder};

#[derive(Debug)]
pub struct Account {
    pub address: Address,
    pub nonce: u64,
    pub balances: HashMap<Currencies, u64>,
    // TODO: Optimize, currently it's stored at two places
    pub orders: HashMap<u64, UserOrder>
}

impl Encodable for Account {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_list(4);
        s.append(&self.address);
        s.append(&self.nonce);
        s.begin_list(self.balances.len());
        for (currency, balance) in &self.balances {
            s.begin_list(2);
            match currency {
                Currencies::USDC => s.append(&0u8),
                Currencies::XTZ => s.append(&1u8),
            };
            s.append(balance);
        }
        s.begin_list(self.orders.len());
        for (order_id, order) in &self.orders {
            s.begin_list(2);
            s.append(order_id);
            s.append(order);
        }
    }
}

impl Decodable for Account {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let address: Address = rlp.val_at(0)?;
        let nonce: u64 = rlp.val_at(1)?;
        let balances_rlp = rlp.at(2)?;
        let mut balances = HashMap::new();
        for i in 0..balances_rlp.item_count()? {
            let entry_rlp = balances_rlp.at(i)?;
            let currency_value: u8 = entry_rlp.val_at(0)?;
            let balance: u64 = entry_rlp.val_at(1)?;
            let currency = match currency_value {
                0 => Currencies::USDC,
                1 => Currencies::XTZ,
                _ => return Err(rlp::DecoderError::Custom("Invalid currency value")),
            };
            balances.insert(currency, balance);
        }
        let orders_rlp = rlp.at(3)?;
        let mut orders = HashMap::new();
        for i in 0..orders_rlp.item_count()? {
            let entry_rlp = orders_rlp.at(i)?;
            let order_id: u64 = entry_rlp.val_at(0)?;
            let order: UserOrder = entry_rlp.val_at(1)?;
            orders.insert(order_id, order);
        }
        Ok(Account {
            address,
            nonce,
            balances,
            orders,
        })
    }
}

pub const ACCOUNT_KEY_PREFIX: RefPath = RefPath::assert_from(b"/accounts");

impl Account {
    pub fn new(address: Address) -> Self {
        Account {
            address,
            nonce: 0,
            balances: HashMap::new(),
            orders: HashMap::new(),
        }
    }

    pub fn load(
        host: &mut impl Runtime,
        address: &Address,
    ) -> Result<Option<Account>, TradezError> {
        let address = format!("/{:x}", address.0);
        let key = concat(
            &ACCOUNT_KEY_PREFIX,
            &RefPath::assert_from(address.as_bytes()),
        )?;
        match host.store_read_all(&key) {
            Ok(data) => {
                let rlp = rlp::Rlp::new(&data);
                let account = Account::decode(&rlp)
                    .map_err(|e| TradezError::DataStoreError(e.to_string()))?;
                Ok(Some(account))
            }
            Err(RuntimeError::PathNotFound) => Ok(None),
            Err(e) => Err(TradezError::DatabaseRuntimeError(e)),
        }
    }

    pub fn save(&self, host: &mut impl Runtime) -> Result<(), TradezError> {
        let address = format!("/{:x}", self.address.0);
        let key = concat(
            &ACCOUNT_KEY_PREFIX,
            &RefPath::assert_from(address.as_bytes()),
        )?;
        let mut stream = rlp::RlpStream::new();
        self.rlp_append(&mut stream);
        let data = stream.out();
        host.store_write_all(&key, &data)
            .map_err(TradezError::DatabaseRuntimeError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_account_rlp() {
        let mut balances = HashMap::new();
        balances.insert(Currencies::USDC, 1000u64);
        balances.insert(Currencies::XTZ, 500u64);
        let orders = HashMap::new();
        let address = Address::from([0u8; 20]);
        let account = Account {
            address: address.clone(),
            nonce: 100,
            balances: balances.clone(),
            orders: orders.clone(),
        };
        let mut stream = rlp::RlpStream::new();
        account.rlp_append(&mut stream);
        let out = stream.out();
        let rlp = rlp::Rlp::new(&out);
        let decoded_account = Account::decode(&rlp).unwrap();
        assert_eq!(decoded_account.address, address);
        assert_eq!(decoded_account.balances, balances);
    }
}
