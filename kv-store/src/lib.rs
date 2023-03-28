#![deny(missing_docs)]
//! This is a simple key-value store

pub use kvstore::KvStore;
pub use error::{KvStoreError, Result};

mod kvstore;
mod error;
