use anyhow::Result;
use log::debug;
use serialport::{self, TTYPort};
use std::io::{Read, Write};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DxlError {
    #[error("corrupted status packet")]
    BadPacket,
}

const OPCODE_PING: u8 = 1;
const OPCODE_READ: u8 = 2;
const OPCODE_WRITE: u8 = 3;

fn encode_instruction(buffer: &mut [u8], id: u8, instruction: u8, params: &[u8]) -> usize {
    let length: u8 = (2 + params.len()) as u8;
    assert!(usize::from(length) <= buffer.len());

    buffer[0] = 0xFF;
    buffer[1] = 0xFF;
    buffer[2] = id;
    buffer[3] = length;
    buffer[4] = instruction;

    for i in 0..params.len() {
        buffer[5 + i] = params[i];
    }

    buffer[5 + params.len()] = !buffer[2..5 + params.len()].iter().sum::<u8>();
    6 + params.len()
}

fn decode_status(buffer: &[u8], params: &mut [u8]) -> Option<usize> {
    if buffer.len() < 6 {
        return None;
    }

    let param_length: usize = (buffer[3] - 2).into();
    if buffer.len() < (6 + param_length).into() || buffer[0] != 0xFF || buffer[1] != 0xFF {
        return None;
    }

    let csum: u8 = buffer[2..5 + param_length].iter().sum::<u8>();
    if csum != !buffer[5 + param_length] || buffer[4] != 0x0 {
        return None;
    }

    params[..param_length].copy_from_slice(&buffer[5..5 + param_length]);

    Some(6 + param_length)
}

fn ping(port: &mut TTYPort, id: u8) -> Result<()> {
    let mut buffer: [u8; 255] = [0; 255];
    let mut params: [u8; 255] = [0; 255];

    let len_write = encode_instruction(&mut buffer, id, OPCODE_PING, &[]);

    debug!("ping {}", id);
    debug!("send {:?}", &buffer[0..len_write]);
    port.write(&buffer[0..len_write])?;

    port.read_exact(&mut buffer[0..6])?;
    debug!("recv {:?}", &buffer[0..6]);
    decode_status(&buffer, &mut params)
        .map(|_| ())
        .ok_or(DxlError::BadPacket.into())
}

pub fn scan(port: &mut TTYPort, retries: usize, scan_start: u8, scan_end: u8) -> Result<Vec<u8>> {
    let mut result: Vec<u8> = Vec::new();
    (scan_start..scan_end).into_iter().for_each(|id| {
        for _ in 0..=retries {
            match ping(port, id) {
                Ok(_) => {
                    result.push(id);
                    break;
                }
                Err(_) => (),
            }
        }
    });
    Ok(result)
}

fn read1(port: &mut TTYPort, id: u8, address: u8, count: u8) -> Result<Vec<u8>> {
    let mut buffer: [u8; 255] = [0; 255];
    let mut params: [u8; 255] = [0; 255];

    let len_write = encode_instruction(&mut buffer, id, OPCODE_READ, &[address, count]);

    debug!("read1 {} {} {}", id, address, count);
    debug!("send {:?}", &buffer[0..len_write]);
    port.write(&buffer[0..len_write])?;

    let len_read = 6 + count as usize;
    port.read_exact(&mut buffer[0..len_read])?;
    debug!("recv {:?}", &buffer[0..len_read]);
    match decode_status(&buffer, &mut params) {
        Some(_) => Ok(params[0..count.into()].to_vec()),
        None => Err(DxlError::BadPacket.into()),
    }
}

pub fn read(port: &mut TTYPort, retries: usize, id: u8, address: u8, count: u8) -> Result<Vec<u8>> {
    let mut error = None;

    for _ in 0..=retries {
        match read1(port, id, address, count) {
            Ok(data) => return Ok(data),
            Err(e) => error = Some(e),
        }
    }
    Err(error.unwrap())
}

fn write1(port: &mut TTYPort, id: u8, address: u8, data: &[u8]) -> Result<()> {
    let mut buffer: [u8; 255] = [0; 255];
    let mut params: [u8; 255] = [0; 255];

    params[0] = address;
    params[1..data.len() + 1].copy_from_slice(data);

    let len_write = encode_instruction(&mut buffer, id, OPCODE_WRITE, &params[..data.len() + 1]);

    debug!("write1 {} {} {:?}", id, address, data);
    debug!("send {:?}", &buffer[0..len_write]);
    port.write(&buffer[0..len_write])?;

    port.read_exact(&mut buffer[0..6])?;
    debug!("recv {:?}", &buffer[0..6]);
    decode_status(&buffer, &mut params)
        .ok_or(DxlError::BadPacket.into())
        .map(|_| ())
}

pub fn write(port: &mut TTYPort, retries: usize, id: u8, address: u8, data: &[u8]) -> Result<()> {
    let mut error = None;

    for _ in 0..=retries {
        match write1(port, id, address, data) {
            Ok(data) => return Ok(data),
            Err(e) => error = Some(e),
        }
    }
    Err(error.unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_ping() {
        let reference: [u8; 6] = [0xFF, 0xFF, 0x01, 0x02, 0x01, 0xFB];
        let mut check: [u8; 6] = [0; 6];

        assert_eq!(
            encode_instruction(&mut check, 1, OPCODE_PING, &[]),
            check.len()
        );

        assert_eq!(reference, check);
    }

    #[test]
    fn encode_read() {
        let reference: [u8; 8] = [0xFF, 0xFF, 0x01, 0x04, 0x02, 0x2B, 0x01, 0xCC];
        let mut check: [u8; 8] = [0; 8];

        assert_eq!(
            encode_instruction(&mut check, 1, OPCODE_READ, &[43, 1]),
            check.len()
        );

        assert_eq!(reference, check);
    }

    #[test]
    fn decode_status_ping() {
        let reference: [u8; 6] = [0xFF, 0xFF, 0x01, 0x02, 0x00, 0xFC];
        let mut params: [u8; 0] = [];

        assert_eq!(
            decode_status(&reference, &mut params),
            Some(reference.len())
        );
    }

    #[test]
    fn decode_status_read() {
        let reference: [u8; 7] = [0xFF, 0xFF, 0x01, 0x03, 0x00, 0x20, 0xDB];
        let mut params: [u8; 1] = [0; 1];

        assert_eq!(
            decode_status(&reference, &mut params),
            Some(reference.len())
        );

        assert_eq!(params, [32]);
    }
}
