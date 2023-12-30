#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
use linux::is_port_open;
#[cfg(target_os = "macos")]
use macos::is_port_open;
use tokio_serial::SerialPortBuilderExt;
#[cfg(target_os = "windows")]
use windows::is_port_open;

pub use serialport::SerialPort;
pub use tokio_serial::SerialStream;

use anyhow::Result;
use core::time::Duration;
use log::debug;
use serialport::{self, SerialPortType};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpenPortError {
    #[error("no dynamixel compatible ports found")]
    NoCompatiblePort,
    #[error("{port_name:?} busy")]
    PortBusy { port_name: String },
    #[error("rs485 configuration failed on {port_name:?}")]
    Rs485Error { port_name: String },
}

trait Rs485 {
    fn rs485_is_enabled(&self) -> Result<bool>;
    fn rs485_enable(&self, enable: bool) -> Result<()>;

    fn rs485_is_supported(&self) -> bool {
        match self.rs485_is_enabled() {
            Ok(enabled) => self.rs485_enable(enabled).is_ok(),
            Err(_) => false,
        }
    }
}

#[derive(PartialEq)]
struct UsbId(u16, u16);

static COMPATIBLE_IDS: &[UsbId] = &[
    UsbId(0x16d0, 0x06a7), // MCS USB2AX
    UsbId(0x0403, 0x6014), // FTDI FT232H Single HS USB-UART/FIFO IC
    UsbId(0x1a86, 0x7523), // QinHeng Electronics HL-340 USB-Serial adapter
    UsbId(0x0483, 0x5740), // STMicroelectronics Virtual COM Port
];

pub fn open_port(
    port_name: &str,
    baudrate: u32,
    force: bool,
) -> Result<Box<dyn SerialPort + Send>> {
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

    let mut port = serialport::new(&true_name, baudrate).open_native()?;

    if port.rs485_is_supported() && port.rs485_enable(true).is_err() && !force {
        return Err(OpenPortError::Rs485Error {
            port_name: true_name,
        }
        .into());
    }

    port.set_timeout(Duration::from_millis(10))?;

    debug!("open_port OK: {} @ {} baud", &true_name, baudrate);
    Ok(Box::new(port))
}

pub fn open_port_async(port_name: &str, baudrate: u32, force: bool) -> Result<SerialStream> {
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

    let mut port = tokio_serial::new(&true_name, baudrate).open_native_async()?;

    if port.rs485_is_supported() && port.rs485_enable(true).is_err() && !force {
        return Err(OpenPortError::Rs485Error {
            port_name: true_name,
        }
        .into());
    }

    port.set_timeout(Duration::from_millis(10))?;

    debug!("open_port OK: {} @ {} baud", &true_name, baudrate);
    Ok(port)
}

fn guess_port() -> Result<String> {
    serialport::available_ports()?
        .into_iter()
        .filter(|info| match &info.port_type {
            SerialPortType::UsbPort(usb_info) => {
                COMPATIBLE_IDS.contains(&UsbId(usb_info.vid, usb_info.pid))
            }
            SerialPortType::Unknown => {
                !is_port_open(&info.port_name)
                    && match serialport::new(&info.port_name, 9600).open_native() {
                        Ok(p) => p.rs485_is_supported(),
                        Err(_) => false,
                    }
            }
            SerialPortType::PciPort | SerialPortType::BluetoothPort => false,
        })
        .map(|info| info.port_name)
        .next()
        .ok_or_else(|| OpenPortError::NoCompatiblePort.into())
}
