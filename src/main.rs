mod cli;
mod port;
mod protocol;

use anyhow::Result;
use log::error;
use nix::libc::EXIT_FAILURE;
use serialport::SerialPort;
use std::process;
use std::{convert::TryInto, fmt::Display};

use cli::{Cli, StructOpt};
use protocol::{Protocol, ProtocolV1, ProtocolV2};

enum OutputFormat {
    Plain,
    Json,
}

fn slice_to_line<T>(data: &[T]) -> String
where
    T: Display,
{
    data.iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(" ")
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
    ids: &[u8],
    address: u16,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    fmt: OutputFormat,
) -> Result<String> {
    let res = ids
        .iter()
        .map(|&id| {
            proto
                .read(port, retries, id, address, 1)
                .map(|bytes| bytes[0])
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(match fmt {
        OutputFormat::Plain => slice_to_line(res.as_slice()),
        OutputFormat::Json => {
            if res.len() > 1 {
                json::stringify(res)
            } else {
                res[0].to_string()
            }
        }
    })
}

fn cmd_read_uint16(
    ids: &[u8],
    address: u16,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    fmt: OutputFormat,
) -> Result<String> {
    let res = ids
        .iter()
        .map(|&id| {
            proto
                .read(port, retries, id, address, 2)
                .map(|bytes| u16::from_le_bytes(bytes.as_slice().try_into().unwrap()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(match fmt {
        OutputFormat::Plain => slice_to_line(res.as_slice()),
        OutputFormat::Json => {
            if res.len() > 1 {
                json::stringify(res)
            } else {
                res[0].to_string()
            }
        }
    })
}

fn cmd_read_uint32(
    ids: &[u8],
    address: u16,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    fmt: OutputFormat,
) -> Result<String> {
    let res = ids
        .iter()
        .map(|&id| {
            proto
                .read(port, retries, id, address, 4)
                .map(|bytes| u32::from_le_bytes(bytes.as_slice().try_into().unwrap()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(match fmt {
        OutputFormat::Plain => slice_to_line(res.as_slice()),
        OutputFormat::Json => {
            if res.len() > 1 {
                json::stringify(res)
            } else {
                res[0].to_string()
            }
        }
    })
}

fn cmd_read_bytes(
    ids: &[u8],
    address: u16,
    count: u16,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    fmt: OutputFormat,
) -> Result<String> {
    let res = ids
        .iter()
        .map(|&id| proto.read(port, retries, id, address, count))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(match fmt {
        OutputFormat::Plain => res
            .iter()
            .map(|x| slice_to_line(x.as_slice()))
            .collect::<Vec<String>>()
            .join("\n"),
        OutputFormat::Json => {
            if res.len() > 1 {
                json::stringify(res)
            } else {
                json::stringify(res[0].clone())
            }
        }
    })
}

fn cmd_write_uint8(
    ids: &[u8],
    address: u16,
    value: u8,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    _fmt: OutputFormat,
) -> Result<String> {
    ids.iter()
        .map(|&id| proto.write(port, retries, id, address, &[value]))
        .collect::<Result<Vec<_>, _>>()
        .map(|_| Ok(String::new()))?
}

fn cmd_write_uint16(
    ids: &[u8],
    address: u16,
    value: u16,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    _fmt: OutputFormat,
) -> Result<String> {
    ids.iter()
        .map(|&id| proto.write(port, retries, id, address, &value.to_le_bytes()))
        .collect::<Result<Vec<_>, _>>()
        .map(|_| Ok(String::new()))?
}

fn cmd_write_uint32(
    ids: &[u8],
    address: u16,
    value: u32,
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    _fmt: OutputFormat,
) -> Result<String> {
    ids.iter()
        .map(|&id| proto.write(port, retries, id, address, &value.to_le_bytes()))
        .collect::<Result<Vec<_>, _>>()
        .map(|_| Ok(String::new()))?
}

fn cmd_write_bytes(
    ids: &[u8],
    address: u16,
    values: &[u8],
    port: &mut dyn SerialPort,
    proto: &dyn Protocol,
    retries: usize,
    _fmt: OutputFormat,
) -> Result<String> {
    ids.iter()
        .map(|&id| proto.write(port, retries, id, address, values))
        .collect::<Result<Vec<_>, _>>()
        .map(|_| Ok(String::new()))?
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
        cli::Commands::ReadUint8 { ids, address } => cmd_read_uint8(
            ids.as_slice(),
            address,
            port.as_mut(),
            proto.as_ref(),
            retries,
            fmt,
        ),
        cli::Commands::ReadUint16 { ids, address } => cmd_read_uint16(
            ids.as_slice(),
            address,
            port.as_mut(),
            proto.as_ref(),
            retries,
            fmt,
        ),
        cli::Commands::ReadUint32 { ids, address } => cmd_read_uint32(
            ids.as_slice(),
            address,
            port.as_mut(),
            proto.as_ref(),
            retries,
            fmt,
        ),
        cli::Commands::ReadBytes {
            ids,
            address,
            count,
        } => cmd_read_bytes(
            ids.as_slice(),
            address,
            count,
            port.as_mut(),
            proto.as_ref(),
            retries,
            fmt,
        ),
        cli::Commands::WriteUint8 {
            ids,
            address,
            value,
        } => cmd_write_uint8(
            ids.as_slice(),
            address,
            value,
            port.as_mut(),
            proto.as_ref(),
            retries,
            fmt,
        ),
        cli::Commands::WriteUint16 {
            ids,
            address,
            value,
        } => cmd_write_uint16(
            ids.as_slice(),
            address,
            value,
            port.as_mut(),
            proto.as_ref(),
            retries,
            fmt,
        ),
        cli::Commands::WriteUint32 {
            ids,
            address,
            value,
        } => cmd_write_uint32(
            ids.as_slice(),
            address,
            value,
            port.as_mut(),
            proto.as_ref(),
            retries,
            fmt,
        ),
        cli::Commands::WriteBytes {
            ids,
            address,
            values,
        } => cmd_write_bytes(
            ids.as_slice(),
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
