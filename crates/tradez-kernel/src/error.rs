use thiserror::Error;

#[derive(Error, Debug)]
pub enum KernelError {
    #[error("Data store read error: {0}")]
    DataStoreReadError(String),
}
