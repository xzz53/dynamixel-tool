pub mod cli;

use std::io;
use std::{convert::TryFrom, convert::TryInto, fmt::Display};

use anyhow::{anyhow, Context, Result};
use clap::CommandFactory;
use clap_complete::{generate, shells::Bash};
use log::error;
use num_traits::{FromBytes, ToBytes};

use dynamixel_lib::port;
use dynamixel_lib::protocol::{self, master::Protocol, ProtocolVersion};
use dynamixel_lib::regs::{self, RegSpec};

use cli::{Cli, MultiReadSpec, MultiWriteSpec, StructOpt};

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

fn slice_to_byte_slices<T: Copy>(slice: &[T]) -> Vec<&[u8]> {
    slice
        .iter()
        .map(|x| unsafe {
            std::slice::from_raw_parts((x as *const T) as *const u8, std::mem::size_of::<T>())
        })
        .collect()
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

fn cmd_read_int<const N: usize, T>(
    proto: &mut dyn Protocol,
    ids: &[u8],
    address: u16,
    fmt: OutputFormat,
    sync: bool,
) -> Result<String>
where
    T: Copy + Display + FromBytes<Bytes = [u8; N]>,
    T: Into<json::JsonValue>,
{
    let res = if !sync {
        ids.iter()
            .map(|&id| -> Result<T> {
                let bytes = proto.read(id, address, N as u16).with_context(|| {
                    format!(
                        "Failed to read {} from id {}",
                        std::any::type_name::<T>(),
                        id
                    )
                })?;
                Ok(T::from_le_bytes(bytes.as_slice().try_into().unwrap()))
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        proto
            .sync_read(ids, address, N as u16)?
            .into_iter()
            .map(|bytes| T::from_le_bytes(bytes[..N].try_into().unwrap()))
            .collect()
    };

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

fn cmd_read_bytes_multiple(
    proto: &mut dyn Protocol,
    specs: &[MultiReadSpec],
    fmt: OutputFormat,
) -> Result<String> {
    let res = specs
        .iter()
        .map(|spec| -> Result<Vec<u8>> {
            proto
                .read(spec.id, spec.address, spec.size)
                .with_context(|| format!("Failed to read bytes from id {}", spec.id))
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
                regs::RegSize::Variable => panic!("variable size registers not supported!"),
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

fn cmd_write_int<const N: usize, T: Copy + ToBytes<Bytes = [u8; N]>>(
    proto: &mut dyn Protocol,
    ids: &[u8],
    address: u16,
    values: &[T],
    sync: bool,
) -> Result<String> {
    if !sync {
        if values.len() != 1 {
            return Err(anyhow!("Multiple values supported in sync mode only"));
        }
        ids.iter()
            .map(|&id| {
                proto
                    .write(id, address, values[0].to_le_bytes().as_slice())
                    .with_context(|| {
                        format!(
                            "Failed to write {} to id {}",
                            std::any::type_name::<T>(),
                            id
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|_| Ok(String::new()))?
    } else {
        if values.len() != ids.len() && values.len() != 1 {
            return Err(anyhow!("Need {} values, got {}", ids.len(), values.len()));
        }

        if values.len() != 1 {
            proto
                .sync_write(ids, address, &slice_to_byte_slices(values))
                .map(|_| Ok(String::new()))?
        } else {
            proto
                .sync_write(
                    ids,
                    address,
                    &slice_to_byte_slices(&vec![values[0]; ids.len()]),
                )
                .map(|_| Ok(String::new()))?
        }
    }
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

fn cmd_write_bytes_multiple(proto: &mut dyn Protocol, specs: &[MultiWriteSpec]) -> Result<String> {
    specs
        .iter()
        .map(|spec| {
            proto
                .write(spec.id, spec.address, &spec.data)
                .with_context(|| format!("Failed to write bytes to id {}", spec.id))
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
                regs::RegSize::Variable => panic!("variable size registers not supported!"),
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

    match cli.command {
        cli::Commands::ListModels => cmd_list_models(cli.protocol, fmt),
        cli::Commands::ListRegisters { model } => cmd_list_registers(cli.protocol, &model, fmt),
        _ => {
            let mut port = port::open_port(&cli.port, cli.baudrate, cli.force)?;
            let mut proto_box =
                protocol::master::make_protocol(cli.protocol, port.as_mut(), cli.retries);
            let proto = proto_box.as_mut();

            match cli.command {
                cli::Commands::Scan {
                    scan_start,
                    scan_end,
                } => cmd_scan(proto, scan_start, scan_end, fmt),
                cli::Commands::ReadUint8 { ids, address, sync } => {
                    cmd_read_int::<1, u8>(proto, &ids, address, fmt, sync)
                }
                cli::Commands::ReadUint16 { ids, address, sync } => {
                    cmd_read_int::<2, u16>(proto, &ids, address, fmt, sync)
                }
                cli::Commands::ReadUint32 { ids, address, sync } => {
                    cmd_read_int::<4, u32>(proto, &ids, address, fmt, sync)
                }
                cli::Commands::ReadBytes {
                    ids,
                    address,
                    count,
                } => cmd_read_bytes(proto, &ids, address, count, fmt),
                cli::Commands::ReadBytesMultiple { specs } => {
                    cmd_read_bytes_multiple(proto, &specs, fmt)
                }
                cli::Commands::ReadReg { ids, reg } => cmd_read_reg(proto, &ids, reg, fmt),
                cli::Commands::WriteUint8 {
                    ids,
                    address,
                    value,
                    sync,
                } => cmd_write_int(proto, &ids, address, &value, sync),
                cli::Commands::WriteUint16 {
                    ids,
                    address,
                    value,
                    sync,
                } => cmd_write_int(proto, &ids, address, &value, sync),
                cli::Commands::WriteUint32 {
                    ids,
                    address,
                    value,
                    sync,
                } => cmd_write_int(proto, &ids, address, &value, sync),
                cli::Commands::WriteBytes {
                    ids,
                    address,
                    values,
                } => cmd_write_bytes(proto, &ids, address, &values),
                cli::Commands::WriteReg { ids, reg, value } => {
                    cmd_write_reg(proto, &ids, reg, value)
                }
                cli::Commands::WriteBytesMultiple { specs } => {
                    cmd_write_bytes_multiple(proto, &specs)
                }
                _ => Err(anyhow!("unexpected command (this is a bug!)")),
            }
        }
    }
}

fn main() {
    match do_main() {
        Ok(s) => println!("{}", s),
        Err(e) => error!("{:#}", e),
    }
}
