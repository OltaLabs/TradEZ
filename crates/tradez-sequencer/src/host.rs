use std::collections::VecDeque;

use redb::{Database, ReadableDatabase, TableDefinition};
use tezos_smart_rollup::{inbox::InboxMessage, michelson::MichelsonUnit};
use tezos_smart_rollup_host::{
    dal_parameters::RollupDalParameters,
    input::Message,
    metadata::RollupMetadata,
    runtime::{Runtime, RuntimeError},
};

const TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("my_data");

pub struct SequencerHost {
    pub inputs: VecDeque<Vec<u8>>,
    pub db: Database,
}

impl SequencerHost {
    pub fn new(inputs: Vec<Vec<u8>>) -> Self {
        let db = Database::create("my_db.redb").unwrap();
        Self {
            db,
            inputs: inputs.into_iter().collect(),
        }
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

    fn write_output(&mut self, _msg: &[u8]) -> Result<(), RuntimeError> {
        unimplemented!()
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
        Ok(table.get(path.to_string().as_str()).unwrap().unwrap().value())
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
            table.insert(path.to_string().as_str(), src.to_vec()).unwrap();
        }
        write_txn.commit().unwrap();
        Ok(())
    }

    fn upgrade_failed(&self) -> Result<bool, RuntimeError> {
        unimplemented!()
    }
}
