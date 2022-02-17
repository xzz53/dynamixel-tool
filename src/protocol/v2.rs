use super::{Protocol, ProtocolError, Result, SerialPort};
use crc::{self, Crc, CRC_16_UMTS};
use log::debug;
use std::convert::TryInto;

pub struct ProtocolV2 {}

impl Protocol for ProtocolV2 {
    fn scan(
        &self,
        port: &mut dyn SerialPort,
        retries: usize,
        scan_start: u8,
        scan_end: u8,
    ) -> Result<Vec<u8>> {
        let mut result: Vec<u8> = Vec::new();
        (scan_start..scan_end).into_iter().for_each(|id| {
            for _ in 0..=retries {
                if ping(port, id).is_ok() {
                    result.push(id);
                    break;
                }
            }
        });
        Ok(result)
    }

    fn read(
        &self,
        port: &mut dyn SerialPort,
        retries: usize,
        id: u8,
        address: u16,
        count: u16,
    ) -> Result<Vec<u8>> {
        let mut error = None;
        for _ in 0..=retries {
            match read1(port, id, address, count) {
                Ok(data) => return Ok(data),
                Err(e) => error = Some(e),
            }
        }
        Err(error.unwrap())
    }

    fn write(
        &self,
        port: &mut dyn SerialPort,
        retries: usize,
        id: u8,
        address: u16,
        data: &[u8],
    ) -> Result<()> {
        let mut error = None;

        for _ in 0..=retries {
            match write1(port, id, address, data) {
                Ok(data) => return Ok(data),
                Err(e) => error = Some(e),
            }
        }
        Err(error.unwrap())
    }
}

const OPCODE_PING: u8 = 1;
const OPCODE_READ: u8 = 2;
const OPCODE_WRITE: u8 = 3;

fn encode_instruction_v2(buffer: &mut [u8], id: u8, instruction: u8, params: &[u8]) -> usize {
    let length = (3 + params.len()) as u16;
    assert!(usize::from(length) <= buffer.len());

    buffer[0] = 0xFF;
    buffer[1] = 0xFF;
    buffer[2] = 0xFD;
    buffer[3] = 0x00;
    buffer[4] = id;
    buffer[5..7].copy_from_slice(&length.to_le_bytes());
    buffer[7] = instruction;

    buffer[8..(8 + params.len())].clone_from_slice(params);

    let crc = Crc::<u16>::new(&CRC_16_UMTS);
    let cs = crc.checksum(&buffer[0..(8 + params.len())]);

    buffer[8 + params.len()..10 + params.len()].clone_from_slice(&cs.to_le_bytes());
    10 + params.len()
}

fn decode_status_v2(buffer: &[u8], params: &mut [u8]) -> Option<usize> {
    if buffer.len() < 10 {
        return None;
    }

    let param_length: usize = u16::from_le_bytes(buffer[5..7].try_into().unwrap()) as usize - 4;

    if buffer.len() < (10 + param_length) || buffer[0..4] != [0xFF, 0xFF, 0xFD, 0x00] {
        return None;
    }

    let crc = Crc::<u16>::new(&CRC_16_UMTS);
    let cs = crc.checksum(&buffer[0..(9 + param_length)]);

    if buffer[9 + param_length..11 + param_length] != cs.to_le_bytes() {
        return None;
    }

    params[..param_length].copy_from_slice(&buffer[9..9 + param_length]);

    Some(10 + param_length)
}

fn ping(port: &mut dyn SerialPort, id: u8) -> Result<()> {
    let mut buffer = [0u8; 255];
    let mut params = [0u8; 255];

    let len_write = encode_instruction_v2(&mut buffer, id, OPCODE_PING, &[]);
    let len_read = 14;

    debug!("ping {}", id);
    debug!("send {:?}", &buffer[0..len_write]);
    port.write_all(&buffer[0..len_write])?;

    port.read_exact(&mut buffer[0..len_read])?;
    debug!("recv {:?}", &buffer[0..len_read]);

    decode_status_v2(&buffer, &mut params)
        .map(|_| ())
        .ok_or_else(|| ProtocolError::BadPacket.into())
}

fn read1(port: &mut dyn SerialPort, id: u8, address: u16, count: u16) -> Result<Vec<u8>> {
    let mut buffer = [0u8; 255];
    let mut params = [0u8; 255];

    let len_write = encode_instruction_v2(
        &mut buffer,
        id,
        OPCODE_READ,
        &[address.to_le_bytes(), count.to_le_bytes()].concat(),
    );

    debug!("read1 {} {} {}", id, address, count);
    debug!("send {:?}", &buffer[0..len_write]);
    port.write_all(&buffer[0..len_write])?;

    let len_read = (11 + count) as usize;
    port.read_exact(&mut buffer[0..len_read])?;
    debug!("recv {:?}", &buffer[0..len_read]);

    match decode_status_v2(&buffer, &mut params) {
        Some(_) => Ok(params[0..count.into()].to_vec()),
        None => Err(ProtocolError::BadPacket.into()),
    }
}

fn write1(port: &mut dyn SerialPort, id: u8, address: u16, data: &[u8]) -> Result<()> {
    let mut buffer: [u8; 255] = [0; 255];
    let mut params: [u8; 255] = [0; 255];

    params[0..2].clone_from_slice(&address.to_le_bytes());
    params[2..2 + data.len()].copy_from_slice(data);

    let len_write = encode_instruction_v2(&mut buffer, id, OPCODE_WRITE, &params[..2 + data.len()]);

    debug!("write1 {} {} {:?}", id, address, data);
    debug!("send {:?}", &buffer[0..len_write]);
    port.write_all(&buffer[0..len_write])?;

    let len_read = 11;

    port.read_exact(&mut buffer[0..len_read])?;
    debug!("recv {:?}", &buffer[0..len_read]);

    decode_status_v2(&buffer, &mut params)
        .ok_or_else(|| ProtocolError::BadPacket.into())
        .map(|_| ())
}