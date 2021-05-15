use confy;

use std::process;

use argh::FromArgs;

mod config;
mod hosts;

#[derive(FromArgs)]
/// Manage your hosts file
struct Oh {
    /// show the version
    #[argh(switch, short = 'V')]
    version: bool,

    /// path to config file to use
    #[argh(option, default = "config::DEFAULT_PATH.to_string()")]
    config: String,

    /// print to stdout instead of updating the hosts file
    #[argh(switch)]
    dry_run: bool,
}

fn main() {
    let args: Oh = argh::from_env();

    if args.version {
        println!(std::env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    let cfg: config::Hosts = confy::load_path(args.config).unwrap_or_else(|err| {
        println!("{}", err);
        process::exit(1);
    });

    let mut hosts_file = hosts::HostsFile::new().unwrap_or_else(|err| {
        println!("{}", err);
        process::exit(1);
    });

    hosts_file.append(&cfg).unwrap_or_else(|err| {
        println!("{}", err);
        process::exit(1);
    });

    let mut out: Box<dyn std::io::Write> = if args.dry_run == true {
        Box::new(std::io::stdout())
    } else {
        match std::fs::File::open(config::DEFAULT_PATH) {
            Ok(f) => Box::new(f),
            Err(err) => {
                println!("{}", err);
                process::exit(1);
            }
        }
    };

    hosts_file.format(&mut out).unwrap_or_else(|err| {
        println!("{}", err);
        process::exit(1);
    });
}
