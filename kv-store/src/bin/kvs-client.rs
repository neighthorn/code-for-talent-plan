use std::net::SocketAddr;

use kvs::{KvClient, Result, KvStoreError};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Set {
    key: String,
    value: String,
    #[structopt(long="addr", default_value="127.0.0.1:4000")]
    addr: SocketAddr,
}

#[derive(Debug, StructOpt)]
struct Get {
    key: String,
    #[structopt(long="addr", default_value="127.0.0.1:4000")]
    addr: SocketAddr,
}

#[derive(Debug, StructOpt)]
struct Rm {
    key: String,
    #[structopt(long="addr", default_value="127.0.0.1:4000")]
    addr: SocketAddr,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "get")]
    Get(Get),
    #[structopt(name = "set")]
    Set(Set),
    #[structopt(name = "rm")]
    Rm(Rm),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "kvs-client")]
struct Arguments {
    #[structopt(subcommand)]
    command: Command,
}

fn main() -> Result<()> {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();
    let opt = Arguments::from_args();
    
    match opt.command {
        Command::Get(Get { key, addr }) => {
            let mut client = KvClient::new(addr)?;
            match client.get(key) {
                Ok(val) => {
                    println!("{}", val);
                },
                Err(KvStoreError::StringErr(err)) => {
                    println!("{}", err);
                },
                Err(KvStoreError::GetNonExistValue) => {
                    println!("Key not found");
                }
                Err(err) => { return Err(err); }
            }
        },
        Command::Set(Set { key, value, addr }) => {
            let mut client = KvClient::new(addr)?;
            client.set(key, value)?;
        },
        Command::Rm(Rm { key, addr }) => {
            let mut client = KvClient::new(addr)?;
            client.rm(key)?;
        }
    }

    Ok(())

    // println!("{:?}", opt);
}