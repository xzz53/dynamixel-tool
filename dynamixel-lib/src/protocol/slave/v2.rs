use std::{collections::VecDeque, io::Cursor, time::Duration};

use async_trait::async_trait;
use crc::{self, Crc, CRC_16_UMTS};
use itertools::enumerate;
use log::debug;
use num_traits::FromPrimitive;
use tokio::{io::AsyncReadExt, time::timeout};
use tokio_serial::SerialStream;

use super::{AsyncProtocol, Opcode, RawInstruction};
use crate::protocol::{ProtocolVersion::V2, Result, ProtocolError};

pub struct ProtocolV2<'a> {
    port: &'a mut SerialStream,
    deq: VecDeque<u8>,
    buf: [u8; 65536],
}

impl<'a> ProtocolV2<'a> {
    pub fn new(port: &'a mut SerialStream) -> Self {
        Self {
            port,
            deq: VecDeque::new(),
            buf: [0u8; 65536],
        }
    }

    async fn ensure_buffer(&mut self, n: usize) -> Result<()> {
        if self.deq.len() >= n {
            return Ok(());
        }
        let to_read = n - self.deq.len();
        let buf = &mut self.buf[0..to_read];

        let res = timeout(Duration::from_millis(100), self.port.read(buf)).await;

        match res {
            Ok(Ok(bytes_read)) if bytes_read == to_read => {
                debug!("read {} bytes: {:02x?}", to_read, buf);
                self.deq.extend(buf.iter());
                Ok(())
            }
            _ => {
                self.deq.clear();
                Err(ProtocolError::TimedOut.into())
            }
        }
    }
}

#[async_trait]
impl<'a> AsyncProtocol for ProtocolV2<'a> {
    async fn recv_instruction(&mut self) -> Result<RawInstruction> {
        loop {
            while self.ensure_buffer(7).await.is_err() {}
            debug!("recv loop start");

            if self.deq[0] != 0xFF {
                self.deq.pop_front();
                continue;
            }
            debug!("got FF (1)");

            if self.deq[1] != 0xFF {
                self.deq.pop_front();
                continue;
            }
            debug!("got FF (2)");

            if self.deq[2] != 0xFD {
                self.deq.pop_front();
                continue;
            }
            debug!("got FD");

            if self.deq[3] != 0x00 {
                self.deq.pop_front();
                continue;
            }
            debug!("got 00");

            let id = self.deq[4];
            if id == 0xFF {
                debug!("bad id");
                self.deq.pop_front();
                continue;
            }
            debug!("got id {id:02}");

            let len = self.deq[5] as usize + ((self.deq[6] as usize) << 8);
            if len == 0x00 {
                debug!("bad len");
                self.deq.pop_front();
                continue;
            }
            debug!("got len {len:02}");

            if self.ensure_buffer(7 + len).await.is_err() {
                self.deq.clear();
                continue;
            }

            let opcode = Opcode::from_u8(self.deq[7]);
            if opcode.is_none() {
                debug!("bad opcode {}", self.deq[7]);
                self.deq.pop_front();
                continue;
            }
            let opcode = opcode.unwrap();

            let crc = Crc::<u16>::new(&CRC_16_UMTS);
            for (dst, src) in enumerate(self.deq.range(0..7 + len - 2)) {
                self.buf[dst] = *src;
            }
            let csum = crc.checksum(&self.buf[0..7 + len - 2]);

            debug!("csum={csum:02x}");
            if csum != self.deq[7 + len - 2] as u16 + ((self.deq[7 + len - 1] as u16) << 8) {
                debug!("bad checksum");
                self.deq.pop_front();
                continue;
            }

            if opcode == Opcode::StatusV2 {
                debug!("discarding status packet");
                self.deq.clear();
                continue;
            }

            let res = RawInstruction {
                version: V2,
                id,
                opcode,
                data: self.deq.range(8..(8 + len - 3)).copied().collect(),
            };
            self.deq.clear();

            return Ok(res);
        }
    }

    async fn send_status(&mut self, id: u8, status: u8, params: &[u8]) -> Result<()> {
        let end_pos = {
            use std::io::Write;

            let mut reply = Cursor::new(self.buf.as_mut_slice());
            let length: u16 = (4 + params.len()) as u16;

            reply.write_all(&[0xFF, 0xFF, 0xFD, 0x00])?;
            reply.write_all(&id.to_le_bytes())?;
            reply.write_all(&length.to_le_bytes())?;
            reply.write_all(&[0x55])?;
            reply.write_all(&status.to_le_bytes())?;

            reply.write_all(params)?;
            reply.position() as usize
        };

        let crc = Crc::<u16>::new(&CRC_16_UMTS);
        let csum = crc.checksum(&self.buf[0..end_pos]);

        self.buf[end_pos..end_pos + 2].copy_from_slice(&csum.to_le_bytes());

        {
            use tokio::io::AsyncWriteExt;
            debug!("dxl write: {:X?}", &self.buf[0..end_pos + 2]);
            self.port.write_all(&self.buf[0..end_pos + 2]).await?;
        }

        Ok(())
    }
}
