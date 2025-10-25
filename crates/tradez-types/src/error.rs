use thiserror::Error;

#[derive(Debug, Error)]
pub enum TradezError {
    #[error("Data store error: {0}")]
    DataStoreError(String),
}
