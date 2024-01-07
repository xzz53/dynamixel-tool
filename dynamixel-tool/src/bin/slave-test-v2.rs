use std::convert::TryInto;

use anyhow::Result;

use dynamixel_utils::port::{self};
use dynamixel_utils::protocol::slave::{make_async_protocol, Opcode};
use dynamixel_utils::protocol::ProtocolVersion;
use env_logger::TimestampPrecision;

#[tokio::main]
async fn main() -> Result<()> {
    let mut port = port::open_port_async("auto", 1000000, false)?;
    let mut proto = make_async_protocol(ProtocolVersion::V2, &mut port);
    let mut regs = [0u8; 65536];
    let my_id = 1u8;

    env_logger::Builder::from_env(env_logger::Env::default())
        .format_timestamp(Some(TimestampPrecision::Millis))
        .format_target(false)
        .init();

    loop {
        let r1 = proto.recv_instruction().await?;
        println!("{:?}", r1);
        if r1.id != my_id {
            println!("skipping id={}", r1.id);
            continue;
        }

        match r1.opcode {
            Opcode::Ping => {
                proto.send_status(my_id, 0, &[0, 0, 0]).await?;
            }
            Opcode::Read => {
                if r1.data.len() != 4 {
                    println!("error: malformed read");
                    continue;
                }
                let addr = u16::from_le_bytes(r1.data[0..2].try_into().unwrap()) as usize;
                let size = u16::from_le_bytes(r1.data[2..4].try_into().unwrap()) as usize;

                if addr + size > regs.len() {
                    println!("error: bad size ({size}) for address {addr}");
                    continue;
                }

                proto
                    .send_status(my_id, 0, &regs[addr..addr + size])
                    .await?;
            }
            Opcode::Write => {
                if r1.data.len() < 3 {
                    println!("error: malformed write");
                    continue;
                }
                let addr = u16::from_le_bytes(r1.data[0..2].try_into().unwrap()) as usize;
                let size = r1.data.len() - 2;

                if addr + size > regs.len() {
                    println!("error: bad size ({size}) for address {addr}");
                    continue;
                }

                regs[addr..addr + size].copy_from_slice(&r1.data[2..2 + size]);

                proto.send_status(my_id, 0, &[]).await?;
            }
            op => {
                println!("{op:?} not supported");
                proto.send_status(my_id, 1u8 << 6, &[]).await?;
            }
        }
    }
}
