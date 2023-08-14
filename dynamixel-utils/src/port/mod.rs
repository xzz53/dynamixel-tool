#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
use linux::{do_open_port, is_port_open};
#[cfg(target_os = "macos")]
use macos::{do_open_port, is_port_open};
#[cfg(target_os = "windows")]
use windows::{do_open_port, is_port_open};

use anyhow::Result;
use core::time::Duration;
use log::debug;
pub use serialport::SerialPort;
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

    let mut port = do_open_port(&true_name, baudrate)?;

    if true_name.contains("ttyS") && port.rs485_enable(true).is_err() && !force {
        return Err(OpenPortError::Rs485Error {
            port_name: true_name,
        }
        .into());
    }

    port.set_timeout(Duration::from_millis(10))?;

    debug!("open_port OK: {} @ {} baud", &true_name, baudrate);
    Ok(Box::new(port))
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
                    && match do_open_port(&info.port_name, 9600) {
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
