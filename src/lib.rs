use std::fmt;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net;
use std::path::Path;
use std::str::FromStr;

use anyhow::{Error, Result};
use regex::Regex;
use serde_derive::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
pub static OS_FILE: &str = "C:\\Windows\\System32\\drivers\\etc";

#[cfg(target_os = "linux")]
pub static OS_FILE: &str = "/etc/hosts";

#[cfg(target_os = "darwin")]
pub static OS_FILE: &str = "/private/etc/hosts";

pub enum Status {
    Changed,
    NotChanged,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Status::Changed => write!(f, "Updated"),
            Status::NotChanged => write!(f, "Already up to date; nothing to do."),
        }
    }
}

enum LineKind {
    Before,
    Managed,
    After,
}

const BEGIN_TAG: &str = "# BEGIN ho — DO NOT REMOVE THIS LINE";
const END_TAG: &str = "# END ho — DO NOT REMOVE THIS LINE";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub ip: net::IpAddr,
    pub hostnames: Vec<String>,
}

impl FromStr for Entry {
    fn from_str(s: &str) -> Result<Entry, Error> {
        let re = Regex::new(r"\s+")?;
        let matched: Vec<&str> = re.split(&s).collect();

        Ok(Entry {
            ip: matched[0].parse().map_err(Error::new)?,
            hostnames: matched[1]
                .trim()
                .split(" ")
                .map(|x| x.to_string())
                .collect(),
        })
    }
    type Err = Error;
}

impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:15} {}", self.ip, self.hostnames.join(" "))
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        // not pretty but works great
        format!("{}", self) == format!("{}", other)
    }
}

#[derive(Debug)]
pub struct File {
    before_lines: Vec<String>,
    managed_lines: Vec<Entry>,
    after_lines: Vec<String>,
}

impl File {
    /// Opens and reads a host file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let f = fs::File::open(path).map_err(Error::new)?;

        let reader = BufReader::new(f);

        match File::parse(reader) {
            Ok((before_lines, managed_lines, after_lines)) => {
                return Ok(File {
                    before_lines,
                    managed_lines,
                    after_lines,
                })
            }
            Err(err) => return Err(err),
        };
    }

    fn parse(reader: BufReader<fs::File>) -> Result<(Vec<String>, Vec<Entry>, Vec<String>), Error> {
        let mut before_lines: Vec<String> = Vec::new();
        let mut managed_lines: Vec<Entry> = Vec::new();
        let mut after_lines: Vec<String> = Vec::new();

        let mut line_kind = LineKind::Before;

        for line in reader.lines() {
            let line = line.map_err(Error::new)?;

            if line == BEGIN_TAG {
                line_kind = LineKind::Managed;
                continue;
            } else if line == END_TAG {
                line_kind = LineKind::After;
                continue;
            }

            match line_kind {
                LineKind::Before => before_lines.push(line),
                LineKind::Managed => managed_lines.push(line.parse::<Entry>()?),
                LineKind::After => after_lines.push(line),
            };
        }

        match line_kind {
            LineKind::Managed => Err(Error::msg("BEGIN tag found but END tag is missing")),
            _ => Ok((before_lines, managed_lines, after_lines)),
        }
    }

    pub fn write(
        &mut self,
        entries: &Vec<Entry>,
        writer: &mut impl std::io::Write,
    ) -> Result<Status, Error> {
        if !self.has_changed(entries) {
            return Ok(Status::NotChanged);
        }

        self.update(entries);
        match self.render(writer) {
            Ok(_) => Ok(Status::Changed),
            Err(err) => Err(err),
        }
    }

    fn has_changed(&mut self, entries: &Vec<Entry>) -> bool {
        for (index, entry) in entries.iter().enumerate() {
            match self.managed_lines.get(index) {
                Some(line) => {
                    if entry != line {
                        return true;
                    }
                }
                _ => return true,
            }
        }
        return false;
    }

    fn update(&mut self, entries: &Vec<Entry>) {
        self.managed_lines = entries.to_vec();
    }

    fn render(&mut self, writer: &mut impl std::io::Write) -> Result<(), Error> {
        let mut buf_writer = BufWriter::new(writer);
        buf_writer.write(self.before_lines.join("\n").as_bytes())?;

        buf_writer.write("\n".as_bytes())?;
        buf_writer.write(BEGIN_TAG.as_bytes())?;
        buf_writer.write("\n".as_bytes())?;

        for line in &self.managed_lines {
            buf_writer.write(format!("{}\n", line).as_bytes())?;
        }

        buf_writer.write(END_TAG.as_bytes())?;
        buf_writer.write("\n".as_bytes())?;

        buf_writer.write(self.after_lines.join("\n").as_bytes())?;

        Ok(buf_writer.flush()?)
    }
}
