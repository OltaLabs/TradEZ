use std::collections::VecDeque;

use redb::{Database, ReadableDatabase, TableDefinition};
use rlp::Decodable;
use tezos_smart_rollup::{inbox::InboxMessage, michelson::MichelsonUnit};
use tezos_smart_rollup_host::{
    dal_parameters::RollupDalParameters,
    input::Message,
    metadata::RollupMetadata,
    runtime::{Runtime, RuntimeError},
};
use tradez_types::{
    address::Address,
    orderbook::Event,
    position::{Price, Qty, Side},
};

const TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("my_data");
const PATH_HISTORY: &str = "tradez/history/";

pub struct SequencerHost {
    pub inputs: VecDeque<Vec<u8>>,
    pub db: Database,
    pub trade_to_notify: Vec<(Address, Address, u128, Qty, Price, Side)>,
}

impl SequencerHost {
    pub fn new(data_dir: String) -> Self {
        let db = Database::create(format!("{}/my_db.redb", data_dir)).unwrap();
        Self {
            db,
            inputs: VecDeque::new(),
            trade_to_notify: Vec::new(),
        }
    }

    pub fn add_inputs(&mut self, new_inputs: Vec<Vec<u8>>) {
        for input in new_inputs {
            self.inputs.push_back(input);
        }
    }

    pub fn read_history(&self) -> Vec<(u128, Qty, Price, Side)> {
        let mut history = Vec::new();
        let read_txn = self.db.begin_read().unwrap();
        let table = read_txn.open_table(TABLE).unwrap();
        let prefix = PATH_HISTORY;
        let mut iter = table.range(prefix..).unwrap();
        while let Some(Ok((_, value))) = iter.next() {
            let rlp_data = value.value();
            let rlp = rlp::Rlp::new(&rlp_data);
            let timestamp: u128 = rlp.val_at(0).unwrap();
            let price: Price = rlp.val_at(1).unwrap();
            let qty: Qty = rlp.val_at(2).unwrap();
            let side_u8: u8 = rlp.val_at(3).unwrap();
            let side = match side_u8 {
                0 => Side::Bid,
                1 => Side::Ask,
                _ => continue, // Skip invalid entries
            };
            history.push((timestamp, qty, price, side));
        }
        history
    }
}

impl Runtime for SequencerHost {
    fn read_input(&mut self) -> Result<Option<Message>, RuntimeError> {
        self.inputs
            .pop_front()
            .map(|data| {
                let inbox_message = InboxMessage::External::<MichelsonUnit>(&data);
                let mut bytes = Vec::new();
                inbox_message.serialize(&mut bytes).unwrap();
                Ok(Some(Message::new(1, 1, bytes)))
            })
            .unwrap_or(Ok(None))
    }

