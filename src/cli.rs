pub use clap::StructOpt;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Skip sanity checks
    #[clap(long, short)]
    pub force: bool,

    /// enable debug output
    #[clap(long, short)]
    pub debug: bool,

    /// UART device or 'auto'
    #[clap(long, short, default_value = "auto")]
    pub port: String,

    /// UART baud rate
    #[clap(long, short, default_value_t = 57600)]
    pub baudrate: u32,

    /// Read/write retry count
    #[clap(long, short, default_value_t = 0)]
    pub retries: usize,

    /// Use json-formatted output
    #[clap(long, short)]
    pub json: bool,

    /// Dynamixel protocol version
    #[clap(long, short = 'P', default_value = "1")]
    pub protocol: String,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scan for servos
    Scan {
        #[clap(default_value_t = 0)]
        scan_start: u8,
        #[clap(default_value_t = 253)]
        scan_end: u8,
    },

    /// Read unsigned 8-bit integer
    #[clap(visible_alias = "readb")]
    ReadUint8 { id: u8, address: u16 },

    /// Read unsigned 16-bit integer
    #[clap(visible_alias = "readh")]
    ReadUint16 { id: u8, address: u16 },

    /// Read unsigned 32-bit integer
    #[clap(visible_alias = "readw")]
    ReadUint32 { id: u8, address: u16 },

    /// Read byte array
    #[clap(visible_alias = "reada")]
    ReadBytes { id: u8, address: u16, count: u16 },

    /// Write unsigned 8-bit integer
    #[clap(visible_alias = "writeb")]
    WriteUint8 { id: u8, address: u16, value: u8 },

    /// Write unsigned 16-bit integer
    #[clap(visible_alias = "writeh")]
    WriteUint16 { id: u8, address: u16, value: u16 },

    /// Write unsigned 32-bit integer
    #[clap(visible_alias = "writew")]
    WriteUint32 { id: u8, address: u16, value: u32 },

    /// Write byte array
    #[clap(visible_alias = "writea")]
    WriteBytes {
        id: u8,
        address: u16,
        #[clap(required = true)]
        values: Vec<u8>,
    },
}
