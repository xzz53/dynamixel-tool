mod port;
mod protocol;

use anyhow::Result;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use log::error;
use serialport::SerialPort;
use std::convert::TryInto;
use std::{collections::HashMap, process};

use protocol::{Protocol, ProtocolV1, ProtocolV2};

enum OutputFormat {
    Plain,
    Json,
}

fn cmd_scan(
    matches: &ArgMatches,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    fmt: OutputFormat,
) -> Result<String> {
    let scan_start: u8 = matches
        .value_of("scan_start")
        .and_then(|s| s.parse().ok())
        .unwrap();
    let scan_end: u8 = matches
        .value_of("scan_end")
        .and_then(|s| s.parse().ok())
        .unwrap();

    proto
        .scan(port, retries, scan_start, scan_end)
        .map(|ids| match fmt {
            OutputFormat::Plain => ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
            OutputFormat::Json => json::stringify(ids),
        })
}

fn cmd_read_uint8(
    matches: &ArgMatches,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    _fmt: OutputFormat,
) -> Result<String> {
    let id: u8 = matches.value_of("id").and_then(|s| s.parse().ok()).unwrap();
    let address: u16 = matches
        .value_of("address")
        .and_then(|s| s.parse().ok())
        .unwrap();

    proto
        .read(port, retries, id, address, 1)
        .map(|bytes| format!("{}", bytes[0]))
}

fn cmd_read_uint16(
    matches: &ArgMatches,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    _fmt: OutputFormat,
) -> Result<String> {
    let id: u8 = matches.value_of("id").and_then(|s| s.parse().ok()).unwrap();
    let address: u16 = matches
        .value_of("address")
        .and_then(|s| s.parse().ok())
        .unwrap();

    proto.read(port, retries, id, address, 2).map(|bytes| {
        format!(
            "{}",
            u16::from_le_bytes(bytes.as_slice().try_into().unwrap())
        )
    })
}

fn cmd_read_uint32(
    matches: &ArgMatches,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    _fmt: OutputFormat,
) -> Result<String> {
    let id: u8 = matches.value_of("id").and_then(|s| s.parse().ok()).unwrap();
    let address: u16 = matches
        .value_of("address")
        .and_then(|s| s.parse().ok())
        .unwrap();

    proto.read(port, retries, id, address, 4).map(|bytes| {
        format!(
            "{}",
            u32::from_le_bytes(bytes.as_slice().try_into().unwrap())
        )
    })
}

fn cmd_read_bytes(
    matches: &ArgMatches,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    fmt: OutputFormat,
) -> Result<String> {
    let id: u8 = matches.value_of("id").and_then(|s| s.parse().ok()).unwrap();
    let address: u16 = matches
        .value_of("address")
        .and_then(|s| s.parse().ok())
        .unwrap();
    let count: u16 = matches
        .value_of("count")
        .and_then(|s| s.parse().ok())
        .unwrap();
    proto
        .read(port, retries, id, address, count)
        .map(|ids| match fmt {
            OutputFormat::Plain => ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
            OutputFormat::Json => json::stringify(ids),
        })
}

fn cmd_write_uint8(
    matches: &ArgMatches,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    _fmt: OutputFormat,
) -> Result<String> {
    let id: u8 = matches.value_of("id").and_then(|s| s.parse().ok()).unwrap();
    let address: u16 = matches
        .value_of("address")
        .and_then(|s| s.parse().ok())
        .unwrap();
    let value: u8 = matches
        .value_of("value")
        .and_then(|s| s.parse().ok())
        .unwrap();

    proto
        .write(port, retries, id, address, &[value])
        .map(|_| String::new())
}

fn cmd_write_uint16(
    matches: &ArgMatches,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    _fmt: OutputFormat,
) -> Result<String> {
    let id: u8 = matches.value_of("id").and_then(|s| s.parse().ok()).unwrap();
    let address: u16 = matches
        .value_of("address")
        .and_then(|s| s.parse().ok())
        .unwrap();
    let value: u16 = matches
        .value_of("value")
        .and_then(|s| s.parse().ok())
        .unwrap();

    proto
        .write(port, retries, id, address, &value.to_le_bytes())
        .map(|_| String::new())
}

