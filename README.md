# Meshtastic MQTT Reticulum Bridge

A bridge that connects Meshtastic devices via MQTT to Reticulum networks, enabling interoperability between Meshtastic mesh networks and Reticulum-based networks.

I am looking to make this into it's own broker instead of connecting to an external MQTT server in the near future.
Currently I am testing all features. I am looking to expand on much more but I need to focus on making sure everything is functional before proceeding with adding more. 

A top priority is to establish that the bridge with reticulum and meshtastic is fully functional. I didn't have a radio so I wasn't able to test that feature.

It is able to send and recieve packets. Now I want to make sure it's serving it's intended purpose so I can build all the features around the bridge.

The scenerio I am envisioning for this app is this would be running as a "server" with a usb dongle attached for it to transmit radio. People will reach the server via mqtt from either reticulum or meshtastic via lora (or mqtt). This will serve as a gateway if people want to mesh but don't have the ability to yet from a lack of the specialized hardware. While people are looking to phase out legacy styles of communication this should serve as a transitionary piece of software that has baseline meshtastic and reticulum features (with more to add in the future).

Yes the majority of the code is vibe coded because I have a big picture I want to accomplish as one guy. This has saved me months of development to where I can focus on functionality of key features. This is far from the final project. I am actually at the stage where AI loses its effectiveness.

## Overview

This project provides a bridge between:
- **Meshtastic** - Open-source, off-grid mesh networking platform for LoRa devices
- **Reticulum** - Cryptographically secure, resilient mesh networking stack

The bridge translates messages between Meshtastic's MQTT protocol and Reticulum's packet format, allowing devices on both networks to communicate seamlessly.

## Features

- **Bidirectional Communication**: Messages flow both ways between Meshtastic and Reticulum networks
- **Protocol Translation**: Converts between Meshtastic protobuf messages and Reticulum packets
- **Multiple Interface Support**: Works with various Reticulum interfaces (TCP, UDP, Serial, KISS, etc.)
- **Configurable**: Easy configuration via JSON files
- **Secure**: Maintains Reticulum's end-to-end encryption

## Architecture

```
Meshtastic Devices (LoRa) ←→ Meshtastic MQTT Bridge ←→ Reticulum Network
  
```

## Installation

### Prerequisites
- Rust toolchain (latest stable)
- Mosquitto MQTT broker (or any MQTT 3.1.1 compatible broker) // Eventually (like tomorrow) it will be it's own mqtt broker
- Meshtastic device(s) configured to use MQTT and/or LoRa dongle.

### Build from Source
```bash
git clone <repository-url>
cd meshtastic_mqtt_reticulum
cargo build --release
```

## Configuration

1. Copy `config.example.json` to `config.json`
2. Edit the configuration:
   - Set MQTT broker connection details
   - Configure Reticulum interfaces
   - Adjust message routing rules

Example configuration:
```json
{
  "mqtt": {
    "host": "localhost",
    "port": 1883,
    "username": "",
    "password": "",
    "topic_prefix": "msh"
  },
  "reticulum": {
    "interfaces": [
      {
        "type": "tcp_client",
        "host": "192.168.1.100",
        "port": 4242
      }
    ]
  }
}
```

## Usage

### Running the Bridge
```bash
./target/release/meshtastic_mqtt_reticulum
```

Or use the provided launch script:
```bash
./launch.sh
```

### Testing
Test scripts are available in the `examples/` directory:
- `test_mqtt_publish.py` - Test MQTT publishing
- `test_reticulum_client.rs` - Test Reticulum connectivity

## Reticulum Interfaces Supported

The bridge supports multiple Reticulum interface types:

1. **TCP Client/Server**: Connect to remote Reticulum nodes over TCP
2. **UDP**: For local network communication
3. **Serial**: Direct serial connection to devices
4. **KISS**: For TNC (Terminal Node Controller) compatibility
5. **MQTT**: Direct MQTT interface (experimental)
6. **Kaonic**: For specialized hardware interfaces

## Message Flow

1. **Meshtastic → Reticulum**:
   - Meshtastic device sends message via LoRa
   - Message appears on MQTT broker
   - Bridge receives MQTT message
   - Bridge converts to Reticulum packet
   - Packet sent via configured Reticulum interfaces

2. **Reticulum → Meshtastic**:
   - Reticulum node sends packet
   - Bridge receives packet via interface
   - Bridge converts to Meshtastic protobuf
   - Bridge publishes to MQTT broker
   - Meshtastic devices receive via LoRa

## Security Considerations

- **Encryption**: Reticulum provides end-to-end encryption for all messages
- **Authentication**: MQTT broker should be secured with authentication
- **Network Isolation**: Consider firewall rules for exposed interfaces
- **Key Management**: Reticulum identity keys are stored in `~/.reticulum/`

## Troubleshooting

### Common Issues

1. **MQTT Connection Failed**:
   - Verify broker is running
   - Check credentials in config
   - Test with `mosquitto_sub -t '#' -v`

2. **Reticulum Interface Not Working**:
   - Check interface configuration
   - Verify network connectivity
   - Check firewall settings

3. **No Messages Flowing**:
   - Enable debug logging with `RUST_LOG=debug`
   - Check both MQTT and Reticulum connections
   - Verify topic subscriptions

### Logs
Logs are written to `meshtastic_bridge.log` by default. Enable verbose logging:
```bash
RUST_LOG=info,meshtastic_mqtt_reticulum=debug ./target/release/meshtastic_mqtt_reticulum
```

## Development

### Project Structure
```
src/
├── main.rs              # Main application entry point
├── mqtt.rs             # MQTT client implementation
├── reticulum.rs        # Reticulum interface management
├── message.rs          # Message conversion logic
└── config.rs           # Configuration handling

Reticulum-rs/           # Reticulum library fork
└── src/iface/          # Interface implementations
```

### Adding New Features
1. Create feature branch
2. Implement changes
3. Update tests
4. Update documentation
5. Submit pull request

## License

[License information to be added]

## Contributing

Contributions are welcome! Please see CONTRIBUTING.md for guidelines.

## Support

- GitHub Issues: For bug reports and feature requests
- Documentation: See the docs/ directory
- Community: [Link to community forum/chat]

## Acknowledgments

- **Meshtastic** team for the amazing open-source mesh networking platform
- **Reticulum** developers for the secure networking stack
- All contributors and testers
