use serde::{Serialize, Deserialize};

/// the request send to server
#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    /// set request
    Set {
        /// key
        key: String, 
        /// value
        value: String
    },
    /// get request
    Get {
        /// key
        key: String
    },
    /// remove request
    Rm {
        /// key
        key: String
    },
}

/// the response from server
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Set {result: String},
    Get {value: String, result: String},
    Rm {result: String},
}