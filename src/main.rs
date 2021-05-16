use std::fs;
use std::io;
use std::process;

use argh::FromArgs;
use confy;

mod config;
mod hosts;

#[derive(FromArgs)]
/// Manage your hosts file
struct Oh {
    /// output to stdout instead of updating the hosts file
    #[argh(switch)]
    stdout: bool,

    /// path to hosts file; defaults to the OS file.
    #[argh(option)]
    hostsfile: Option<String>,

    /// path to config file to use; defaults to ho.toml
    #[argh(option, short = 'c', default = "config::DEFAULT_PATH.to_string()")]
    config: String,

    /// show the version
    #[argh(switch, short = 'V')]
    version: bool,
}

fn main() {
    let args: Oh = argh::from_env();

    if args.version {
        println!(std::env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    let cfg: config::Hosts = confy::load_path(args.config).unwrap_or_else(|err| {
        println!("failed to load file {}: {}", config::DEFAULT_PATH, err);
        process::exit(1);
    });

    let mut hosts_file = hosts::HostsFile::new(args.hostsfile).unwrap_or_else(|err| {
        println!("{}", err);
        process::exit(1);
    });

    hosts_file.append(&cfg);

    let mut out: Box<dyn io::Write> = if args.stdout == true {
        Box::new(io::stdout())
    } else {
        match fs::File::open(config::DEFAULT_PATH) {
            Ok(f) => Box::new(f),
            Err(err) => {
                println!("failed to open {}: {}", config::DEFAULT_PATH, err);
                process::exit(1);
            }
        }
    };

    hosts_file.format(&mut out).unwrap_or_else(|err| {
        println!("failed to write to file: {}", err);
        process::exit(1);
    });
}
