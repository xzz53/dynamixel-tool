use anyhow::Result;

use dynamixel_lib::port::{self, SerialPort};
use dynamixel_lib::protocol::{self, ProtocolVersion};
use env_logger::TimestampPrecision;

fn main() -> Result<()> {
    let mut port: Box<dyn SerialPort + Send> = port::open_port("auto", 1000000, false)?;
    let mut proto_box = protocol::master::make_protocol(ProtocolVersion::V1, port.as_mut(), 0);

    env_logger::Builder::from_env(env_logger::Env::default())
        .format_timestamp(Some(TimestampPrecision::Millis))
        .format_target(false)
        .init();

    loop {
        proto_box.write(1, 20, &[0, 0])?;
        proto_box.write(1, 20, &[0, 0])?;
        proto_box.write(1, 20, &[0, 0])?;
        proto_box.write(1, 20, &[0, 0])?;
        proto_box.write(1, 20, &[0, 0])?;
        // std::thread::sleep_ms(10);
        let r1 = proto_box.read(51, 18, 2)?;
        println!("{:?}", r1);
    }
}
