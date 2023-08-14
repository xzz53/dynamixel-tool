mod v1;
mod v2;

use anyhow::Result;
use serialport::SerialPort;
use std::{fmt::Display, str::FromStr};
use thiserror::Error;

use v1::ProtocolV1;
use v2::ProtocolV2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProtocolVersion {
    V1 = 1,
    V2 = 2,
}

impl Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (*self as u8).fmt(f)
    }
}

#[derive(Error, Debug)]
pub enum ProtocolVersionError {
    #[error("invalid protocol '{0}'")]
    BadProtocol(String),
}

impl FromStr for ProtocolVersion {
    type Err = ProtocolVersionError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "1" => Ok(ProtocolVersion::V1),
            "2" => Ok(ProtocolVersion::V2),
            _ => Err(ProtocolVersionError::BadProtocol(input.to_string())),
        }
    }
}

pub fn make_protocol<'a>(
    version: ProtocolVersion,
    port: &'a mut dyn SerialPort,
    retries: usize,
) -> Box<dyn Protocol + 'a> {
    match version {
        ProtocolVersion::V1 => Box::new(ProtocolV1::new(port, retries)),
        ProtocolVersion::V2 => Box::new(ProtocolV2::new(port, retries)),
    }
}

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("corrupted status packet")]
    BadPacket,
    #[error("invalid address for chosen protocol")]
    InvalidAddress,
    #[error("invalid byte count for chosen protocol")]
    InvalidCount,
    #[error("dynamixel status error {0}")]
    StatusError(u8),
}

pub trait Protocol: Send {
    fn scan(&mut self, scan_start: u8, scan_end: u8) -> Result<Vec<u8>>;
    fn read(&mut self, id: u8, address: u16, count: u16) -> Result<Vec<u8>>;
    fn write(&mut self, id: u8, address: u16, data: &[u8]) -> Result<()>;
    fn version(&self) -> ProtocolVersion;
}
