# Direct LoRa Hardware Interface for Meshtastic Reticulum Bridge

This module provides a direct LoRa hardware interface for the Meshtastic Reticulum Bridge, allowing communication over LoRa radio frequencies without requiring internet connectivity.

## Features

- **Multiple Hardware Support**: Works with SPI-based LoRa chips (SX127x/SX126x), serial LoRa modules (RAK811, RAK3172), and USB LoRa dongles
- **Configurable Parameters**: Adjust frequency, spreading factor, bandwidth, coding rate, and transmission power
- **Async Interface**: Built on Tokio for non-blocking operations
- **Integration Ready**: Designed to work with the existing Reticulum bridge architecture
- **Extensible**: Easy to add support for new LoRa hardware

## Hardware Support

### 1. SPI LoRa Modules (SX127x/SX126x)
- **Chips**: SX1276, SX1277, SX1278, SX1279
- **Interface**: SPI with GPIO for chip select, reset, and interrupt pins
- **Typical Boards**: Dragino LoRa Shield, Adafruit Feather M0 with RFM95, Heltec LoRa32
- **Configuration**: Set via environment variables or code

### 2. Serial LoRa Modules
- **Modules**: RAK3172, RAK811, LoRa-E5
- **Interface**: Serial UART with AT commands
- **Configuration**: Baud rate, port path

### 3. USB LoRa Dongles
- **Devices**: RAKDAP1, other USB LoRa adapters
- **Interface**: USB CDC or custom protocol
- **Configuration**: Vendor ID, Product ID

## Installation

### With LoRa Support (Optional)

```bash
# Clone the repository
git clone <repository-url>
cd meshtastic_mqtt_reticulum

# Build with LoRa support (optional features)
cargo build --features lora

# Or build specific binaries
cargo build --bin lora_bridge --features lora
```

## Usage

### 1. Running the LoRa Bridge

```bash
# Enable LoRa and configure via environment variables
export LORA_ENABLED=true
export LORA_TYPE=serial  # or "spi" or "usb"
export LORA_FREQUENCY=915000000  # 915 MHz
export LORA_SERIAL_PORT=/dev/ttyUSB0
export LORA_SERIAL_BAUD=9600

# Run the LoRa-enabled bridge
cargo run --bin lora_bridge --features lora
```

### 2. Using the Example Program

```bash
# Run the example to test LoRa interface
cargo run --example lora_example --features lora
```

### 3. Programmatic Usage

```rust
use meshtastic_reticulum_bridge::lora_interface::{
    LoRaConfig, LoRaHardware, LoRaInterface, LoRaManager
};

// Create configuration
let config = LoRaConfig {
    frequency: 915_000_000,  // 915 MHz
    spreading_factor: 7,      // SF7
    bandwidth: 125_000,       // 125 kHz
    tx_power: 17,            // 17 dBm
    ..Default::default()
};

// Create hardware interface
let hardware = LoRaHardware::Serial {
    port: "/dev/ttyUSB0".to_string(),
    baud_rate: 9600,
};

// Create and initialize interface
let mut interface = LoRaInterface::new(config, hardware);
interface.initialize().await?;

// Send data
interface.send(b"Hello LoRa!").await?;

// Receive data
if let Some(data) = interface.receive().await? {
    println!("Received: {:?}", data);
}
```

## Configuration Parameters

### LoRa Configuration (`LoRaConfig`)

| Parameter | Description | Default | Range |
|-----------|-------------|---------|-------|
| `frequency` | Center frequency in Hz | 915,000,000 | Depends on region |
| `bandwidth` | Signal bandwidth in Hz | 125,000 | 7,800 - 500,000 |
| `spreading_factor` | Spreading factor (SF) | 7 | 6 - 12 |
| `coding_rate` | Coding rate (4/CR) | 5 | 5 - 8 |
| `tx_power` | Transmission power in dBm | 17 | -4 - 20 (SX127x) |
| `explicit_header` | Use explicit header mode | true | bool |
| `crc_enabled` | Enable CRC checking | true | bool |
| `preamble_length` | Preamble length in symbols | 8 | 6 - 65535 |
| `sync_word` | Sync word byte | 0x12 | 0x00 - 0xFF |

### Regional Frequencies

| Region | Frequency Bands | Common Frequencies |
|--------|----------------|-------------------|
| North America | 902-928 MHz | 915,000,000 Hz |
| Europe | 863-870 MHz | 868,000,000 Hz |
| Asia | 433 MHz, 470-510 MHz | 433,000,000 Hz |
| China | 470-510 MHz | 490,000,000 Hz |

## Integration with Reticulum

The LoRa interface can be integrated with the Reticulum protocol in several ways:

1. **Direct Integration**: LoRa packets are encapsulated in Reticulum packets
2. **Bridge Mode**: LoRa interface runs alongside existing TCP/UDP interfaces
3. **Gateway Mode**: Forward messages between LoRa and other Reticulum interfaces

### Example Integration Architecture

```
[LoRa Radio] <-> [LoRa Interface] <-> [Reticulum Transport] <-> [Internet/MQTT]
                     ↑
                [GUI/Config]
```

## Adding New Hardware Support

To add support for new LoRa hardware:

1. Create a new driver module in `src/`
2. Implement the hardware-specific initialization and communication
3. Add a new variant to the `LoRaHardware` enum
4. Update the `LoRaInterface` to handle the new hardware type

Example driver structure:
```rust
pub struct NewLoRaDriver {
    // Hardware-specific fields
}

impl NewLoRaDriver {
    pub async fn initialize(&mut self) -> Result<()> { /* ... */ }
    pub async fn send(&mut self, data: &[u8]) -> Result<()> { /* ... */ }
    pub async fn receive(&mut self) -> Result<Option<Vec<u8>>> { /* ... */ }
}
```

## Testing

### Unit Tests
```bash
cargo test --lib
```

### Hardware Tests
1. Connect LoRa hardware
2. Run the example program
3. Use a second LoRa device to send/receive test messages
4. Verify communication works at different ranges and configurations

## Troubleshooting

### Common Issues

1. **Permission denied on serial port/USB**
   ```bash
   sudo usermod -a -G dialout $USER
   sudo usermod -a -G tty $USER
   # Log out and back in
   ```

2. **SPI not enabled (Raspberry Pi)**
   ```bash
   sudo raspi-config
   # Interface Options -> SPI -> Enable
   ```

3. **Frequency out of range**
   - Check regional regulations
   - Verify hardware supports the frequency

4. **No communication**
   - Check wiring/connections
   - Verify both devices use same settings
   - Check antenna is connected

## Security Considerations

1. **Encryption**: LoRa packets are not encrypted by default. Use Reticulum's end-to-end encryption.
2. **Regulations**: Follow local regulations for transmission power and frequencies.
3. **Authentication**: Implement device authentication if needed.
4. **Rate Limiting**: Prevent spam with rate limiting.

## Future Enhancements

1. **Mesh Networking**: Implement LoRa mesh protocols
2. **GPS Integration**: Add location data to packets
3. **Power Management**: Optimize for battery-powered devices
4. **OTA Updates**: Firmware updates over LoRa
5. **Diagnostic Tools**: Signal strength, link quality, etc.

## License

[Same as main project]

## Contributing

1. Fork the repository
2. Create a feature branch
3. Implement your changes
4. Add tests
5. Submit a pull request