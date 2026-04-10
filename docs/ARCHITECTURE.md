# Architecture Documentation

## Overview

The Meshtastic MQTT Reticulum Bridge is designed to facilitate communication between Meshtastic LoRa mesh networks and Reticulum-based networks. The bridge acts as a protocol translator, converting messages between the two different networking stacks.

## System Architecture

### High-Level Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│                 │    │                 │    │                 │
│  Meshtastic     │    │      MQTT       │    │   Reticulum     │
│    Devices      │◄──►│     Broker      │◄──►│     Bridge      │
│                 │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                         │
                                                         ▼
                                                ┌─────────────────┐
                                                │                 │
                                                │   Reticulum     │
                                                │    Network      │
                                                │                 │
                                                └─────────────────┘
```

### Component Details

#### 1. Meshtastic Devices
- LoRa-based mesh networking devices
- Communicate via Meshtastic protocol
- Connect to MQTT broker via WiFi or other backhaul

#### 2. MQTT Broker
- Message broker implementing MQTT 3.1.1/5.0
- Receives messages from Meshtastic devices
- Distributes messages to subscribed clients
- Typically Mosquitto or similar

#### 3. Reticulum Bridge
- Core application that performs protocol translation
- Consists of multiple subcomponents:
  - MQTT Client: Connects to MQTT broker
  - Reticulum Interface Manager: Manages Reticulum connections
  - Message Converter: Translates between protocols
  - Configuration Manager: Handles settings

#### 4. Reticulum Network
- Cryptographically secure mesh network
- Supports multiple transport types (TCP, UDP, Serial, etc.)
- Provides end-to-end encryption

## Message Flow

### Meshtastic → Reticulum

1. **LoRa Transmission**: Meshtastic device sends message via LoRa
2. **MQTT Publication**: Message appears on MQTT broker under `msh/<region>/<node_id>/<topic>`
3. **Bridge Reception**: Bridge MQTT client subscribes and receives message
4. **Protocol Conversion**: Message converted from Meshtastic protobuf to Reticulum packet
5. **Reticulum Transmission**: Packet sent via configured Reticulum interfaces
6. **Network Propagation**: Packet propagates through Reticulum network

### Reticulum → Meshtastic

1. **Reticulum Reception**: Bridge receives Reticulum packet via interface
2. **Protocol Conversion**: Packet converted to Meshtastic protobuf format
3. **MQTT Publication**: Converted message published to MQTT broker
4. **Device Reception**: Meshtastic devices receive via MQTT subscription
5. **LoRa Transmission**: Devices may rebroadcast via LoRa if configured

## Data Formats

### Meshtastic Messages
- Protocol Buffer format
- Common message types:
  - `TextMessage`: Text content
  - `Position`: GPS coordinates
  - `Telemetry`: Sensor data
  - `NodeInfo`: Device information
  - `Waypoint`: Navigation points

### Reticulum Packets
- Binary format with header and payload
- Fields:
  - Destination hash
  - Source hash
  - Context ID
  - Flags
  - Payload (encrypted)
- Uses HDLC framing for serial interfaces

## Interface Types

### Supported Reticulum Interfaces

#### 1. TCP Client
- Connects to remote Reticulum node
- Configuration: `host`, `port`
- Use case: Connecting to public Reticulum gateways

#### 2. TCP Server
- Listens for incoming connections
- Configuration: `listen_host`, `listen_port`
- Use case: Allowing other nodes to connect

#### 3. UDP
- Connectionless communication
- Configuration: `listen_host`, `listen_port`, `connect_host`, `connect_port`
- Use case: Local network communication

#### 4. Serial
- Direct serial connection
- Configuration: `port`, `baud_rate`
- Use case: Connecting to serial devices

#### 5. KISS (Keep It Simple, Stupid)
- TNC (Terminal Node Controller) protocol
- Configuration: `port`, `baud_rate`
- Use case: Amateur radio interfaces

#### 6. MQTT (Experimental)
- MQTT-based interface
- Configuration: `host`, `port`, `topic`
- Use case: Cloud-based Reticulum networks

#### 7. Kaonic
- Specialized hardware interface
- Configuration: Varies by hardware
- Use case: Custom hardware integration

## Configuration Management

### Configuration Files
- Primary: `config.json`
- Example: `config.example.json`
- Format: JSON with nested structure

### Configuration Sections

#### MQTT Section
```json
"mqtt": {
  "host": "mqtt.meshtastic.org",
  "port": 1883,
  "username": "meshdev",
  "password": "large4cats",
  "topic_prefix": "msh"
}
```

#### Reticulum Section
```json
"reticulum": {
  "interfaces": [
    {
      "type": "tcp_client",
      "host": "RNS.MichMesh.net",
      "port": 7822,
      "enabled": true
    }
  ]
}
```

#### Bridge Section
```json
"bridge": {
  "log_level": "info",
  "log_file": "meshtastic_bridge.log",
  "message_timeout_seconds": 30
}
```

## Security Considerations

### Encryption
- **Reticulum**: End-to-end encryption using Fernet symmetric encryption
- **MQTT**: TLS support for broker connections
- **Meshtastic**: Optional encryption at application layer

### Authentication
- MQTT: Username/password authentication
- Reticulum: Cryptographic identity verification
- Interface-specific authentication where applicable

### Network Security
- Firewall configuration for exposed interfaces
- Rate limiting to prevent abuse
- Message validation and sanitization

## Performance Considerations

### Message Processing
- Asynchronous I/O using Tokio runtime
- Concurrent message handling
- Connection pooling for MQTT

### Memory Management
- Fixed-size buffers for packet processing
- Message caching with size limits
- Connection state management

### Network Efficiency
- Message compression (future)
- Batch processing where possible
- Adaptive rate limiting

## Error Handling

### Connection Errors
- Automatic reconnection for MQTT and Reticulum interfaces
- Exponential backoff for repeated failures
- Graceful degradation when interfaces fail

### Message Errors
- Invalid message detection and logging
- Malformed packet rejection
- Checksum verification

### System Errors
- Resource exhaustion handling
- File system error recovery
- Signal handling for graceful shutdown

## Monitoring and Logging

### Log Levels
- `error`: Critical failures
- `warn`: Non-critical issues
- `info`: Operational information
- `debug`: Detailed debugging
- `trace`: Verbose tracing

### Log Output
- Console output with colored formatting
- File logging with rotation
- Structured logging for machine processing

### Metrics
- Message counters (in/out)
- Connection statistics
- Error rates
- Performance metrics

## Deployment Considerations

### Single Instance
- Simple deployment for testing
- All components on one machine
- Suitable for personal use

### Distributed Deployment
- Multiple bridge instances for redundancy
- Load balancing across instances
- Geographic distribution

### Containerization
- Docker support (future)
- Kubernetes deployment (future)
- Resource isolation

## Future Enhancements

### Planned Features
1. **Web Interface**: Administrative web UI
2. **Plugin System**: Extensible architecture
3. **Advanced Routing**: Smart message routing
4. **Protocol Extensions**: Additional message types
5. **Monitoring API**: REST API for monitoring

### Integration Points
1. **Home Assistant**: Smart home integration
2. **Grafana**: Metrics visualization
3. **Prometheus**: Metrics collection
4. **Slack/Discord**: Notification integration

## Related Documentation

- [Configuration Guide](../CONFIGURATION_GUIDE.md)
- [Security Guide](../SECURITY_GUIDE.md)
- [Platform Compatibility Guide](../PLATFORM_COMPATIBILITY_GUIDE.md)
- [Device Compatibility Analysis](../DEVICE_COMPATIBILITY_ANALYSIS.md)