fn cmd_write_uint32(
    matches: &ArgMatches,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    _fmt: OutputFormat,
) -> Result<String> {
    let id: u8 = matches.value_of("id").and_then(|s| s.parse().ok()).unwrap();
    let address: u16 = matches
        .value_of("address")
        .and_then(|s| s.parse().ok())
        .unwrap();
    let value: u32 = matches
        .value_of("value")
        .and_then(|s| s.parse().ok())
        .unwrap();

    proto
        .write(port, retries, id, address, &value.to_le_bytes())
        .map(|_| String::new())
}

fn cmd_write_bytes(
    matches: &ArgMatches,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    _fmt: OutputFormat,
) -> Result<String> {
    let id: u8 = matches.value_of("id").and_then(|s| s.parse().ok()).unwrap();
    let address: u16 = matches
        .value_of("address")
        .and_then(|s| s.parse().ok())
        .unwrap();
    let values: Vec<u8> = matches
        .values_of("values")
        .unwrap()
        .map(|s| s.parse::<u8>().unwrap())
        .collect();

    proto
        .write(port, retries, id, address, values.as_slice())
        .map(|_| String::new())
}

type Cmd =
    fn(&ArgMatches, &mut dyn SerialPort, &dyn Protocol, usize, OutputFormat) -> Result<String>;

fn main() {
    let mut cmds: HashMap<&str, Cmd> = HashMap::new();

    cmds.insert("scan", cmd_scan);
    cmds.insert("read-uint8", cmd_read_uint8);
    cmds.insert("read-uint16", cmd_read_uint16);
    cmds.insert("read-uint32", cmd_read_uint32);
    cmds.insert("read-bytes", cmd_read_bytes);
    cmds.insert("write-uint8", cmd_write_uint8);
    cmds.insert("write-uint16", cmd_write_uint16);
    cmds.insert("write-uint32", cmd_write_uint32);
    cmds.insert("write-bytes", cmd_write_bytes);

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
        .arg(
            Arg::with_name("protocol")
                .short("P")
                .long("protocol")
                .default_value("1")
                .help("Dynamixel protocol version"),
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
                .visible_alias("readb")
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
                .visible_alias("readh")
                .about("Read unsigned 16-bit integer")
                .arg(Arg::with_name("id").required(true).help("Servo id"))
                .arg(
                    Arg::with_name("address")
                        .required(true)
                        .help("Register address"),
                ),
        )
        .subcommand(
            SubCommand::with_name("read-uint32")
                .visible_alias("readw")
                .about("Read unsigned 32-bit integer")
                .arg(Arg::with_name("id").required(true).help("Servo id"))
                .arg(
                    Arg::with_name("address")
                        .required(true)
                        .help("Register address"),
                ),
        )
        .subcommand(
            SubCommand::with_name("read-bytes")
                .visible_alias("reada")
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
                .visible_alias("writeb")
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
                .visible_alias("writeh")
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
            SubCommand::with_name("write-uint32")
                .visible_alias("writew")
                .about("Write unsigned 32-bit integer")
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
                .visible_alias("writea")
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

    let fmt = if matches.is_present("json") {
        OutputFormat::Json
    } else {
        OutputFormat::Plain
    };

    let force = matches.is_present("force");

    let port_name = matches.value_of("port").unwrap();

    let baudrate: u32 = matches
        .value_of("baudrate")
        .and_then(|s| s.parse().ok())
        .expect("bad baudrate");

    let retries: usize = matches
        .value_of("retries")
        .and_then(|s| s.parse().ok())
        .expect("bad retries");

    let proto = matches
        .value_of("protocol")
        .and_then(|s| -> Option<Box<dyn Protocol>> {
            match s {
                "1" => Some(Box::new(ProtocolV1 {})),
                "2" => Some(Box::new(ProtocolV2 {})),
                _ => None,
            }
        })
        .expect("bad protocol");

    let mut port = match port::open_port(port_name, baudrate, force) {
        Ok(port) => port,
        Err(e) => {
            error!("Can't open port '{}': {}", port_name, e);
            process::exit(1);
        }
    };

    if let (name, Some(sub_matches)) = matches.subcommand() {
        let cmd = cmds.get(name).unwrap();
        match cmd(sub_matches, port.as_mut(), proto.as_ref(), retries, fmt) {
            Ok(s) => println!("{}", s),
            Err(e) => error!("{}", e),
        }
    }
}
