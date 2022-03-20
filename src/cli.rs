use anyhow::Result;
pub use clap::StructOpt;
use clap::{Parser, Subcommand};
use lazy_static::lazy_static;
use regex::Regex;
use std::cmp;
use std::ops::Deref;
use std::str::FromStr;
use thiserror::Error;

use dynamixel_utils::protocol::ProtocolVersion;
use dynamixel_utils::regs::RegSpec;

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

fn parse_with_radix<T>(input: &str) -> Result<T, T::FromStrRadixErr>
where
    T: num::Num,
    <T as num::Num>::FromStrRadixErr: std::error::Error + Send + Sync,
{
    if input.starts_with("0x") {
        T::from_str_radix(input.trim_start_matches("0x"), 16)
    } else if input.starts_with("0b") {
        T::from_str_radix(input.trim_start_matches("0b"), 2)
    } else {
        T::from_str_radix(input.trim_start_matches("0b"), 10)
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
    pub protocol: ProtocolVersion,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List known device models
    ListModels,

    /// List registers for a model
    ListRegisters { model: String },

    /// Scan for servos
    Scan {
        #[clap(default_value_t = 0, parse(try_from_str=parse_with_radix))]
        scan_start: u8,
        #[clap(default_value_t = 253, parse(try_from_str=parse_with_radix))]
        scan_end: u8,
    },

    /// Read unsigned 8-bit integer
    #[clap(visible_alias = "readb")]
    ReadUint8 {
        ids: IdRange,
        #[clap(parse(try_from_str=parse_with_radix))]
        address: u16,
    },

    /// Read unsigned 16-bit integer
    #[clap(visible_alias = "readh")]
    ReadUint16 {
        ids: IdRange,
        #[clap(parse(try_from_str=parse_with_radix))]
        address: u16,
    },

    /// Read unsigned 32-bit integer
    #[clap(visible_alias = "readw")]
    ReadUint32 {
        ids: IdRange,
        #[clap(parse(try_from_str=parse_with_radix))]
        address: u16,
    },

    /// Read byte array
    #[clap(visible_alias = "reada")]
    ReadBytes {
        ids: IdRange,
        #[clap(parse(try_from_str=parse_with_radix))]
        address: u16,
        #[clap(parse(try_from_str=parse_with_radix))]
        count: u16,
    },

    /// Read register
    ReadReg { ids: IdRange, reg: RegSpec },

    /// Write unsigned 8-bit integer
    #[clap(visible_alias = "writeb")]
    WriteUint8 {
        ids: IdRange,
        #[clap(parse(try_from_str=parse_with_radix))]
        address: u16,
        #[clap(parse(try_from_str=parse_with_radix))]
        value: u8,
    },

    /// Write unsigned 16-bit integer
    #[clap(visible_alias = "writeh")]
    WriteUint16 {
        ids: IdRange,
        #[clap(parse(try_from_str=parse_with_radix))]
        address: u16,
        #[clap(parse(try_from_str=parse_with_radix))]
        value: u16,
    },

    /// Write unsigned 32-bit integer
    #[clap(visible_alias = "writew")]
    WriteUint32 {
        ids: IdRange,
        #[clap(parse(try_from_str=parse_with_radix))]
        address: u16,
        #[clap(parse(try_from_str=parse_with_radix))]
        value: u32,
    },

    /// Write byte array
    #[clap(visible_alias = "writea")]
    WriteBytes {
        ids: IdRange,
        #[clap(parse(try_from_str=parse_with_radix))]
        address: u16,
        #[clap(required = true, parse(try_from_str=parse_with_radix))]
        values: Vec<u8>,
    },

    /// Write register
    WriteReg {
        ids: IdRange,
        reg: RegSpec,
        value: u32,
    },
}
