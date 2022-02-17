mod v1;
mod v2;

use anyhow::Result;
use serialport::SerialPort;
use thiserror::Error;

pub use v1::ProtocolV1;
pub use v2::ProtocolV2;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("corrupted status packet")]
    BadPacket,
    #[error("invalid address for chosen protocol")]
    InvalidAddress,
    #[error("invalid byte count for chosen protocol")]
    InvalidCount,
}

pub trait Protocol {
    fn scan(
        &self,
        port: &mut dyn SerialPort,
        retries: usize,
        scan_start: u8,
        scan_end: u8,
    ) -> Result<Vec<u8>>;

    fn read(
        &self,
        port: &mut dyn SerialPort,
        retries: usize,
        id: u8,
        address: u16,
        count: u16,
    ) -> Result<Vec<u8>>;

    fn write(
        &self,
        port: &mut dyn SerialPort,
        retries: usize,
        id: u8,
        address: u16,
        data: &[u8],
    ) -> Result<()>;
}
