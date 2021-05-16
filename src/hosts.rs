use std::fs::File;
use std::io::BufWriter;
use std::io::{BufRead, BufReader, Write};
use std::net;

use super::config;

#[cfg(target_os = "windows")]
pub static LOCATION: &str = "C:\\Windows\\System32\\drivers\\etc";

#[cfg(target_os = "linux")]
pub static LOCATION: &str = "/etc/hosts";

#[cfg(target_os = "darwin")]
pub static LOCATION: &str = "/private/etc/hosts";

enum LineKind {
    Before,
    Managed,
    After,
}

#[derive(Debug)]
struct ManagedLine {
    ip: net::IpAddr,
    hostnames: Vec<String>,
}

impl std::fmt::Display for ManagedLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:16} {}\n", self.ip, self.hostnames.join(" "))
    }
}

#[derive(Debug)]
pub struct HostsFile {
    before_lines: Vec<String>,
    managed_lines: Vec<ManagedLine>,
    after_lines: Vec<String>,
}

const BEGIN_TAG: &str = "# BEGIN ho — DO NOT REMOVE THIS LINE";
const END_TAG: &str = "# END ho — DO NOT REMOVE THIS LINE";

impl HostsFile {
    fn parse(
        reader: BufReader<File>,
    ) -> Result<(Vec<String>, Vec<ManagedLine>, Vec<String>), std::io::Error> {
        let mut before_lines: Vec<String> = Vec::new();
        let mut managed_lines: Vec<ManagedLine> = Vec::new();
        let mut after_lines: Vec<String> = Vec::new();

        let mut line_kind = LineKind::Before;

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if line == BEGIN_TAG {
                        line_kind = LineKind::Managed;
                    } else if line == END_TAG {
                        line_kind = LineKind::After;
                    }

                    match line_kind {
                        LineKind::Before => before_lines.push(line),
                        LineKind::Managed => {
                            let matched: Vec<&str> = line.split(r"\s+").collect();
                            let parsed_line = ManagedLine {
                                ip: matched[0].parse().unwrap(),
                                hostnames: matched[1]
                                    .trim()
                                    .split(" ")
                                    .map(|x| x.to_string())
                                    .collect(),
                            };
                            managed_lines.push(parsed_line);
                        }
                        LineKind::After => after_lines.push(line),
                    };
                }
                Err(err) => return Err(err),
            };
        }

        Ok((before_lines, managed_lines, after_lines))
    }

    /// Opens and reads the host file
    pub fn new() -> Result<Self, std::io::Error> {
        let f = File::open(LOCATION);

        let f = match f {
            Ok(file) => file,
            Err(e) => return Err(e),
        };

        let reader = BufReader::new(f);

        match HostsFile::parse(reader) {
            Ok((before_lines, managed_lines, after_lines)) => {
                return Ok(Self {
                    before_lines,
                    managed_lines,
                    after_lines,
                })
            }
            Err(err) => return Err(err),
        };
    }

    pub fn append(&mut self, entries: &config::Hosts) {
        for (ip, hostnames) in entries.iter() {
            self.managed_lines.push(ManagedLine {
                ip: *ip,
                hostnames: hostnames.to_vec(),
            })
        }
    }

    pub fn format(
        self,
        writer: &mut impl std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut buf_writer = BufWriter::new(writer);
        buf_writer.write(self.before_lines.join("\n").as_bytes())?;

        buf_writer.write("\n\n".as_bytes())?;
        buf_writer.write(BEGIN_TAG.as_bytes())?;
        buf_writer.write("\n".as_bytes())?;

        for line in self.managed_lines {
            buf_writer.write(format!("{}\n", line).as_bytes())?;
        }

        buf_writer.write(END_TAG.as_bytes())?;
        buf_writer.write("\n\n".as_bytes())?;

        buf_writer.write(self.after_lines.join("\n").as_bytes())?;

        Ok(buf_writer.flush()?)
    }
}
