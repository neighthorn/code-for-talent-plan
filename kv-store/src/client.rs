use std::{net::{SocketAddr, TcpStream}, io::{Write, Read, BufWriter, BufReader}};

use log::info;
use serde::Deserialize;
use serde_json::Deserializer;

use crate::{Result, Request, common::Response, KvStoreError};

/// used to establish a connection to server and send request
pub struct KvClient {
    addr: SocketAddr,
    writer: BufWriter<TcpStream>,
    reader: BufReader<TcpStream>,
}

impl KvClient {
    /// construct a new client
    pub fn new(addr: SocketAddr) -> Result<KvClient> {
        let mut stream = TcpStream::connect(addr)?;
        let writer = BufWriter::new(stream.try_clone()?);
        let reader = BufReader::new(stream);
        info!("connected");
        Ok(KvClient { addr, writer, reader })
    }

    /// send set command to server
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        info!("set {} {}", key, value);
        let command = Request::Set { key, value };
        serde_json::to_writer(&mut self.writer, &command)?;
        self.writer.flush()?;
        // self.stream.write_all(request.as_bytes())?;
        info!("send request");
        let response = Response::deserialize(&mut Deserializer::from_reader(&mut self.reader))?;
        info!("receive response");
        if let Response::Set { result } = response {
            if !result.eq("Success") {
                return Err(KvStoreError::StringErr(result));
            }
        }
        Ok(())
    }

    /// send get command to server and get the result
    pub fn get(&mut self, key: String) -> Result<String> {
        let command = Request::Get { key };
        serde_json::to_writer(&mut self.writer, &command)?;
        self.writer.flush()?;
        let response = Response::deserialize(&mut Deserializer::from_reader(&mut self.reader))?;
        if let Response::Get { value, result } = response {
            if result.eq("Success") {
                return Ok(value);
            } else {
                return Err(KvStoreError::StringErr(value));
            }
        }
        Err(KvStoreError::StringErr(String::from("Key not found")))
    }

    /// send rm command to server
    pub fn rm(&mut self, key: String) -> Result<()> {
        let command = Request::Rm { key };
        serde_json::to_writer(&mut self.writer, &command)?;
        self.writer.flush()?;
        let response = Response::deserialize(&mut Deserializer::from_reader(&mut self.reader))?;
        if let Response::Rm { result } = response {
            if !result.eq("Success") {
                return Err(KvStoreError::StringErr(result));
            }
        }
        Ok(())
    }
}

