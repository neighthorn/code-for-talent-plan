use std::{net::{SocketAddr, TcpListener, TcpStream}, io::{Write, BufReader, BufWriter}};
use log::info;
use serde_json::Deserializer;

use crate::{Result, Request, common::Response, KvEngine, thread_pool::ThreadPool};

/// a server used to handle request, contains a kvstore
pub struct KvServer<E: KvEngine, T: ThreadPool> {
    addr: SocketAddr,
    listener: TcpListener,
    kvengine: E,
    thread_pool: T,
}

impl<E: KvEngine, T: ThreadPool> KvServer<E, T> {
    /// construct a new server
    pub fn new(addr: SocketAddr, engine: E, thread_pool: T) -> Result<KvServer<E, T>> {
        let listener = TcpListener::bind(addr)?;
        info!("bind to {}", addr);
        Ok(KvServer { addr, listener, kvengine: engine, thread_pool })
    }

    /// run server to catch connection and handle requests
    pub fn run(&mut self) -> Result<()> {
        for stream in self.listener.incoming() {
            info!("get connenction");
            let mut stream = stream.unwrap();
            let engine = self.kvengine.clone();
            self.thread_pool.spawn(move || { serve_connection(engine, stream); });
        }
        Ok(())
    }
}

/// handle connection
fn serve_connection<E: KvEngine>(engine: E, mut stream: TcpStream) -> Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    info!("get stream");
    let request_iter = Deserializer::from_reader(reader).into_iter::<Request>();
    info!("receive request");
    for request in request_iter {
        let request = request?;
        match request {
            Request::Set { key, value } => {
                let res = engine.set(key, value);
                match res {
                    Ok(_) => {
                        info!("success to set value");
                        let res = Response::Set { result: String::from("Success") };
                        serde_json::to_writer(&mut writer, &res)?;
                        writer.flush()?;
                        info!("finish send response");
                    },
                    Err(_) => {
                        let res = Response::Set { result: String::from("Failure") };
                        serde_json::to_writer(&mut writer, &res)?;
                        writer.flush()?;
                    }
                }
            },
            Request::Get { key } => {
                let res = engine.get(key);
                match res {
                    Ok(value) => {
                        match value {
                            Some(val) => {
                                let res = Response::Get { value: val, result: String::from("Success") };
                                serde_json::to_writer(&mut writer, &res)?;
                                writer.flush()?;
                            },
                            None => {
                                let res = Response::Get { value: String::from("Key not found"), result: String::from("Failure") };
                                serde_json::to_writer(&mut writer, &res)?;
                                writer.flush()?;
                            }
                        }
                    }
                    Err(_) => {
                        let res = Response::Get { value: String::from("Key not found"), result: String::from("Failure") };
                        serde_json::to_writer(&mut writer, &res)?;
                        writer.flush()?;
                    }
                }
            },
            Request::Rm { key } => {
                let res = engine.remove(key);
                match res {
                    Ok(_) => {
                        let res = Response::Rm { result: String::from("Success") };
                        serde_json::to_writer(&mut writer, &res)?;
                        writer.flush()?;
                    },
                    Err(_) => {
                        let res = Response::Rm { result: String::from("Key not found") };
                        serde_json::to_writer(&mut writer, &res)?;
                        writer.flush()?;
                    }
                }
            }
        }
    }
    Ok(())
}