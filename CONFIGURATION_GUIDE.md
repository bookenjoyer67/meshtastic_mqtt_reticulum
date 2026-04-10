# Configuration Guide for Meshtastic MQTT Reticulum Bridge

This document explains how to configure the Meshtastic MQTT Reticulum Bridge with secure defaults and environment variables.

## Overview

The application has been secured with the following improvements:
- **No hardcoded credentials** in source code
- **Environment variable support** for all configuration
- **Secure defaults** (TLS enabled, empty credentials)
- **Input validation** and sanitization
- **Configuration validation** for security issues

## Security First Configuration

### Critical Security Settings

| Variable | Description | Secure Default | Importance |
|----------|-------------|----------------|------------|
| `MQTT_USERNAME` | MQTT broker username | **Empty** (must be set) | **CRITICAL** |
| `MQTT_PASSWORD` | MQTT broker password | **Empty** (must be set) | **CRITICAL** |
| `MQTT_USE_TLS` | Use TLS for MQTT | `true` | **HIGH** |
| `MQTT_PORT` | MQTT broker port | `8883` (TLS) | **HIGH** |
| `GUI_BIND_ADDRESS` | GUI bind address | `127.0.0.1` | **MEDIUM** |

### Complete Environment Variables

| Variable | Description | Default Value | Security Note |
|----------|-------------|---------------|---------------|
| `MQTT_USERNAME` | MQTT broker username | `""` | **Must be set** |
| `MQTT_PASSWORD` | MQTT broker password | `""` | **Must be set** |
| `MQTT_HOST` | MQTT broker hostname | `mqtt.meshtastic.org` | - |
| `MQTT_PORT` | MQTT broker port | `8883` | TLS port |
| `MQTT_USE_TLS` | Use TLS for MQTT (`true`/`false` or `1`/`0`) | `true` | **Enabled by default** |
| `RETICULUM_SERVER` | Reticulum server address (host:port) | `RNS.MichMesh.net:7822` | - |
| `MESHTASTIC_CHANNELS` | Initial channels with PSKs (format: `CHANNEL1:PSK1,CHANNEL2:PSK2`) | Empty | Use secure PSKs |
| `GUI_BIND_ADDRESS` | TCP bind address for GUI interface | `127.0.0.1` | Localhost only |
| `GUI_PORT` | TCP port for GUI interface | `4243` | - |

## Secure Deployment Examples

### Basic Secure Configuration

```bash
# REQUIRED: Set MQTT credentials (never use defaults)
export MQTT_USERNAME="your_secure_username"
export MQTT_PASSWORD="strong_password_here"

# OPTIONAL: Override other settings
export MQTT_HOST="mqtt.meshtastic.org"
export MQTT_PORT="8883"           # TLS port
export MQTT_USE_TLS="true"        # Always use TLS

# Reticulum Configuration
export RETICULUM_SERVER="RNS.MichMesh.net:7822"

# Generate secure PSK: openssl rand -base64 32
export MESHTASTIC_CHANNELS="secure-channel:yJjV8L3P7qKtWxZbNcRfTgYhUjIkOlP0Q1S2D3F4G5H6J7K8L9M0N1O2P3Q4"

# GUI Configuration (localhost only for security)
export GUI_BIND_ADDRESS="127.0.0.1"
export GUI_PORT="4243"

# Run with security validation
cargo run --bin bridge
```

### Using a Secure .env File

Create a `.env` file (add to `.gitignore`):

```env
# === SECURITY WARNING: Never commit this file ===
# MQTT Configuration
MQTT_USERNAME=your_secure_username
MQTT_PASSWORD=strong_password_here
MQTT_HOST=mqtt.meshtastic.org
MQTT_PORT=8883                    # TLS port
MQTT_USE_TLS=true                 # Always use TLS

# Reticulum Configuration
RETICULUM_SERVER=RNS.MichMesh.net:7822

# Channels Configuration (generate with: openssl rand -base64 32)
MESHTASTIC_CHANNELS=secure-channel:yJjV8L3P7qKtWxZbNcRfTgYhUjIkOlP0Q1S2D3F4G5H6J7K8L9M0N1O2P3Q4

# GUI Configuration
GUI_BIND_ADDRESS=127.0.0.1        # Localhost only for security
GUI_PORT=4243
```

Load with dotenv:
```bash
dotenv cargo run --bin bridge
```

### Docker Secure Configuration

```yaml
version: '3.8'
services:
  meshtastic-bridge:
    build: .
    environment:
      # Required credentials (use Docker secrets in production)
      - MQTT_USERNAME=${MQTT_USERNAME}
      - MQTT_PASSWORD=${MQTT_PASSWORD}
      
      # Security settings
      - MQTT_USE_TLS=true
      - MQTT_PORT=8883
      - GUI_BIND_ADDRESS=0.0.0.0  # Only if remote access needed
      
      # Other configuration
      - RETICULUM_SERVER=${RETICULUM_SERVER}
      - MESHTASTIC_CHANNELS=${MESHTASTIC_CHANNELS}
      - GUI_PORT=4243
    ports:
      - "4243:4243"  # Only expose if needed
    restart: unless-stopped
    security_opt:
      - no-new-privileges:true
    read_only: true  # If filesystem supports it
```

