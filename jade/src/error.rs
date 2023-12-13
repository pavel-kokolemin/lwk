use std::time::SystemTimeError;

use serde::{Deserialize, Serialize};
use serde_cbor::Value;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Jade Error: {0}")]
    JadeError(ErrorDetails),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("SystemTime Error: {0}")]
    SystemTimeError(SystemTimeError),

    #[cfg(feature = "serial")]
    #[error("Serial Error: {0}")]
    SerialError(#[from] serialport::Error),

    #[error("No available ports")]
    NoAvailablePorts,

    #[error("Jade returned neither an error nor a result")]
    JadeNeitherErrorNorResult,

    #[error(transparent)]
    SerdeCbor(#[from] serde_cbor::Error),

    #[error(transparent)]
    Bip32(#[from] elements::bitcoin::bip32::Error),

    #[error("Mismatching network, jade was initialized with: {init} but the method params received {passed}")]
    MismatchingXpub {
        init: crate::Network,
        passed: crate::Network,
    },

    #[error("Poison error: {0}")]
    PoisonError(String),

    #[error(transparent)]
    Http(#[from] minreq::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error("Http request to {0} returned {1} instead of 200")]
    HttpStatus(String, i32),

    #[error("Jade authentication returned a response without urlA")]
    MissingUrlA,

    #[error("The handshake complete call to the pin server failed")]
    HandshakeFailed,

    #[error("Unexpected \"false\" result")]
    UnexpectedFalse,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorDetails {
    code: i64,
    message: String,
    data: Option<Value>,
}

impl std::fmt::Display for ErrorDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error code: {} - message: {}", self.code, self.message)
    }
}
