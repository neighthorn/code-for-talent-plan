use clap::{App, Arg, SubCommand};
use std::env;
use std::process::exit;
// use kvs::KvStore;

fn main() {
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
        ("set", Some(_args)) => {
            eprintln!("unimplemented");
            exit(1);
        }
        ("get", Some(_args)) => {
            eprintln!("unimplemented");
            exit(1);
        }
        ("rm", Some(_args)) => {
            eprintln!("unimplemented");
            exit(1);
        }
        _ => {
            exit(1);
        }
    }
}
