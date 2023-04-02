#![deny(missing_docs)]
//! This is a simple key-value store

pub use kvstore::KvStore;
pub use error::{KvStoreError, Result};
pub use client::KvClient;
pub use server::KvServer;
pub use common::Request;
pub use kvengine::KvEngine;
pub use kvsled::KvSledStore;

mod client;
mod server;
mod kvstore;
mod kvsled;
mod error;
mod common;
mod kvengine;
