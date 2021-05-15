use std::collections;
use std::net;

pub static DEFAULT_PATH: &str = "ho.toml";
pub type Hosts = collections::HashMap<net::IpAddr, Vec<String>>;
