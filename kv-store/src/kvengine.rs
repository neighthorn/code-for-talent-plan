use crate::Result;

/// a trait for kvengines, kvstore and sled have to impl this trait
pub trait KvEngine: Clone + Send + 'static {
    /// set key-value
    fn set(&self, key: String, value: String) -> Result<()>;
    /// get value
    fn get(&self, key: String) -> Result<Option<String>>;
    /// remove key
    fn remove(&self, key: String) -> Result<()>;
}