mod dxl;
mod port;

use clap::{App, AppSettings, Arg, SubCommand};
use env_logger;
use log::error;

use std::process;

use json;

use crate::dxl::{read, scan, write};
use crate::port::open_port;

fn main() {
    let matches = App::new("Dynamixel test tool")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .about("Debug and configure dynamixel servos")
        .arg(
            Arg::with_name("force")
                .short("f")
                .long("force")
                .help("skip sanity checks"),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("Enable debug output"),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .default_value("auto")
                .help("UART device or 'auto'"),
        )
        .arg(
            Arg::with_name("baudrate")
                .short("b")
                .long("baudrate")
                .default_value("1000000")
                .help("UART baud rate"),
        )
        .arg(
            Arg::with_name("retries")
                .short("r")
                .long("retries")
                .default_value("0")
                .help("Read/write retry count"),
        )
        .arg(
            Arg::with_name("json")
                .short("j")
                .long("json")
                .help("Use json-formatted output"),
        )
        .subcommand(
            SubCommand::with_name("scan")
                .about("Scan for servos")
                .arg(
                    Arg::with_name("scan_start")
                        .default_value("0")
                        .help("Minimal ID for scanning"),
                )
                .arg(
                    Arg::with_name("scan_end")
                        .default_value("253")
                        .help("Maximal ID for scanning"),
                ),
        )
        .subcommand(
            SubCommand::with_name("read-uint8")
                .about("Read unsigned 8-bit integer")
                .arg(Arg::with_name("id").required(true).help("Servo id"))
                .arg(
                    Arg::with_name("address")
                        .required(true)
                        .help("Register address"),
                ),
        )
        .subcommand(
            SubCommand::with_name("read-uint16")
                .about("Read unsigned 16-bit integer")
                .arg(Arg::with_name("id").required(true).help("Servo id"))
                .arg(
                    Arg::with_name("address")
                        .required(true)
                        .help("Register address"),
                ),
        )
        .subcommand(
            SubCommand::with_name("read-bytes")
                .about("Read byte array")
                .arg(Arg::with_name("id").required(true).help("Servo id"))
                .arg(
                    Arg::with_name("address")
                        .required(true)
                        .help("Register address"),
                )
                .arg(Arg::with_name("count").required(true).help("Byte count")),
        )
        .subcommand(
            SubCommand::with_name("write-uint8")
                .about("Write unsigned 8-bit integer")
                .arg(Arg::with_name("id").required(true).help("Servo id"))
                .arg(
                    Arg::with_name("address")
                        .required(true)
                        .help("Register address"),
                )
                .arg(
                    Arg::with_name("value")
                        .required(true)
                        .help("Register value"),
                ),
        )
        .subcommand(
            SubCommand::with_name("write-uint16")
                .about("Write unsigned 16-bit integer")
                .arg(Arg::with_name("id").required(true).help("Servo id"))
                .arg(
                    Arg::with_name("address")
                        .required(true)
                        .help("Register address"),
                )
                .arg(
                    Arg::with_name("value")
                        .required(true)
                        .help("Register value"),
                ),
        )
        .subcommand(
            SubCommand::with_name("write-bytes")
                .about("Write byte array")
                .arg(Arg::with_name("id").required(true).help("Servo id"))
                .arg(
                    Arg::with_name("address")
                        .required(true)
                        .help("Register address"),
                )
                .arg(
                    Arg::with_name("values")
                        .required(true)
                        .multiple(true)
                        .help("Values to write"),
                ),
        )
        .get_matches();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(
        if matches.is_present("debug") {
            "debug"
        } else {
            "info"
        },
    ))
    .format_timestamp(None)
    .format_target(false)
    .init();

    let json = matches.is_present("json");
    let force = matches.is_present("force");
    let port_name = matches.value_of("port").unwrap();
    let baudrate: u32 = matches
        .value_of("baudrate")
        .and_then(|s| s.parse().ok())
        .unwrap();
    let retries: usize = matches
        .value_of("retries")
        .and_then(|s| s.parse().ok())
        .unwrap();

    let mut port = match open_port(port_name, baudrate, force) {
        Ok(port) => port,
        Err(e) => {
            error!("Can't open port '{}': {}", port_name, e);
            process::exit(1);
        }
    };

    match matches.subcommand() {
        ("scan", Some(sub_m)) => {
            let scan_start: u8 = sub_m
                .value_of("scan_start")
                .and_then(|s| s.parse().ok())
                .unwrap();
            let scan_end: u8 = sub_m
                .value_of("scan_end")
                .and_then(|s| s.parse().ok())
                .unwrap();

            let ids = match scan(&mut port, retries, scan_start, scan_end) {
                Ok(ids) => ids,
                Err(e) => {
                    error!("Scan error: {}", e);
                    process::exit(1);
                }
            };
            if !json {
                for id in ids {
                    println!("{}", id);
                }
            } else {
                println!("{}", json::stringify(ids));
            }
        }
        ("read-uint8", Some(sub_m)) => {
            let id: u8 = sub_m.value_of("id").and_then(|s| s.parse().ok()).unwrap();
            let address: u8 = sub_m
                .value_of("address")
                .and_then(|s| s.parse().ok())
                .unwrap();

            let bytes = match read(&mut port, retries, id, address, 1) {
                Ok(data) => data,
                Err(e) => {
                    error!("Read error: {}", e);
                    process::exit(1);
                }
            };
            println!("{}", bytes[0]);
        }
        ("read-uint16", Some(sub_m)) => {
            let id: u8 = sub_m.value_of("id").and_then(|s| s.parse().ok()).unwrap();
            let address: u8 = sub_m
                .value_of("address")
                .and_then(|s| s.parse().ok())
                .unwrap();

            let bytes = match read(&mut port, retries, id, address, 2) {
                Ok(data) => data,
                Err(e) => {
                    error!("Read error: {}", e);
                    process::exit(1);
                }
            };
            let val: u16 = 255 * (bytes[1] as u16) + (bytes[0] as u16);
            println!("{}", val);
        }
        ("read-bytes", Some(sub_m)) => {
            let id: u8 = sub_m.value_of("id").and_then(|s| s.parse().ok()).unwrap();
            let address: u8 = sub_m
                .value_of("address")
                .and_then(|s| s.parse().ok())
                .unwrap();
            let count: u8 = sub_m
                .value_of("count")
                .and_then(|s| s.parse().ok())
                .unwrap();
            let bytes = match read(&mut port, retries, id, address, count) {
                Ok(data) => data,
                Err(e) => {
                    error!("Read error: {}", e);
                    process::exit(1);
                }
            };
            if !json {
                for b in bytes {
                    println!("{}", b);
                }
            } else {
                println!("{}", json::stringify(bytes));
            }
        }
        ("write-uint8", Some(sub_m)) => {
            let id: u8 = sub_m.value_of("id").and_then(|s| s.parse().ok()).unwrap();
            let address: u8 = sub_m
                .value_of("address")
                .and_then(|s| s.parse().ok())
                .unwrap();
            let value: u8 = sub_m
                .value_of("value")
                .and_then(|s| s.parse().ok())
                .unwrap();

            match write(&mut port, retries, id, address, &[value]) {
                Ok(_) => (),
                Err(e) => {
                    error!("Write error: {}", e);
                    process::exit(1);
                }
            }
        }
        ("write-uint16", Some(sub_m)) => {
            let id: u8 = sub_m.value_of("id").and_then(|s| s.parse().ok()).unwrap();
            let address: u8 = sub_m
                .value_of("address")
                .and_then(|s| s.parse().ok())
                .unwrap();
            let value: u16 = sub_m
                .value_of("value")
                .and_then(|s| s.parse().ok())
                .unwrap();
            let lo = (value & 0xff) as u8;
            let hi = ((value >> 8) & 0xff) as u8;

            match write(&mut port, retries, id, address, &[lo, hi]) {
                Ok(_) => (),
                Err(e) => {
                    error!("Write error: {}", e);
                    process::exit(1);
                }
            }
        }
        ("write-bytes", Some(sub_m)) => {
            let id: u8 = sub_m.value_of("id").and_then(|s| s.parse().ok()).unwrap();
            let address: u8 = sub_m
                .value_of("address")
                .and_then(|s| s.parse().ok())
                .unwrap();
            let values: Vec<u8> = sub_m
                .values_of("values")
                .unwrap()
                .map(|s| match s.parse::<u8>() {
                    Ok(num) => num,
                    Err(_) => {
                        error!("Bad byte: '{}'", s);
                        process::exit(1);
                    }
                })
                .collect();
            match write(&mut port, retries, id, address, values.as_slice()) {
                Ok(_) => (),
                Err(e) => {
                    error!("Write error: {}", e);
                    process::exit(1);
                }
            }
        }
        _ => (),
    }
}
