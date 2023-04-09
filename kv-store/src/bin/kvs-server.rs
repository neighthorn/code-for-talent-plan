use std::{net::SocketAddr, env::current_dir};
use clap::arg_enum;
use log::info;
use structopt::StructOpt;
use std::fs;

use kvs::{Result, KvStoreError, KvServer, KvStore, KvEngine, thread_pool::SharedQueueThreadPool, thread_pool::ThreadPool};

const DEFAULT_ENGINE: Engine = Engine::kvs;
const ENGINE_META_PATH: &str = "engine.meta";

/// engine type, kvs or sled
arg_enum! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum Engine {
        kvs,
        sled,
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "kvs-server")]
struct Arguments {
    #[structopt(long, default_value="127.0.0.1:4000")]
    addr: SocketAddr,
    #[structopt(long)]
    engine: Option<Engine>,
}

/// load_engine_meta is used to read engine type from the persistent file
/// if the engine type is not specified, use DEFAULT_ENGINE and write it to disk
fn load_engine_meta(engine_arg: Option<Engine>) -> Result<Engine> {
    
    // persist the engine type into a file
    let mut path = current_dir()?;
    path.push(ENGINE_META_PATH);

    if path.clone().as_path().exists() {
        // .parse()实际上是需要实现FromStr trait的，但是由于arg_enum!宏自动实现了FromStr trait，因此这里不需要单独实现
        match fs::read_to_string(path)?.parse() {
            Ok(engine) => {
                // if the args contain --engine, check ENGINE-NAME == engine(meta data)
                if let Some(engine_selected) = engine_arg {
                    if engine_selected != engine {
                        return Err(KvStoreError::WrongEngineError);
                    }    
                }
                // if the args do not contain --eigine, or ENGINE-NAME == engine, return Ok(engine)
                Ok(engine)
            },
            Err(e) => {
                Err(KvStoreError::StringErr(e))
            },
        }
    } else {
        // if args do not conatain --engine, use DEFAULT_ENGINE, otherwise ENGINE-NAME
        let engine = match engine_arg {
            Some(engine_selected) => { engine_selected },
            None => {DEFAULT_ENGINE},
        };
        // persist engine meta to disk
        fs::write(path, engine.to_string())?;
        Ok(engine)
    }
}

fn main() -> Result<()> {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();
    let opt = Arguments::from_args();
    let engine_type = load_engine_meta(opt.engine)?;

    // print server info
    info!("kvs-server version: {}", env!("CARGO_PKG_VERSION"));
    info!("engine: {}", engine_type);
    info!("ip addr: {}", opt.addr);

    let path = current_dir()?;
    match engine_type {
        Engine::kvs => {
            run_server(KvStore::open(path)?, opt.addr)?;
        },
        Engine::sled => {
            run_server(KvStore::open(path)?, opt.addr)?;
        },
    }
    Ok(())
}

fn run_server<E: KvEngine>(engine: E, addr: SocketAddr) -> Result<()> {
    info!("run server");
    let thread_pool = SharedQueueThreadPool::new(4)?;
    let mut server = KvServer::new(addr, engine, thread_pool)?;
    server.run()?;
    Ok(())
}