mod cli;
mod port;
mod protocol;

use anyhow::Result;
use log::error;
use nix::libc::EXIT_FAILURE;
use serialport::SerialPort;
use std::convert::TryInto;
use std::process;

use cli::{Cli, StructOpt};
use protocol::{Protocol, ProtocolV1, ProtocolV2};

enum OutputFormat {
    Plain,
    Json,
}

fn cmd_scan(
    scan_start: u8,
    scan_end: u8,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    fmt: OutputFormat,
) -> Result<String> {
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
    id: u8,
    address: u16,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    _fmt: OutputFormat,
) -> Result<String> {
    proto
        .read(port, retries, id, address, 1)
        .map(|bytes| format!("{}", bytes[0]))
}

fn cmd_read_uint16(
    id: u8,
    address: u16,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    _fmt: OutputFormat,
) -> Result<String> {
    proto.read(port, retries, id, address, 2).map(|bytes| {
        format!(
            "{}",
            u16::from_le_bytes(bytes.as_slice().try_into().unwrap())
        )
    })
}

fn cmd_read_uint32(
    id: u8,
    address: u16,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    _fmt: OutputFormat,
) -> Result<String> {
    proto.read(port, retries, id, address, 4).map(|bytes| {
        format!(
            "{}",
            u32::from_le_bytes(bytes.as_slice().try_into().unwrap())
        )
    })
}

fn cmd_read_bytes(
    id: u8,
    address: u16,
    count: u16,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    fmt: OutputFormat,
) -> Result<String> {
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
    id: u8,
    address: u16,
    value: u8,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    _fmt: OutputFormat,
) -> Result<String> {
    proto
        .write(port, retries, id, address, &[value])
        .map(|_| String::new())
}

fn cmd_write_uint16(
    id: u8,
    address: u16,
    value: u16,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    _fmt: OutputFormat,
) -> Result<String> {
    proto
        .write(port, retries, id, address, &value.to_le_bytes())
        .map(|_| String::new())
}

fn cmd_write_uint32(
    id: u8,
    address: u16,
    value: u32,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    _fmt: OutputFormat,
) -> Result<String> {
    proto
        .write(port, retries, id, address, &value.to_le_bytes())
        .map(|_| String::new())
}

fn cmd_write_bytes(
    id: u8,
    address: u16,
    values: &[u8],
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    _fmt: OutputFormat,
) -> Result<String> {
    proto
        .write(port, retries, id, address, values)
        .map(|_| String::new())
}

fn main() {
    let cli = Cli::parse();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(if cli.debug {
        "debug"
    } else {
        "info"
    }))
    .format_timestamp(None)
    .format_target(false)
    .init();

    let fmt = if cli.json {
        OutputFormat::Json
    } else {
        OutputFormat::Plain
    };

    let force = cli.force;
    let baudrate = cli.baudrate;
    let retries = cli.retries;

    let proto: Box<dyn Protocol> = match cli.protocol.as_ref() {
        "1" => Box::new(ProtocolV1 {}),
        "2" => Box::new(ProtocolV2 {}),
        _ => {
            error!("unknown protocol {}", cli.protocol);
            process::exit(EXIT_FAILURE);
        }
    };

    let mut port = match port::open_port(&cli.port, baudrate, force) {
        Ok(port) => port,
        Err(e) => {
            error!("Can't open port '{}': {}", cli.port, e);
            process::exit(EXIT_FAILURE);
        }
    };

    // if let Some((name, sub_matches)) = matches.subcommand() {
    //     let cmd = cmds.get(name).unwrap();
    //     match cmd(sub_matches, port.as_mut(), proto.as_ref(), retries, fmt) {
    //         Ok(s) => println!("{}", s),
    //         Err(e) => error!("{}", e),
    //     }
    // }

    match match cli.command {
        cli::Commands::Scan {
            scan_start,
            scan_end,
        } => cmd_scan(
            scan_start,
            scan_end,
            port.as_mut(),
            proto.as_ref(),
            retries,
            fmt,
        ),
        cli::Commands::ReadUint8 { id, address } => {
            cmd_read_uint8(id, address, port.as_mut(), proto.as_ref(), retries, fmt)
        }
        cli::Commands::ReadUint16 { id, address } => {
            cmd_read_uint16(id, address, port.as_mut(), proto.as_ref(), retries, fmt)
        }
        cli::Commands::ReadUint32 { id, address } => {
            cmd_read_uint32(id, address, port.as_mut(), proto.as_ref(), retries, fmt)
        }
        cli::Commands::ReadBytes { id, address, count } => cmd_read_bytes(
            id,
            address,
            count,
            port.as_mut(),
            proto.as_ref(),
            retries,
            fmt,
        ),
        cli::Commands::WriteUint8 { id, address, value } => cmd_write_uint8(
            id,
            address,
            value,
            port.as_mut(),
            proto.as_ref(),
            retries,
            fmt,
        ),
        cli::Commands::WriteUint16 { id, address, value } => cmd_write_uint16(
            id,
            address,
            value,
            port.as_mut(),
            proto.as_ref(),
            retries,
            fmt,
        ),
        cli::Commands::WriteUint32 { id, address, value } => cmd_write_uint32(
            id,
            address,
            value,
            port.as_mut(),
            proto.as_ref(),
            retries,
            fmt,
        ),
        cli::Commands::WriteBytes {
            id,
            address,
            values,
        } => cmd_write_bytes(
            id,
            address,
            &values,
            port.as_mut(),
            proto.as_ref(),
            retries,
            fmt,
        ),
    } {
        Ok(s) => {
            println!("{}", s)
        }
        Err(e) => {
            error!("{}", e)
        }
    }
}
