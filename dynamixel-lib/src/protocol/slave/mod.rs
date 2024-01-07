mod v1;
mod v2;

use async_trait::async_trait;
use num_derive::{FromPrimitive, ToPrimitive};
use tokio_serial::SerialStream;

use super::{ProtocolVersion, Result};

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum Opcode {
    Ping = 0x01,
    Read = 0x02,
    Write = 0x03,
    RegWrite = 0x04,
    Action = 0x05,
    FactoryReset = 0x06,
    Reboot = 0x08,
    Clear = 0x10,
    ControlTableBackup = 0x20,
    SyncRead = 0x82,
    SyncWrite = 0x83,
    FastSyncRead = 0x8A,
    BulkRead = 0x92,
    BulkWrite = 0x93,
    FastBulkRead = 0x9A,
}

#[derive(Debug)]
pub struct RawInstruction {
    pub version: ProtocolVersion,
    pub id: u8,
    pub opcode: Opcode,
    pub data: Vec<u8>,
}

#[async_trait]
pub trait AsyncProtocol {
    async fn recv_instruction(&mut self) -> Result<RawInstruction>;
    async fn send_status(&mut self, id: u8, status: u8, params: &[u8]) -> Result<()>;
}

pub fn make_async_protocol<'a>(
    version: ProtocolVersion,
    port: &'a mut SerialStream,
) -> Box<dyn AsyncProtocol + 'a> {
    match version {
        ProtocolVersion::V1 => Box::new(v1::ProtocolV1::new(port)),
        ProtocolVersion::V2 => Box::new(v2::ProtocolV2::new(port)),
    }
}
