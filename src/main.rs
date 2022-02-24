mod cli;
mod port;
mod protocol;
mod regs;

use anyhow::{anyhow, Context, Result};
use clap::CommandFactory;
use clap_complete::{generate, shells::Bash};
use log::error;
use regs::RegSpec;
use std::io;
use std::{convert::TryFrom, convert::TryInto, fmt::Display};

use cli::{Cli, StructOpt};
use protocol::{Protocol, ProtocolVersion};

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

fn slice_to_column<T>(data: &[T]) -> String
where
    T: Display,
{
    data.iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join("\n")
}

fn cmd_list_models(proto: ProtocolVersion, fmt: OutputFormat) -> Result<String> {
    let models = regs::list_models(proto);
    Ok(match fmt {
        OutputFormat::Plain => slice_to_column(models.as_slice()),
        OutputFormat::Json => json::stringify(models),
    })
}

fn cmd_list_registers(proto: ProtocolVersion, model: &str, _fmt: OutputFormat) -> Result<String> {
    let regs = regs::list_registers(proto, model);

    if !regs.is_empty() {
        Ok(slice_to_column(
            regs.iter()
                .map(|reg| reg.to_string())
                .collect::<Vec<_>>()
                .as_slice(),
        ))
    } else {
        Err(anyhow!("Model {} not found (protocol {})", model, proto))
    }
}

fn cmd_scan(
    proto: &mut dyn Protocol,
    scan_start: u8,
    scan_end: u8,
    fmt: OutputFormat,
) -> Result<String> {
    proto.scan(scan_start, scan_end).map(|ids| match fmt {
        OutputFormat::Plain => slice_to_column(&ids),
        OutputFormat::Json => json::stringify(ids),
    })
}

