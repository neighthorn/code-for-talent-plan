use clap::{App, Arg, SubCommand};
use kvs::{KvStore, KvStoreError, Result};
use std::env::{current_dir, args};
use std::process::exit;
// use kvs::KvStore;

fn main() -> Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("-V").help("print the version of package"))
        .subcommand(
            SubCommand::with_name("get")
                .about("get value for the given key")
                .arg(Arg::with_name("KEY").help("A string key").required(true)),
        )
        .subcommand(
            SubCommand::with_name("set")
                .about("set value for the given key")
                .args(&[
                    Arg::with_name("KEY").help("A string key").required(true),
                    Arg::with_name("VALUE")
                        .help("A string value")
                        .required(true),
                ]),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("remove the key-value pair for the given key")
                .arg(Arg::with_name("KEY").help("A string key").required(true)),
        )
        .get_matches();

    match matches.subcommand() {
        ("set", Some(args)) => {
            let key = args.value_of("KEY").unwrap();
            let value = args.value_of("VALUE").unwrap();
            let mut store = KvStore::open(current_dir()?)?;
            store.set(key.to_string(), value.to_string())?;
        }
        ("get", Some(args)) => {
            let key = args.value_of("KEY").unwrap();
            let mut store = KvStore::open(current_dir()?)?;
            if let Some(value) = store.get(key.to_string())? {
                println!("{value}");
            } else {
                println!("Key not found");
            }
        }
        ("rm", Some(args)) => {
            let key = args.value_of("KEY").unwrap();
            let mut store = KvStore::open(current_dir()?)?;
            match store.remove(key.to_string()) {
                Err(KvStoreError::RemoveNonExistKey) => {
                    println!("Key not found");
                    exit(1);
                }
                _ => {}
            }
        }
        _ => {
            exit(1);
        }
    }
    Ok(())
}
