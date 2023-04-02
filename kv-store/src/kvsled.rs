use std::path::PathBuf;

use log::info;
use sled::Db;

use crate::{KvEngine, Result, KvStoreError};

/// a key-value store using sled
pub struct KvSledStore {
    store: Db,
}

impl KvSledStore {
    /// create a new sled db or recover the past instance
    pub fn open(path: impl Into<PathBuf>) -> Result<KvSledStore> {
        let mut path = path.into();
        path.push("sledDB");
        info!("sled path: {}", path.clone().display());
        let store = sled::open(path)?;
        info!("recover config: {}", Db::was_recovered(&store));
        Ok(KvSledStore{store})
    }
}

impl KvEngine for KvSledStore {
    /// set key-value
    fn set(&mut self, key: String, value: String) -> Result<()> {
        info!("sled set {}-{}", key, value);
        self.store.insert(key, value.as_bytes().to_vec())?;
        self.store.flush()?;
        Ok(())
    }
    /// get value
    fn get(&self, key: String) -> Result<Option<String>> {
        let result = self.store.get(&key)?;
        info!("get key: {},  result is success", key);
        match result {
            Some(value) => {
                info!("get some");
                return Ok(Some(String::from_utf8(value.to_vec())?));
            },
            None => {
                info!("get none");
                return Err(KvStoreError::GetNonExistValue);
            }
        }
    }
    /// remove key
    fn remove(&mut self, key: String) -> Result<()> {
        info!("sled remove key: {}", key);
        let res = self.store.remove(key);
        self.store.flush()?;
        match res {
            Ok(None) => {
                return Err(KvStoreError::RemoveNonExistKey);
            }
            Ok(Some(_)) => {
                return Ok(());
            }
            Err(err) => {
                return Err(KvStoreError::RemoveNonExistKey);
            }
        }
    }
}