# Quick Start Guide

This guide will help you get the Meshtastic MQTT Reticulum Bridge up and running quickly.

## Prerequisites

### Required Software
- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Cargo**: Comes with Rust installation
- **Git**: For cloning the repository
- **MQTT Broker**: Mosquitto or any MQTT 3.1.1+ broker

### Optional Dependencies
- **Meshtastic Device**: For testing with actual hardware
- **Reticulum Network Access**: Public gateway or local network

## Installation

### Step 1: Clone the Repository
```bash
git clone https://github.com/yourusername/meshtastic_mqtt_reticulum.git
cd meshtastic_mqtt_reticulum
```

### Step 2: Build the Project
```bash
cargo build --release
```

The built binaries will be in `target/release/`:
- `gateway`: Main bridge application
- `gui`: Graphical interface (optional)
- `relay`: Simple relay utility
- `lora_bridge`: LoRa bridge (if LoRa features enabled)

### Step 3: Configure the Bridge

1. Copy the example configuration:
   ```bash
   cp config.example.json config.json
   ```

2. Edit `config.json` with your settings:
   ```json
   {
     "mqtt": {
       "host": "your-mqtt-broker.com",
       "port": 1883,
       "username": "your-username",
       "password": "your-password",
       "topic_prefix": "msh"
     },
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
   }
   ```

### Step 4: Run the Bridge

#### Option A: Using the launch script (Linux/macOS)
```bash
./launch.sh
```

#### Option B: Direct execution
```bash
./target/release/gateway
```

#### Option C: With logging enabled
```bash
RUST_LOG=info ./target/release/gateway
```

## Configuration Details

### MQTT Configuration
- `host`: MQTT broker hostname or IP address
- `port`: MQTT broker port (default: 1883)
- `username`: MQTT username (if required)
- `password`: MQTT password (if required)
- `topic_prefix`: Meshtastic topic prefix (default: "msh")
- `use_tls`: Enable TLS encryption (default: false)

### Reticulum Configuration
- `interfaces`: Array of Reticulum interfaces to use
  - `type`: Interface type (tcp_client, udp, serial, kiss, mqtt, kaonic)
  - `host`: Hostname or IP for TCP/UDP interfaces
  - `port`: Port number for TCP/UDP interfaces
  - `enabled`: Whether the interface is active

### Bridge Configuration
- `log_level`: Logging level (error, warn, info, debug, trace)
- `log_file`: Path to log file
- `message_timeout_seconds`: Timeout for message processing

## Testing the Setup

### Test 1: Verify MQTT Connection
```bash
# Subscribe to Meshtastic topics
mosquitto_sub -h your-mqtt-broker.com -t "msh/#" -v
```

### Test 2: Send a Test Message
```bash
# Publish a test message
mosquitto_pub -h your-mqtt-broker.com -t "msh/test" -m "Hello from MQTT"
```

### Test 3: Check Bridge Logs
```bash
# View bridge logs
tail -f meshtastic_bridge.log
```

### Test 4: Verify Reticulum Connection
Check the logs for messages like:
```
INFO: reticulum_bridge: Connected to Reticulum via TCP
INFO: reticulum_bridge: Received packet from <address>
```

## Common Interface Configurations

### TCP Client (Connecting to Public Gateway)
```json
{
  "type": "tcp_client",
  "host": "RNS.MichMesh.net",
  "port": 7822,
  "enabled": true
}
```

### UDP (Local Network)
```json
{
  "type": "udp",
  "listen_host": "0.0.0.0",
  "listen_port": 4242,
  "connect_host": "192.168.1.100",
  "connect_port": 4242,
  "enabled": true
}
```

### Serial (Direct Device Connection)
```json
{
  "type": "serial",
  "port": "/dev/ttyUSB0",
  "baud_rate": 115200,
  "enabled": true
}
```

### KISS (Amateur Radio TNC)
```json
{
  "type": "kiss",
  "port": "/dev/ttyUSB0",
  "baud_rate": 9600,
  "enabled": true
}
```