fn cmd_read_uint8(
    proto: &mut dyn Protocol,
    ids: &[u8],
    address: u16,
    fmt: OutputFormat,
) -> Result<String> {
    let res = ids
        .iter()
        .map(|&id| -> Result<u8> {
            Ok(proto
                .read(id, address, 1)
                .with_context(|| format!("Failed to read uint8 from id {}", id))?[0])
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
    proto: &mut dyn Protocol,
    ids: &[u8],
    address: u16,
    fmt: OutputFormat,
) -> Result<String> {
    let res = ids
        .iter()
        .map(|&id| -> Result<u16> {
            let bytes = proto
                .read(id, address, 2)
                .with_context(|| format!("Failed to read uint16 from id {}", id))?;
            Ok(u16::from_le_bytes(bytes.as_slice().try_into().unwrap()))
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
    proto: &mut dyn Protocol,
    ids: &[u8],
    address: u16,
    fmt: OutputFormat,
) -> Result<String> {
    let res = ids
        .iter()
        .map(|&id| -> Result<u32> {
            let bytes = proto
                .read(id, address, 4)
                .with_context(|| format!("Failed to read uint32 from id {}", id))?;
            Ok(u32::from_le_bytes(bytes.as_slice().try_into().unwrap()))
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
    proto: &mut dyn Protocol,
    ids: &[u8],
    address: u16,
    count: u16,
    fmt: OutputFormat,
) -> Result<String> {
    let res = ids
        .iter()
        .map(|&id| -> Result<Vec<u8>> {
            proto
                .read(id, address, count)
                .with_context(|| format!("Failed to read bytes from id {}", id))
        })
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

fn cmd_read_reg(
    proto: &mut dyn Protocol,
    ids: &[u8],
    regspec: RegSpec,
    fmt: OutputFormat,
) -> Result<String> {
    let reg = regs::find_register(proto.version(), regspec).ok_or(anyhow!("Register not found"))?;

    let res = ids
        .iter()
        .map(|&id| -> Result<u32> {
            let bytes: Vec<_> = proto
                .read(id, reg.address, reg.size as u16)
                .with_context(|| format!("Failed to read register from id {}", id))?;
            Ok(match reg.size {
                regs::RegSize::Byte => u8::from_le_bytes(bytes[0..=0].try_into().unwrap()) as u32,
                regs::RegSize::Half => u16::from_le_bytes(bytes[0..=1].try_into().unwrap()) as u32,
                regs::RegSize::Word => u32::from_le_bytes(bytes[0..=3].try_into().unwrap()),
            })
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

fn cmd_write_uint8(
    proto: &mut dyn Protocol,
    ids: &[u8],
    address: u16,
    value: u8,
) -> Result<String> {
    ids.iter()
        .map(|&id| {
            proto
                .write(id, address, &[value])
                .with_context(|| format!("Failed to write uint8 to id {}", id))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|_| Ok(String::new()))?
}

fn cmd_write_uint16(
    proto: &mut dyn Protocol,
    ids: &[u8],
    address: u16,
    value: u16,
) -> Result<String> {
    ids.iter()
        .map(|&id| {
            proto
                .write(id, address, &value.to_le_bytes())
                .with_context(|| format!("Failed to write uint16 to id {}", id))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|_| Ok(String::new()))?
}

fn cmd_write_uint32(
    proto: &mut dyn Protocol,
    ids: &[u8],
    address: u16,
    value: u32,
) -> Result<String> {
    ids.iter()
        .map(|&id| {
            proto
                .write(id, address, &value.to_le_bytes())
                .with_context(|| format!("Failed to write uint32 to id {}", id))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|_| Ok(String::new()))?
}

fn cmd_write_bytes(
    proto: &mut dyn Protocol,
    ids: &[u8],
    address: u16,
    values: &[u8],
) -> Result<String> {
    ids.iter()
        .map(|&id| {
            proto
                .write(id, address, values)
                .with_context(|| format!("Failed to write bytes to id {}", id))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|_| Ok(String::new()))?
}

fn cmd_write_reg(
    proto: &mut dyn Protocol,
    ids: &[u8],
    regspec: RegSpec,
    value: u32,
) -> Result<String> {
    let reg = regs::find_register(proto.version(), regspec).ok_or(anyhow!("Register not found"))?;

    ids.iter()
        .map(|&id| {
            match reg.size {
                regs::RegSize::Byte => {
                    proto.write(id, reg.address, &u8::try_from(value)?.to_le_bytes())
                }
                regs::RegSize::Half => {
                    proto.write(id, reg.address, &u16::try_from(value)?.to_le_bytes())
                }
                regs::RegSize::Word => proto.write(id, reg.address, &value.to_le_bytes()),
            }
            .with_context(|| format!("Failed to write register to id {}", id))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|_| Ok(String::new()))?
}

fn do_main() -> Result<String> {
    if std::env::var("GENERATE_COMPLETION").is_ok() {
        generate(
            Bash,
            &mut cli::Cli::command(),
            "dynamixel-tool",
            &mut io::stdout(),
        );

        return Ok(String::default());
    }

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

    let port = port::open_port(&cli.port, cli.baudrate, cli.force)?;

    let mut proto_box = protocol::make_protocol(cli.protocol, port, cli.retries);
    let proto = proto_box.as_mut();

    match cli.command {
        cli::Commands::ListModels => cmd_list_models(cli.protocol, fmt),
        cli::Commands::ListRegisters { model } => cmd_list_registers(cli.protocol, &model, fmt),
        cli::Commands::Scan {
            scan_start,
            scan_end,
        } => cmd_scan(proto, scan_start, scan_end, fmt),
        cli::Commands::ReadUint8 { ids, address } => cmd_read_uint8(proto, &ids, address, fmt),
        cli::Commands::ReadUint16 { ids, address } => cmd_read_uint16(proto, &ids, address, fmt),
        cli::Commands::ReadUint32 { ids, address } => cmd_read_uint32(proto, &ids, address, fmt),
        cli::Commands::ReadBytes {
            ids,
            address,
            count,
        } => cmd_read_bytes(proto, &ids, address, count, fmt),
        cli::Commands::ReadReg { ids, reg } => cmd_read_reg(proto, &ids, reg, fmt),
        cli::Commands::WriteUint8 {
            ids,
            address,
            value,
        } => cmd_write_uint8(proto, &ids, address, value),
        cli::Commands::WriteUint16 {
            ids,
            address,
            value,
        } => cmd_write_uint16(proto, &ids, address, value),
        cli::Commands::WriteUint32 {
            ids,
            address,
            value,
        } => cmd_write_uint32(proto, &ids, address, value),
        cli::Commands::WriteBytes {
            ids,
            address,
            values,
        } => cmd_write_bytes(proto, &ids, address, &values),
        cli::Commands::WriteReg { ids, reg, value } => cmd_write_reg(proto, &ids, reg, value),
    }
}

fn main() {
    match do_main() {
        Ok(s) => println!("{}", s),
        Err(e) => error!("{:#}", e),
    }
}
