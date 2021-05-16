use std::fmt;
use std::fs::File;
use std::io::BufWriter;
use std::io::{BufRead, BufReader, Write};
use std::net;

use super::config;

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

#[derive(Debug)]
pub enum ErrorKind {
    Io(std::io::Error),
    Parse(MissingEndTagError),
}

impl From<std::io::Error> for ErrorKind {
    fn from(error: std::io::Error) -> Self {
        ErrorKind::Io(error)
    }
}

impl From<MissingEndTagError> for ErrorKind {
    fn from(error: MissingEndTagError) -> Self {
        ErrorKind::Parse(error)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::Io(ref err) => write!(f, "IO error: {}", err),
            ErrorKind::Parse(ref err) => write!(f, "Parse error: {}", err),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MissingEndTagError;

impl fmt::Display for MissingEndTagError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BEGIN tag found but END tag is missing")
    }
}

const BEGIN_TAG: &str = "# BEGIN ho — DO NOT REMOVE THIS LINE";
const END_TAG: &str = "# END ho — DO NOT REMOVE THIS LINE";

#[derive(Debug)]
struct ManagedLine {
    ip: net::IpAddr,
    hostnames: Vec<String>,
}

impl std::fmt::Display for ManagedLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:16} {}", self.ip, self.hostnames.join(" "))
    }
}

impl PartialEq for ManagedLine {
    fn eq(&self, other: &Self) -> bool {
        format!("{}", self) == format!("{}", other)
    }
}

#[derive(Debug)]
pub struct HostsFile {
    before_lines: Vec<String>,
    managed_lines: Vec<ManagedLine>,
    after_lines: Vec<String>,
}

impl HostsFile {
    /// Opens and reads the host file
    pub fn open(location: String) -> Result<Self, ErrorKind> {
        let f = File::open(location);

        let f = match f {
            Ok(file) => file,
            Err(e) => return Err(ErrorKind::Io(e)),
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

    fn parse(
        reader: BufReader<File>,
    ) -> Result<(Vec<String>, Vec<ManagedLine>, Vec<String>), ErrorKind> {
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
                Err(err) => return Err(ErrorKind::Io(err)),
            };
        }

        match line_kind {
            LineKind::Managed => return Err(ErrorKind::Parse(MissingEndTagError)),
            _ => Ok((before_lines, managed_lines, after_lines)),
        }
    }

    pub fn write(
        &mut self,
        entries: &config::Hosts,
        writer: &mut impl std::io::Write,
    ) -> Result<Status, ErrorKind> {
        if !self.has_changed(entries) {
            return Ok(Status::NotChanged);
        }

        self.update(entries);
        match self.render(writer) {
            Ok(_) => Ok(Status::Changed),
            Err(err) => Err(err),
        }
    }

    fn has_changed(&mut self, entries: &config::Hosts) -> bool {
        for (index, (ip, hostnames)) in entries.iter().enumerate() {
            let l = ManagedLine {
                ip: *ip,
                hostnames: hostnames.to_vec(),
            };

            match self.managed_lines.get(index) {
                Some(line) => {
                    if l != *line {
                        return true;
                    }
                }
                None => return true,
            }
        }
        return false;
    }

    fn update(&mut self, entries: &config::Hosts) {
        self.managed_lines = entries
            .iter()
            .map(|(ip, hostnames)| ManagedLine {
                ip: *ip,
                hostnames: hostnames.to_vec(),
            })
            .collect();
    }

    fn render(&mut self, writer: &mut impl std::io::Write) -> Result<(), ErrorKind> {
        let mut buf_writer = BufWriter::new(writer);
        buf_writer.write(self.before_lines.join("\n").as_bytes())?;

        buf_writer.write("\n\n".as_bytes())?;
        buf_writer.write(BEGIN_TAG.as_bytes())?;
        buf_writer.write("\n".as_bytes())?;

        for line in &self.managed_lines {
            buf_writer.write(format!("{}\n", line).as_bytes())?;
        }

        buf_writer.write(END_TAG.as_bytes())?;
        buf_writer.write("\n\n".as_bytes())?;

        buf_writer.write(self.after_lines.join("\n").as_bytes())?;

        Ok(buf_writer.flush()?)
    }
}
