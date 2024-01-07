use anyhow::Result;

use dynamixel_lib::port::{self};
use dynamixel_lib::protocol::slave::{make_async_protocol, Opcode};
use dynamixel_lib::protocol::ProtocolVersion;
use env_logger::TimestampPrecision;

#[tokio::main]
async fn main() -> Result<()> {
    let mut port = port::open_port_async("auto", 1000000, false)?;
    let mut proto = make_async_protocol(ProtocolVersion::V1, &mut port);
    let mut regs = [0u8; 256];
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
                proto.send_status(my_id, 0, &[]).await?;
            }
            Opcode::Read => {
                if r1.data.len() != 2 {
                    println!("error: malformed read");
                    continue;
                }
                let addr = r1.data[0] as usize;
                let size = r1.data[1] as usize;

                if addr + size > regs.len() {
                    println!("error: bad size ({size}) for address {addr}");
                    continue;
                }

                proto
                    .send_status(my_id, 0, &regs[addr..addr + size])
                    .await?;
            }
            Opcode::Write => {
                if r1.data.len() < 2 {
                    println!("error: malformed write");
                    continue;
                }
                let addr = r1.data[0] as usize;
                let size = r1.data.len() - 1;

                if addr + size > regs.len() {
                    println!("error: bad size ({size}) for address {addr}");
                    continue;
                }

                regs[addr..addr + size].copy_from_slice(&r1.data[1..1 + size]);

                proto.send_status(my_id, 0, &[]).await?;
            }
            op => {
                println!("{op:?} not supported");
                proto.send_status(my_id, 1u8 << 6, &[]).await?;
            }
        }
    }
}
