use std::collections;
use std::fs;
use std::io;
use std::net;
use std::process;

use argh::FromArgs;
use confy;

use hostsfile;

pub static DEFAULT_PATH: &str = "ho.toml";

#[derive(FromArgs)]
/// Manage your hosts file
struct Oh {
    /// output to stdout instead of updating the hosts file
    #[argh(switch)]
    stdout: bool,

    /// path to hosts file; defaults to the OS file.
    #[argh(option, default = "hostsfile::OS_FILE.to_string()")]
    hostsfile: String,

    /// path to config file to use; defaults to ho.toml
    #[argh(option, short = 'c', default = "DEFAULT_PATH.to_string()")]
    config: String,

    /// show the version
    #[argh(switch, short = 'V')]
    version: bool,
}

fn main() {
    let args: Oh = argh::from_env();

    if args.version {
        eprintln!(std::env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    let cfg: collections::HashMap<net::IpAddr, Vec<String>> = confy::load_path(&args.config)
        .unwrap_or_else(|err| {
            eprintln!("failed to load file {}: {}", args.config, err);
            process::exit(1);
        });

    let entries = cfg
        .iter()
        .map(|(ip, hostnames)| hostsfile::Entry {
            ip: *ip,
            hostnames: hostnames.to_vec(),
        })
        .collect();

    let mut hosts_file = hostsfile::File::open(&args.hostsfile).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });

    let mut out: Box<dyn io::Write> = if args.stdout == true {
        Box::new(io::stdout())
    } else {
        match fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&args.hostsfile)
        {
            Ok(f) => Box::new(f),
            Err(err) => {
                eprintln!("failed to open {}: {}", args.hostsfile, err);
                process::exit(1);
            }
        }
    };

    match hosts_file.write(&entries, &mut out) {
        Ok(status @ hostsfile::Status::NotChanged) | Ok(status @ hostsfile::Status::Changed) => {
            if args.stdout == false {
                eprintln!("{}", status)
            }
        }
        Err(err) => {
            let out = if args.stdout == true {
                "stdout".to_string()
            } else {
                args.hostsfile
            };

            eprintln!("failed to write {}: {}", out, err);
            process::exit(1);
        }
    };
}
