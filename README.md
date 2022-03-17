# Dynamixel tool &mdash; an unofficial Dynamixel protocol CLI
Reading and writing dynamixel device registers of any size via v1 or
v2 protocols. Built-in register descriptions for (almost) all released
servos/boards, including tab completion. Scripting friendly.

## Protocol reference
- https://emanual.robotis.com/docs/en/dxl/protocol1/
- https://emanual.robotis.com/docs/en/dxl/protocol2/

## Build
Both Linux and Windows platforms are supported. Linux builds depend on
`libudev` library and headers. Build command is
``` shell
cargo build --release
```

## Usage
### Common options
```
    -b, --baudrate <BAUDRATE>    UART baud rate [default: 57600]
    -d, --debug                  enable debug output
    -f, --force                  Skip sanity checks
    -h, --help                   Print help information
    -j, --json                   Use json-formatted output
    -p, --port <PORT>            UART device or 'auto' [default: auto]
    -P, --protocol <PROTOCOL>    Dynamixel protocol version [default: 1]
    -r, --retries <RETRIES>      Read/write retry count [default: 0]
    -V, --version                Print version information
```

### Querying information
List known device models
```
dynamixel-tool list-models
```

List known registers for a model
```
dynamixel-tool list-registers <MODEL>
```

### Scanning bus
Scanning bus for devices. `START`-`END` is optional device ID range.
```
dynamixel-tool scan [START [END]]
```

### Reading registers
Reading registers by address and size. `IDS` is the list of device
IDs. Examples are `1`, `3-5`, `1,3-5`.
```
dynamixel-tool read-uint<8|16|32> <IDS> <ADDRESS>
```

Reading registers by name:
```
dynamixel-tool read-reg <IDS> <MODEL/REGISTER>
```

### Writing registers
Writing registers by address and size.
```
dynamixel-tool read-uint<8|16|32> <IDS> <ADDRESS> <VALUE>
```

Writing registers by name:
```
dynamixel-tool read-reg <IDS> <MODEL/REGISTER> <VALUE>
```

## Misc
Bash completion script is available in [bash](bash).
