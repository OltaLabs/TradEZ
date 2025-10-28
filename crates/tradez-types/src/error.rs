use tezos_smart_rollup::host::RuntimeError;
use tezos_smart_rollup_host::path::PathError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TradezError {
    #[error("Data store error: {0}")]
    DataStoreError(String),
    #[error("Database path error: {0}")]
    DatabasePathError(#[from] PathError),
    #[error("Database runtime error: {0}")]
    DatabaseRuntimeError(#[from] RuntimeError),
}
