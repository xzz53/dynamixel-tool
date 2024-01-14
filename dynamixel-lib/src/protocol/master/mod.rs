mod v1;
mod v2;

use serialport::SerialPort;

use super::{ProtocolVersion, Result};

pub trait Protocol: Send {
    fn scan(&mut self, scan_start: u8, scan_end: u8) -> Result<Vec<u8>>;
    fn read(&mut self, id: u8, address: u16, count: u16) -> Result<Vec<u8>>;
    fn write(&mut self, id: u8, address: u16, data: &[u8]) -> Result<()>;
    fn sync_write(&mut self, ids: &[u8], address: u16, data: &[&[u8]]) -> Result<()>;
    fn sync_read(&mut self, ids: &[u8], address: u16, count: u16) -> Result<Vec<Vec<u8>>>;

    fn version(&self) -> ProtocolVersion;
}

pub fn make_protocol<'a>(
    version: ProtocolVersion,
    port: &'a mut dyn SerialPort,
    retries: usize,
) -> Box<dyn Protocol + 'a> {
    match version {
        ProtocolVersion::V1 => Box::new(v1::ProtocolV1::new(port, retries)),
        ProtocolVersion::V2 => Box::new(v2::ProtocolV2::new(port, retries)),
    }
}