## Troubleshooting

### Issue: MQTT Connection Failed
**Symptoms**: "Failed to connect to MQTT broker" error
**Solutions**:
1. Verify broker is running: `systemctl status mosquitto`
2. Check firewall settings
3. Verify credentials in config.json
4. Test with mosquitto client tools

### Issue: Reticulum Interface Not Working
**Symptoms**: No Reticulum connection messages in logs
**Solutions**:
1. Check interface configuration
2. Verify network connectivity
3. Test with `telnet <host> <port>` for TCP interfaces
4. Check if interface requires special permissions (serial/KISS)

### Issue: No Messages Flowing
**Symptoms**: Bridge runs but no messages are transmitted
**Solutions**:
1. Enable debug logging: `RUST_LOG=debug`
2. Check both MQTT and Reticulum connections
3. Verify topic subscriptions match Meshtastic topics
4. Check message filters in configuration

### Issue: High CPU/Memory Usage
**Symptoms**: Bridge uses excessive resources
**Solutions**:
1. Reduce log level from debug/trace to info
2. Check for message loops or high traffic
3. Review message cache settings
4. Consider rate limiting configuration

## Advanced Configuration

### Multiple Interfaces
You can configure multiple Reticulum interfaces:
```json
"interfaces": [
  {
    "type": "tcp_client",
    "host": "RNS.MichMesh.net",
    "port": 7822,
    "enabled": true
  },
  {
    "type": "udp",
    "listen_host": "0.0.0.0",
    "listen_port": 4242,
    "connect_host": "localhost",
    "connect_port": 4242,
    "enabled": true
  }
]
```

### Message Filtering
Control which messages are forwarded:
```json
"routing": {
  "forward_meshtastic_to_reticulum": true,
  "forward_reticulum_to_meshtastic": true,
  "filter_messages": true,
  "allowed_message_types": ["text", "position"],
  "blocked_sources": ["unwanted-node-id"],
  "allowed_destinations": []
}
```

### Performance Tuning
```json
"bridge": {
  "log_level": "info",
  "message_timeout_seconds": 30,
  "max_message_size": 1024,
  "enable_message_cache": true,
  "cache_size": 1000,
  "deduplicate_messages": true,
  "deduplication_window_seconds": 300
}
```

## Running as a Service

### Systemd Service (Linux)
Create `/etc/systemd/system/meshtastic-bridge.service`:
```ini
[Unit]
Description=Meshtastic MQTT Reticulum Bridge
After=network.target

[Service]
Type=simple
User=meshtastic
WorkingDirectory=/opt/meshtastic_mqtt_reticulum
ExecStart=/opt/meshtastic_mqtt_reticulum/target/release/gateway
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Enable and start the service:
```bash
sudo systemctl daemon-reload
sudo systemctl enable meshtastic-bridge
sudo systemctl start meshtastic-bridge
```

### Docker (Future)
```bash
docker run -d \
  -v ./config.json:/app/config.json \
  -v ./logs:/app/logs \
  --name meshtastic-bridge \
  meshtastic/reticulum-bridge:latest
```

## Next Steps

1. **Monitor Performance**: Check logs and system resources
2. **Test with Real Devices**: Connect actual Meshtastic devices
3. **Explore Advanced Features**: Try message filtering and routing
4. **Join the Community**: Share experiences and get help
5. **Contribute**: Report issues or submit improvements

## Getting Help

- **Documentation**: Check the `docs/` directory
- **Issues**: Report bugs on GitHub
- **Community**: Join Meshtastic and Reticulum communities
- **Logs**: Enable debug logging for detailed information

## Security Notes

1. **Use TLS**: Enable TLS for MQTT connections when possible
2. **Strong Passwords**: Use strong credentials for MQTT
3. **Firewall**: Restrict access to exposed interfaces
4. **Updates**: Keep the bridge and dependencies updated
5. **Monitoring**: Regularly check logs for suspicious activity