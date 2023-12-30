pub mod master;

use anyhow::Result;
use std::{fmt::Display, str::FromStr};
use thiserror::Error;

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