## Security Validation

The application includes configuration validation. To check your configuration:

```rust
// In your code:
let config = Config::from_env();
if let Err(errors) = config.validate() {
    for error in errors {
        eprintln!("SECURITY WARNING: {}", error);
    }
    // Handle insecure configuration
}
```

Common validation errors:
- Empty MQTT credentials
- Non-TLS MQTT connection
- Weak or short PSKs
- Invalid server addresses

## Security Best Practices

### 1. Credential Management
- **Never use default credentials** - they are empty for security
- **Generate strong passwords** - 16+ characters, mixed characters
- **Use environment variables** or secure secret stores
- **Rotate credentials regularly** - especially after personnel changes

### 2. Transport Security
- **Always use TLS** for MQTT (port 8883, not 1883)
- **Verify certificates** when connecting to MQTT brokers
- **Use localhost binding** (`127.0.0.1`) unless remote access required
- **Implement firewall rules** to restrict network access

### 3. Cryptographic Security
- **Generate secure PSKs**: `openssl rand -base64 32`
- **Minimum PSK length**: 16+ characters in base64
- **Unique PSKs per channel** - never reuse PSKs
- **Regular PSK rotation** - especially for sensitive channels

### 4. Network Security
- **Restrict GUI access** to localhost by default
- **Use VPN for remote access** instead of exposing ports
- **Monitor connection attempts** for brute force attacks
- **Implement rate limiting** (planned feature)

### 5. Operational Security
- **Regular updates** - keep dependencies patched
- **Log monitoring** - watch for security warnings
- **Backup configurations** without plaintext passwords
- **Least privilege** - run with minimal permissions

## Configuration Structure

The secure configuration system provides:

- **`Config::from_env()`** - Load from environment variables
- **`config.validate()`** - Security validation
- **Secure defaults** - TLS enabled, empty credentials
- **Type safety** - Compile-time configuration checking

## Security Features

### Input Validation
- **Message limits**: 10,000 characters maximum
- **Channel validation**: No MQTT wildcards (`/`, `#`, `+`)
- **PSK validation**: Warnings for weak/short keys
- **Sanitization**: Removal of control characters

### Error Handling
- **No unsafe `unwrap()` calls** - proper error handling
- **Graceful degradation** on failures
- **Security warnings** for weak configurations
- **No sensitive data in logs**

### Secure Defaults
- **TLS enabled by default**
- **Empty credentials** force explicit configuration
- **Localhost binding** for GUI
- **Strong validation** of all inputs

## Verification Commands

Verify no hardcoded credentials remain:
```bash
# Check for removed credentials
grep -r "meshdev\|large4cats" src/ && echo "FAIL: Hardcoded credentials found" || echo "PASS: No hardcoded credentials"

# Check for secure defaults
grep -r "mqtt_port.*8883" src/config.rs && echo "PASS: TLS port default" || echo "FAIL: Incorrect port"

# Check for empty credential defaults
grep -r 'mqtt_username: "".to_string()' src/config.rs && echo "PASS: Empty username default" || echo "FAIL"
```

## Troubleshooting Security Issues

### Common Problems and Solutions

1. **"MQTT credentials are empty" warning**
   - **Solution**: Set `MQTT_USERNAME` and `MQTT_PASSWORD` environment variables

2. **"MQTT connection is not using TLS" warning**
   - **Solution**: Set `MQTT_USE_TLS=true` and `MQTT_PORT=8883`

3. **"Channel has a very short PSK" warning**
   - **Solution**: Generate new PSK: `openssl rand -base64 32`

4. **Connection refused on port 8883**
   - **Solution**: Verify broker supports TLS on port 8883

5. **GUI not accessible remotely**
   - **Solution**: Set `GUI_BIND_ADDRESS=0.0.0.0` (only if needed, with firewall)

### Security Incident Response

If you suspect a security issue:

1. **Immediate actions**:
   ```bash
   # Stop the application
   pkill -f meshtastic_bridge
   
   # Rotate credentials
   export NEW_PSK=$(openssl rand -base64 32)
   export NEW_PASSWORD=$(openssl rand -base64 16)
   
   # Check logs for anomalies
   grep -i "error\|warning\|fail\|invalid" application.log
   ```

2. **Investigation**:
   - Review configuration for weaknesses
   - Check for unauthorized channel access
   - Monitor for unusual traffic patterns

3. **Recovery**:
   - Update all credentials and PSKs
   - Apply security patches
   - Review and tighten configuration

## Migration from Insecure Configurations

If upgrading from a version with hardcoded credentials:

1. **Backup your data** but not plaintext credentials
2. **Set environment variables** with secure values
3. **Generate new PSKs** for all channels
4. **Test thoroughly** before production deployment
5. **Monitor logs** for security warnings

## Getting Security Help

For security concerns:
1. **Review logs** for security warnings
2. **Check configuration** against this guide
3. **Consult** `SECURITY_GUIDE.md` for detailed guidance
4. **Report issues** responsibly to maintainers

---

**Security Version**: 2.0  
**Last Updated**: 2026-03-31  
**Status**: Production-ready with secure defaults