use super::{OpenPortError, Rs485};
use anyhow::Result;
use serialport::SerialPort;
use serialport::TTYPort as NativePort;

// TODO: implement proper check
pub fn is_port_open(_port_name: &str) -> bool {
    false
}

impl Rs485 for NativePort {
    fn rs485_is_enabled(&self) -> Result<bool> {
        Err(OpenPortError::Rs485Error {
            port_name: self.name().unwrap_or_default(),
        }
        .into())
    }

    fn rs485_enable(&self, _enable: bool) -> Result<()> {
        Err(OpenPortError::Rs485Error {
            port_name: self.name().unwrap_or_default(),
        }
        .into())
    }
}
