use super::{Protocol, ProtocolError, Result, SerialPort};
use log::debug;

pub struct ProtocolV1 {}

impl Protocol for ProtocolV1 {
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
                if ping_v1(port, id).is_ok() {
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
        if address > 0xFE {
            return Err(ProtocolError::InvalidAddress.into());
        }

        if count > 0xFF {
            return Err(ProtocolError::InvalidCount.into());
        }

        let mut error = None;

        for _ in 0..=retries {
            match read_v1(port, id, address as u8, count as u8) {
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
    ) -> anyhow::Result<()> {
        let mut error = None;

        if address > 0xFF {
            return Err(ProtocolError::InvalidCount.into());
        }

        for _ in 0..=retries {
            match write_v1(port, id, address as u8, data) {
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

fn encode_instruction_v1(buffer: &mut [u8], id: u8, instruction: u8, params: &[u8]) -> usize {
    let length: u8 = (2 + params.len()) as u8;
    assert!(usize::from(length) <= buffer.len());

    buffer[0] = 0xFF;
    buffer[1] = 0xFF;
    buffer[2] = id;
    buffer[3] = length;
    buffer[4] = instruction;

    buffer[5..(params.len() + 5)].clone_from_slice(params);

    buffer[5 + params.len()] = !buffer[2..5 + params.len()]
        .iter()
        .cloned()
        .fold(0u8, |x, y| x.overflowing_add(y).0);
    6 + params.len()
}

fn decode_status_v1(buffer: &[u8], params: &mut [u8]) -> Option<usize> {
    if buffer.len() < 6 {
        return None;
    }

    let param_length: usize = (buffer[3] - 2).into();
    if buffer.len() < (6 + param_length) || buffer[0..2] != [0xFF, 0xFF] {
        return None;
    }

    let csum = buffer[2..5 + param_length]
        .iter()
        .cloned()
        .fold(0u8, |x, y| x.overflowing_add(y).0);

    if csum != !buffer[5 + param_length] || buffer[4] != 0x0 {
        return None;
    }

    params[..param_length].copy_from_slice(&buffer[5..5 + param_length]);

    Some(6 + param_length)
}

fn ping_v1(port: &mut dyn SerialPort, id: u8) -> Result<()> {
    let mut buffer: [u8; 255] = [0u8; 255];
    let mut params: [u8; 255] = [0u8; 255];

    let len_write = encode_instruction_v1(&mut buffer, id, OPCODE_PING, &[]);
    let len_read = 6;

    debug!("ping {}", id);
    debug!("send {:?}", &buffer[0..len_write]);
    port.write_all(&buffer[0..len_write])?;

    port.read_exact(&mut buffer[0..len_read])?;
    debug!("recv {:?}", &buffer[0..len_read]);

    decode_status_v1(&buffer, &mut params)
        .map(|_| ())
        .ok_or_else(|| ProtocolError::BadPacket.into())
}

fn read_v1(port: &mut dyn SerialPort, id: u8, address: u8, count: u8) -> Result<Vec<u8>> {
    let mut buffer = [0u8; 255];
    let mut params = [0u8; 255];

    let len_write = encode_instruction_v1(&mut buffer, id, OPCODE_READ, &[address, count]);

    debug!("read1 {} {} {}", id, address, count);
    debug!("send {:?}", &buffer[0..len_write]);
    port.write_all(&buffer[0..len_write])?;

    let len_read = (6 + count) as usize;
    port.read_exact(&mut buffer[0..len_read])?;
    debug!("recv {:?}", &buffer[0..len_read]);

    match decode_status_v1(&buffer, &mut params) {
        Some(_) => Ok(params[0..count.into()].to_vec()),
        None => Err(ProtocolError::BadPacket.into()),
    }
}

fn write_v1(port: &mut dyn SerialPort, id: u8, address: u8, data: &[u8]) -> Result<()> {
    let mut buffer: [u8; 255] = [0; 255];
    let mut params: [u8; 255] = [0; 255];

    params[0] = address;
    params[1..data.len() + 1].copy_from_slice(data);

    let len_write = encode_instruction_v1(&mut buffer, id, OPCODE_WRITE, &params[..data.len() + 1]);

    debug!("write1 {} {} {:?}", id, address, data);
    debug!("send {:?}", &buffer[0..len_write]);
    port.write_all(&buffer[0..len_write])?;

    let len_read = 6;

    port.read_exact(&mut buffer[0..len_read])?;
    debug!("recv {:?}", &buffer[0..len_read]);

    decode_status_v1(&buffer, &mut params)
        .ok_or_else(|| ProtocolError::BadPacket.into())
        .map(|_| ())
}
