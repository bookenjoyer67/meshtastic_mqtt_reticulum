# Security Guide for Meshtastic MQTT Reticulum Bridge

## Overview

This guide provides security best practices for deploying and using the Meshtastic MQTT Reticulum Bridge. The application has been hardened with several security improvements, but proper configuration and deployment practices are essential for secure operation.

## Security Improvements Applied

### 1. Credential Management
- **Removed hardcoded credentials** from source code
- **Environment variable support** for all sensitive configuration
- **Empty default credentials** to force explicit configuration
- **Configuration validation** to detect insecure settings

### 2. Transport Security
- **TLS enabled by default** for MQTT connections (port 8883)
- **Secure defaults** for all network connections
- **Input validation** to prevent injection attacks

### 3. Error Handling
- **Replaced unsafe `unwrap()` calls** with proper error handling
- **Graceful degradation** on connection failures
- **Comprehensive error messages** without exposing sensitive information

### 4. Input Validation
- **Message length limits** (10,000 characters max)
- **Channel name validation** (no special MQTT wildcards)
- **PSK validation** with warnings for weak keys
- **Input sanitization** to remove control characters

### 5. Cryptographic Security
- **Warnings for weak PSKs** (short or default keys)
- **Minimum PSK length recommendations** (16+ characters in base64)
- **Secure channel configuration** validation

## Configuration Security

### Environment Variables

Set these environment variables for secure operation:

```bash
# Required: MQTT Credentials (never use defaults)
export MQTT_USERNAME="your_username"
export MQTT_PASSWORD="strong_password_here"

# Optional: Override defaults
export MQTT_HOST="mqtt.meshtastic.org"
export MQTT_PORT="8883"  # Use TLS port
export MQTT_USE_TLS="true"  # Always use TLS

# Reticulum Configuration
export RETICULUM_SERVER="RNS.MichMesh.net:7822"

# Channel Configuration (channel:psk pairs)
export MESHTASTIC_CHANNELS="channel1:base64psk1,channel2:base64psk2"

# GUI Configuration
export GUI_BIND_ADDRESS="127.0.0.1"  # Bind to localhost only
export GUI_PORT="4243"
```

### Secure PSK Generation

Generate secure PSKs for channels:

```bash
# Generate a 32-byte (256-bit) random PSK
openssl rand -base64 32

# Example output: yJjV8L3P7qKtWxZbNcRfTgYhUjIkOlP0Q1S2D3F4G5H6J7K8L9M0N1O2P3Q4
```

**Minimum Requirements:**
- At least 16 characters in base64 encoding
- Cryptographically random (not predictable)
- Unique per channel/deployment

## Deployment Security Checklist

### Before Deployment
- [ ] Generate unique MQTT credentials
- [ ] Generate secure PSKs for all channels
- [ ] Configure TLS for MQTT connections
- [ ] Set appropriate bind addresses (localhost recommended)
- [ ] Review and update all environment variables

### Runtime Security
- [ ] Monitor for security warnings in application logs
- [ ] Regularly rotate credentials and PSKs
- [ ] Keep dependencies updated (`cargo audit`)
- [ ] Monitor for unusual message patterns
- [ ] Use firewall rules to restrict network access

### Network Security
- **MQTT**: Always use TLS (port 8883, not 1883)
- **GUI Interface**: Bind to localhost (127.0.0.1) unless remote access is needed
- **Reticulum**: Use trusted Reticulum servers
- **Firewall**: Restrict inbound connections to necessary ports only

## Threat Mitigation

### 1. Credential Theft
- **Mitigation**: Environment variables, no hardcoded credentials
- **Detection**: Monitor for authentication failures
- **Response**: Immediate credential rotation

### 2. Eavesdropping
- **Mitigation**: TLS for all MQTT communications
- **Detection**: Certificate validation failures
- **Response**: Verify TLS configuration, check certificates

### 3. Injection Attacks
- **Mitigation**: Input validation and sanitization
- **Detection**: Malformed message rejection logs
- **Response**: Review attack patterns, enhance validation

### 4. Denial of Service
- **Mitigation**: Message size limits, rate limiting (planned)
- **Detection**: Unusual message volume
- **Response**: Adjust limits, block abusive sources

### 5. Weak Cryptography
- **Mitigation**: PSK validation and warnings
- **Detection**: Security warning logs
- **Response**: Regenerate weak PSKs immediately

## Monitoring and Logging

### Security Events to Monitor
- Authentication failures
- TLS handshake failures
- Input validation rejections
- Connection rate anomalies
- Security warnings about weak PSKs

### Log Configuration
Enable verbose logging for security monitoring:
```bash
RUST_LOG=info,meshtastic_reticulum_bridge=debug ./bridge
```

## Incident Response

### Immediate Actions
1. **Isolate**: Disconnect from network if compromise suspected
2. **Rotate**: Change all credentials and PSKs
3. **Audit**: Review logs for attack patterns
4. **Update**: Patch any vulnerable dependencies

### Forensic Analysis
- Preserve application logs
- Document configuration state
- Note any unusual behavior before incident
- Check for unauthorized channel access

## Additional Security Considerations

### 1. Dependency Security
Regularly audit dependencies:
```bash
cargo audit
cargo update
```

### 2. Code Review
- Review all configuration changes
- Audit custom channel implementations
- Verify PSK generation procedures

### 3. Physical Security
- Secure devices running the bridge
- Protect access to configuration files
- Use secure boot where possible

### 4. Operational Security
- Least privilege principle for service accounts
- Regular security updates
- Backup of secure configurations (without plaintext passwords)

## Getting Help

For security concerns:
1. Review application logs for security warnings
2. Check configuration against this guide
3. Consult Meshtastic community security channels
4. Report security vulnerabilities responsibly

---

**Last Updated:** 2026-03-31  
**Security Version:** 1.0  
**Status:** Production-ready with secure defaults