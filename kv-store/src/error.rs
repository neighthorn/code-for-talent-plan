use failure::Fail;
use std::{io, string::FromUtf8Error};

/// Error type for KvStore
#[derive(Fail, Debug)]
pub enum KvStoreError {
    /// try to get the value of a non-existent key
    #[fail(display = "get value for the non-existent key")]
    GetNonExistValue,
    /// try to remove a non-existent key
    #[fail(display = "remove non-existent key")]
    RemoveNonExistKey,
    /// fail to serialize a command to the log file
    #[fail(display = "fail to serialize a command to the log file")]
    SerializeCmdError,
    /// fail to rebuild the in-memory index
    #[fail(display = "fail to rebuild the in-memory index")]
    RebuildIndexError,
    /// wrong engine, try to use different engine than selected originally
    #[fail(display = "wrong engine")]
    WrongEngineError,
    /// io error
    #[fail(display = "io error: {}", _0)]
    Io(#[cause] io::Error),
    /// serde error
    #[fail(display = "serde error: {}", _0)]
    Serde(#[cause] serde_json::Error),
    /// string error
    #[fail(display = "{}", _0)]
    StringErr(String),
    /// sled error
    #[fail(display = "{}", _0)]
    Sled(#[cause] sled::Error),
    /// utf8 error
    #[fail(display = "{}", _0)]
    Utf8Error(#[cause] FromUtf8Error),
}

impl From<io::Error> for KvStoreError {
    fn from(err: io::Error) -> KvStoreError {
        KvStoreError::Io(err)
    }
}

impl From<serde_json::Error> for KvStoreError {
    fn from(err: serde_json::Error) -> KvStoreError {
        KvStoreError::Serde(err)
    }
}

impl From<sled::Error> for KvStoreError {
    fn from(err: sled::Error) -> KvStoreError {
        KvStoreError::Sled(err)
    }
}

impl From<FromUtf8Error> for KvStoreError {
    fn from(err: FromUtf8Error) -> KvStoreError {
        KvStoreError::Utf8Error(err)
    }
}

/// Result type for KvStore
pub type Result<T> = std::result::Result<T, KvStoreError>;
// use std::error::Error;

// #[derive(Debug)]
// pub struct KvStoreError {
//     err: GetNonExistValue,
// }

// impl std::fmt::Display for KvStoreError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "KvStoreError")
//     }
// }

// impl Error for KvStoreError {
//     fn source(&self) -> Option<&(dyn Error + 'static)> {
//         Some(&self.err)
//     }
// }

// #[derive(Debug)]
// struct GetNonExistValue;

// impl std::fmt::Display for GetNonExistValue {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "try to get the value of a non-exist key.")
//     }
// }

// impl Error for GetNonExistValue {}

// pub type Result<T> = std::result::Result<T, KvStoreError>;