    fn write_output(&mut self, msg: &[u8]) -> Result<(), RuntimeError> {
        let event = Event::decode(&rlp::Rlp::new(msg)).map_err(|_| RuntimeError::DecodingError)?;
        match event {
            Event::Trade {
                maker_id: _,
                maker_user,
                taker_id: _,
                taker_user,
                price,
                qty,
                origin_side,
            } => {
                let timestamp = chrono::Utc::now().timestamp_millis() as u128;
                println!(
                    "[KERNEL Trade Event] timestamp: {}, price: {}, qty: {}, side: {:?}",
                    timestamp, price, qty, origin_side
                );
                self.trade_to_notify.push((
                    maker_user,
                    taker_user,
                    timestamp,
                    qty,
                    price,
                    origin_side,
                ));
                // Append to history in the db
                let write_txn = self.db.begin_write().unwrap();
                {
                    let mut table = write_txn.open_table(TABLE).unwrap();
                    let path = format!("{}_{}", PATH_HISTORY, timestamp);
                    let mut rlp_stream = rlp::RlpStream::new();
                    rlp_stream
                        .begin_list(4)
                        .append(&timestamp)
                        .append(&price)
                        .append(&qty)
                        .append(&(origin_side as u8));
                    table
                        .insert(path.as_str(), rlp_stream.out().to_vec())
                        .unwrap();
                }
                write_txn.commit().unwrap();
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn write_debug(&self, msg: &str) {
        println!("[KERNEL Debug]: {}", msg);
    }

    fn last_run_aborted(&self) -> Result<bool, RuntimeError> {
        unimplemented!()
    }

    fn mark_for_reboot(&mut self) -> Result<(), RuntimeError> {
        unimplemented!()
    }

    fn reboot_left(&self) -> Result<u32, RuntimeError> {
        unimplemented!()
    }

    fn restart_forced(&self) -> Result<bool, RuntimeError> {
        unimplemented!()
    }

    fn reveal_dal_page(
        &self,
        _published_level: i32,
        _slot_index: u8,
        _page_index: i16,
        _destination: &mut [u8],
    ) -> Result<usize, RuntimeError> {
        unimplemented!()
    }

    fn reveal_dal_parameters(&self) -> RollupDalParameters {
        unimplemented!()
    }

    fn reveal_metadata(&self) -> RollupMetadata {
        unimplemented!()
    }

    fn reveal_preimage(
        &self,
        _hash: &[u8; 33],
        _destination: &mut [u8],
    ) -> Result<usize, RuntimeError> {
        unimplemented!()
    }

    fn runtime_version(&self) -> Result<String, RuntimeError> {
        unimplemented!()
    }

    fn store_copy(
        &mut self,
        _from_path: &impl tezos_smart_rollup_host::path::Path,
        _to_path: &impl tezos_smart_rollup_host::path::Path,
    ) -> Result<(), RuntimeError> {
        unimplemented!()
    }

    fn store_count_subkeys<T: tezos_smart_rollup_host::path::Path>(
        &self,
        _prefix: &T,
    ) -> Result<u64, RuntimeError> {
        unimplemented!()
    }

    fn store_delete<T: tezos_smart_rollup_host::path::Path>(
        &mut self,
        _path: &T,
    ) -> Result<(), RuntimeError> {
        unimplemented!()
    }

    fn store_delete_value<T: tezos_smart_rollup_host::path::Path>(
        &mut self,
        _path: &T,
    ) -> Result<(), RuntimeError> {
        unimplemented!()
    }

    fn store_has<T: tezos_smart_rollup_host::path::Path>(
        &self,
        _path: &T,
    ) -> Result<Option<tezos_smart_rollup_host::runtime::ValueType>, RuntimeError> {
        unimplemented!()
    }

    fn store_move(
        &mut self,
        _from_path: &impl tezos_smart_rollup_host::path::Path,
        _to_path: &impl tezos_smart_rollup_host::path::Path,
    ) -> Result<(), RuntimeError> {
        unimplemented!()
    }

    fn store_read<T: tezos_smart_rollup_host::path::Path>(
        &self,
        _path: &T,
        _from_offset: usize,
        _max_bytes: usize,
    ) -> Result<Vec<u8>, RuntimeError> {
        unimplemented!()
    }

    fn store_read_all(
        &self,
        path: &impl tezos_smart_rollup_host::path::Path,
    ) -> Result<Vec<u8>, RuntimeError> {
        let read_txn = self.db.begin_read().unwrap();
        let Ok(table) = read_txn.open_table(TABLE) else {
            return Err(RuntimeError::PathNotFound);
        };
        Ok(table
            .get(path.to_string().as_str())
            .unwrap()
            .ok_or(RuntimeError::PathNotFound)?
            .value())
    }

    fn store_read_slice<T: tezos_smart_rollup_host::path::Path>(
        &self,
        _path: &T,
        _from_offset: usize,
        _buffer: &mut [u8],
    ) -> Result<usize, RuntimeError> {
        unimplemented!()
    }

    fn store_value_size(
        &self,
        _path: &impl tezos_smart_rollup_host::path::Path,
    ) -> Result<usize, RuntimeError> {
        unimplemented!()
    }

    fn store_write<T: tezos_smart_rollup_host::path::Path>(
        &mut self,
        _path: &T,
        _src: &[u8],
        _at_offset: usize,
    ) -> Result<(), RuntimeError> {
        unimplemented!()
    }

    fn store_write_all<T: tezos_smart_rollup_host::path::Path>(
        &mut self,
        path: &T,
        src: &[u8],
    ) -> Result<(), RuntimeError> {
        let write_txn = self.db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(TABLE).unwrap();
            table
                .insert(path.to_string().as_str(), src.to_vec())
                .unwrap();
        }
        write_txn.commit().unwrap();
        Ok(())
    }

    fn upgrade_failed(&self) -> Result<bool, RuntimeError> {
        unimplemented!()
    }
}
