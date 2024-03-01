use std::{collections::VecDeque, io::Cursor, time::Duration};

use async_trait::async_trait;
use log::debug;
use num_traits::FromPrimitive;
use tokio::{io::AsyncReadExt, time::timeout};
use tokio_serial::SerialStream;

use super::{AsyncProtocol, Opcode, RawInstruction};
use crate::protocol::{ProtocolVersion::V1, Result, ProtocolError};

pub struct ProtocolV1<'a> {
    port: &'a mut SerialStream,
    deq: VecDeque<u8>,
    buf: [u8; 256],
}

impl<'a> ProtocolV1<'a> {
    pub fn new(port: &'a mut SerialStream) -> Self {
        Self {
            port,
            deq: VecDeque::new(),
            buf: [0u8; 256],
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
                debug!("ensure_buffer timeout");
                Err(ProtocolError::TimedOut.into())
            }
        }
    }
}

#[async_trait]
impl<'a> AsyncProtocol for ProtocolV1<'a> {
    async fn recv_instruction(&mut self) -> Result<RawInstruction> {
        loop {
            while self.ensure_buffer(4).await.is_err() {}
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

            let id = self.deq[2];
            if id == 0xFF {
                debug!("bad id");
                self.deq.pop_front();
                continue;
            }
            debug!("got id {id:02}, deq: {:?}", self.deq);

            let len = self.deq[3];
            if len == 0x00 {
                debug!("bad len");
                self.deq.pop_front();
                continue;
            }
            debug!("got len {len:02}");

            if self.ensure_buffer(4 + len as usize).await.is_err() {
                self.deq.clear();
                continue;
            }

            let opcode = Opcode::from_u8(self.deq[4]);
            if opcode.is_none() {
                debug!("bad opcode {}", self.deq[4]);
                self.deq.pop_front();
                continue;
            }
            let opcode = opcode.unwrap();

            let csum = !self
                .deq
                .range(2..5 + (len as usize - 1))
                .cloned()
                .fold(0u8, |x, y| x.overflowing_add(y).0);

            debug!("csum={csum}");
            if csum != 0 {
                debug!("bad checksum");
                self.deq.pop_front();
                continue;
            }

            if opcode == Opcode::StatusV1 {
                debug!("discarding status packet");
                self.deq.clear();
                continue;
            }

            let res = RawInstruction {
                version: V1,
                id,
                opcode,
                data: self
                    .deq
                    .range(5..(5 + (len as usize - 2)))
                    .copied()
                    .collect(),
            };

            debug!("full deq: {:?}", self.deq);
            self.deq.clear();

            return Ok(res);
        }
    }

    async fn send_status(&mut self, id: u8, status: u8, params: &[u8]) -> Result<()> {
        let end_pos = {
            use std::io::Write;

            let mut reply = Cursor::new(self.buf.as_mut_slice());
            let length: u8 = (2 + params.len()) as u8;

            reply.write_all(&[0xFF, 0xFF])?;
            reply.write_all(&id.to_le_bytes())?;
            reply.write_all(&length.to_le_bytes())?;
            reply.write_all(&status.to_le_bytes())?;

            reply.write_all(params)?;
            reply.position() as usize
        };

        let csum = !self.buf[2..end_pos]
            .iter()
            .cloned()
            .fold(0u8, |x, y| x.overflowing_add(y).0);

        self.buf[end_pos] = csum;
        {
            use tokio::io::AsyncWriteExt;
            debug!("dxl write: {:X?}", &self.buf[0..=end_pos]);
            self.port.write_all(&self.buf[0..=end_pos]).await?;
        }

        Ok(())
    }
}
