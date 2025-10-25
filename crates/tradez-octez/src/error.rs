use thiserror::Error;

#[derive(Debug, Error)]
pub enum OctezError {
    #[error("HTTP request failed: {0}")]
    HttpRequestError(#[from] reqwest::Error),
    #[error("HTTP response error: {0}")]
    HttpResponseError(String),
    #[error("Hex decoding error: {0}")]
    HexDecodingError(#[from] hex::FromHexError),
}
