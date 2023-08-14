use super::Rs485;

use anyhow::Result;
use glob::glob;
use nix::{ioctl_read_bad, ioctl_write_ptr_bad};
use serialport::TTYPort as NativePort;
use std::fs;
use std::os::unix::io::AsRawFd;

pub fn do_open_port(port_name: &str, baudrate: u32) -> Result<NativePort> {
    Ok(serialport::new(port_name, baudrate).open_native()?)
}

pub fn is_port_open(port_name: &str) -> bool {
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

impl Rs485 for NativePort {
    fn rs485_is_enabled(&self) -> Result<bool> {
        let mut rs485 = ioctl::serial_rs485::default();
        match unsafe { ioctl::serial_rs485_get(self.as_raw_fd(), &mut rs485) } {
            Ok(_) => Ok(rs485.flags & ioctl::SER_RS485_ENABLED != 0),
            Err(err) => Err(err.into()),
        }
    }

    fn rs485_enable(&self, enable: bool) -> Result<()> {
        let mut rs485 = ioctl::serial_rs485::default();
        if enable {
            rs485.flags |= ioctl::SER_RS485_ENABLED | ioctl::SER_RS485_RTS_ON_SEND;
        }
        match unsafe { ioctl::serial_rs485_set(self.as_raw_fd(), &rs485) } {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }
}

#[allow(dead_code)]
mod ioctl {
    use super::*;
    pub const SER_RS485_ENABLED: u32 = 1 << 0;
    pub const SER_RS485_RTS_ON_SEND: u32 = 1 << 1;
    pub const SER_RS485_RTS_AFTER_SEND: u32 = 1 << 2;
    pub const SER_RS485_RX_DURING_TX: u32 = 1 << 4;
    pub const SER_RS485_TERMINATE_BUS: u32 = 1 << 5;
    pub const SER_RS485_ADDRB: u32 = 1 << 6;
    pub const SER_RS485_ADDR_RECV: u32 = 1 << 7;
    pub const SER_RS485_ADDR_DEST: u32 = 1 << 8;

    #[allow(non_camel_case_types)]
    #[derive(Debug, Default)]
    #[repr(C)]
    pub struct serial_rs485 {
        pub flags: u32,
        delay_rts_before_send: u32,
        delay_rts_after_send: u32,
        padding: [u32; 5],
    }

    const TIOCGRS485: u32 = 0x542E;
    const TIOCSRS485: u32 = 0x542F;

    ioctl_read_bad!(serial_rs485_get, TIOCGRS485, serial_rs485);
    ioctl_write_ptr_bad!(serial_rs485_set, TIOCSRS485, serial_rs485);
}
