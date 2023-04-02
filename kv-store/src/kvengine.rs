use crate::Result;

/// a trait for kvengines, kvstore and sled have to impl this trait
pub trait KvEngine {
    /// set key-value
    fn set(&mut self, key: String, value: String) -> Result<()>;
    /// get value
    fn get(&self, key: String) -> Result<Option<String>>;
    /// remove key
    fn remove(&mut self, key: String) -> Result<()>;
}