use anyhow::Result;
pub use clap::StructOpt;
use clap::{Parser, Subcommand};
use lazy_static::lazy_static;
use regex::Regex;
use std::cmp;
use std::ops::Deref;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RangeError {
    #[error("invalid range '{0}'")]
    BadRange(String),
}

#[derive(Debug)]
pub struct IdRange(Vec<u8>);

impl Deref for IdRange {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for IdRange {
    type Err = RangeError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+)(?:-(\d+))?$").unwrap();
        }

        let mut result: Vec<u8> = Vec::new();

        for s in input.split(',') {
            if let Some(c) = RE.captures(s) {
                if c.get(2).is_none() {
                    let val = c
                        .get(1)
                        .unwrap()
                        .as_str()
                        .parse::<u8>()
                        .map_err(|_| RangeError::BadRange(s.to_string()))?;
                    result.push(val)
                } else {
                    let val1 = c
                        .get(1)
                        .unwrap()
                        .as_str()
                        .parse::<u8>()
                        .map_err(|_| RangeError::BadRange(s.to_string()))?;
                    let val2 = c
                        .get(2)
                        .unwrap()
                        .as_str()
                        .parse::<u8>()
                        .map_err(|_| RangeError::BadRange(s.to_string()))?;
                    result.extend(cmp::min(val1, val2)..=cmp::max(val1, val2));
                }
            } else {
                return Err(RangeError::BadRange(s.to_string()));
            }
        }

        result.sort_unstable();
        Ok(IdRange(result))
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Skip sanity checks
    #[clap(long, short)]
    pub force: bool,

    /// enable debug output
    #[clap(long, short)]
    pub debug: bool,

    /// UART device or 'auto'
    #[clap(long, short, default_value = "auto")]
    pub port: String,

    /// UART baud rate
    #[clap(long, short, default_value_t = 57600)]
    pub baudrate: u32,

    /// Read/write retry count
    #[clap(long, short, default_value_t = 0)]
    pub retries: usize,

    /// Use json-formatted output
    #[clap(long, short)]
    pub json: bool,

    /// Dynamixel protocol version
    #[clap(long, short = 'P', default_value = "1")]
    pub protocol: String,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scan for servos
    Scan {
        #[clap(default_value_t = 0)]
        scan_start: u8,
        #[clap(default_value_t = 253)]
        scan_end: u8,
    },

    /// Read unsigned 8-bit integer
    #[clap(visible_alias = "readb")]
    ReadUint8 { ids: IdRange, address: u16 },

    /// Read unsigned 16-bit integer
    #[clap(visible_alias = "readh")]
    ReadUint16 { ids: IdRange, address: u16 },

    /// Read unsigned 32-bit integer
    #[clap(visible_alias = "readw")]
    ReadUint32 { ids: IdRange, address: u16 },

    /// Read byte array
    #[clap(visible_alias = "reada")]
    ReadBytes {
        ids: IdRange,
        address: u16,
        count: u16,
    },

    /// Write unsigned 8-bit integer
    #[clap(visible_alias = "writeb")]
    WriteUint8 {
        ids: IdRange,
        address: u16,
        value: u8,
    },

    /// Write unsigned 16-bit integer
    #[clap(visible_alias = "writeh")]
    WriteUint16 {
        ids: IdRange,
        address: u16,
        value: u16,
    },

    /// Write unsigned 32-bit integer
    #[clap(visible_alias = "writew")]
    WriteUint32 {
        ids: IdRange,
        address: u16,
        value: u32,
    },

    /// Write byte array
    #[clap(visible_alias = "writea")]
    WriteBytes {
        ids: IdRange,
        address: u16,
        #[clap(required = true)]
        values: Vec<u8>,
    },
}
