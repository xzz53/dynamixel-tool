use anyhow::Result;
use core::time::Duration;
use nix::{ioctl_read_bad, ioctl_write_ptr_bad};
use serialport::{self, SerialPort};
use serialport::{SerialPortType, TTYPort};
use std::os::unix::io::AsRawFd;
use thiserror::Error;

use glob::glob;
use std::fs;

#[derive(Error, Debug)]
pub enum OpenPortError {
    #[error("no dynamixel compatible ports found")]
    NoCompatiblePort,
    #[error("{port_name:?} busy")]
    PortBusy { port_name: String },
    #[error("rs485 configuration failed on {port_name:?}")]
    Rs485Error { port_name: String },
}

pub fn open_port(port_name: &str, baudrate: u32, force: bool) -> Result<TTYPort> {
    let true_name: String = if port_name == "auto" {
        guess_port()?
    } else {
        port_name.to_string()
    };

    if !force && is_port_open(&true_name) {
        return Err(OpenPortError::PortBusy {
            port_name: true_name,
        }
        .into());
    }

    let mut port = do_open_port(&true_name, baudrate)?;

    if true_name.contains("ttyS") && port.rs485_enable(true).is_err() && !force {
        return Err(OpenPortError::Rs485Error {
            port_name: true_name,
        }
        .into());
    }

    port.set_timeout(Duration::from_millis(1))?;
    Ok(port)
}

#[derive(PartialEq)]
struct UsbId(u16, u16);

static COMPATIBLE_IDS: &'static [UsbId] = &[UsbId(0x16d0, 0x06a7)];

fn do_open_port(port_name: &str, baudrate: u32) -> Result<TTYPort> {
    Ok(serialport::new(port_name, baudrate).open_native()?)
}

fn guess_port() -> Result<String> {
    serialport::available_ports()?
        .into_iter()
        .filter(|info| match &info.port_type {
            SerialPortType::UsbPort(usb_info) => {
                COMPATIBLE_IDS.contains(&UsbId(usb_info.vid, usb_info.pid))
            }
            SerialPortType::Unknown => match do_open_port(&info.port_name, 9600) {
                Ok(p) => p.rs485_is_supported(),
                Err(_) => false,
            },
            SerialPortType::PciPort | SerialPortType::BluetoothPort => false,
        })
        .map(|info| info.port_name)
        .next()
        .ok_or(OpenPortError::NoCompatiblePort.into())
}

fn is_port_open(port_name: &str) -> bool {
    glob("/proc/[0-9]*/fd/*")
        .unwrap()
        .filter_map(|p| match p {
            Ok(path) => Some(path),
            Err(_) => None,
        })
        .filter_map(|path| match fs::read_link(path) {
            Ok(link) => Some(link),
            Err(_) => None,
        })
        .any(|link| link.to_str() == Some(port_name))
}

const SER_RS485_ENABLED: u32 = 1 << 0;

#[allow(non_camel_case_types)]
#[derive(Debug, Default)]
#[repr(C)]
pub struct serial_rs485 {
    flags: u32,
    delay_rts_before_send: u32,
    delay_rts_after_send: u32,
    padding: [u32; 5],
}

trait Rs485 {
    fn rs485_is_enabled(&self) -> Result<bool>;
    fn rs485_enable(&self, enable: bool) -> Result<()>;

    fn rs485_is_supported(&self) -> bool {
        match self.rs485_is_enabled() {
            Ok(enabled) => match self.rs485_enable(enabled) {
                Ok(_) => true,
                Err(_) => false,
            },
            Err(_) => false,
        }
    }
}

impl Rs485 for TTYPort {
    fn rs485_is_enabled(&self) -> Result<bool> {
        let mut rs485 = serial_rs485::default();
        match unsafe { ioctl::serial_rs485_get(self.as_raw_fd(), &mut rs485) } {
            Ok(_) => Ok(rs485.flags & SER_RS485_ENABLED != 0),
            Err(err) => Err(err.into()),
        }
    }

    fn rs485_enable(&self, enable: bool) -> Result<()> {
        let mut rs485 = serial_rs485::default();
        if enable {
            rs485.flags |= SER_RS485_ENABLED;
        }
        match unsafe { ioctl::serial_rs485_set(self.as_raw_fd(), &rs485) } {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }
}

mod ioctl {
    use super::*;

    const TIOCGRS485: u32 = 0x542E;
    const TIOCSRS485: u32 = 0x542F;

    ioctl_read_bad!(serial_rs485_get, TIOCGRS485, serial_rs485);
    ioctl_write_ptr_bad!(serial_rs485_set, TIOCSRS485, serial_rs485);
}
