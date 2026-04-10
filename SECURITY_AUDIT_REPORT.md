# Security Audit Report: Meshtastic MQTT Reticulum Bridge

**Date:** 2026-03-31  
**Auditor:** Claude Code  
**Project:** Meshtastic MQTT Reticulum Bridge  
**Version:** 0.1.0  

## Executive Summary

This security audit identified several critical security issues in the Meshtastic MQTT Reticulum Bridge project. The most severe issues include **hardcoded credentials**, **insecure error handling**, and **lack of input validation**. Immediate remediation is recommended for the high-severity findings.

## Risk Assessment Summary

| Severity | Count | Status |
|----------|-------|--------|
| Critical | 2 | Requires immediate attention |
| High | 3 | Should be addressed promptly |
| Medium | 4 | Should be addressed in next release |
| Low | 2 | Consider addressing |

## Detailed Findings

### Critical Severity Issues

#### 1. **Hardcoded MQTT Credentials** (CRITICAL)
- **Location:** `src/mqtt.rs:35`
- **Issue:** Hardcoded MQTT credentials `("meshdev", "large4cats")` in source code
- **Impact:** Credentials are exposed in source control and binaries
- **Risk:** Unauthorized access to MQTT broker, potential for credential theft
- **Recommendation:** 
  - Move credentials to environment variables or configuration files
  - Use secure credential storage
  - Implement credential rotation

#### 2. **Hardcoded Pre-Shared Keys (PSK)** (CRITICAL)
- **Location:** `src/mqtt.rs:28`
- **Issue:** Hardcoded PSK `"bB4gkiCaZhFRtAHBDYmwssojzLxFSBuERQMvmmgESfs="` for channel "STLIW-MC"
- **Impact:** Encryption keys exposed in source code
- **Risk:** Compromised channel security, unauthorized decryption of messages
- **Recommendation:**
  - Store PSKs in encrypted configuration
  - Use key management system
  - Generate unique PSKs per deployment

### High Severity Issues

#### 3. **Unsafe Error Handling with `unwrap()`** (HIGH)
- **Location:** Multiple files (82 instances found)
- **Issue:** Extensive use of `unwrap()` without proper error handling
- **Impact:** Application crashes on unexpected conditions
- **Risk:** Denial of service, unstable operation
- **Recommendation:**
  - Replace `unwrap()` with proper error handling (`?` operator, `match` statements)
  - Implement graceful degradation
  - Add comprehensive error logging

#### 4. **Lack of Input Validation** (HIGH)
- **Location:** `src/main_bridge.rs:85-105`
- **Issue:** JSON parsing without validation of structure or content
- **Impact:** Potential for malformed data causing crashes or unexpected behavior
- **Risk:** Injection attacks, parsing errors, memory issues
- **Recommendation:**
  - Implement schema validation for JSON messages
  - Validate string lengths and content
  - Sanitize user inputs

#### 5. **Insecure Default PSK** (HIGH)
- **Location:** `src/gui/channels_impl.rs:15`
- **Issue:** Default PSK set to `"AQ=="` (base64 for single null byte)
- **Impact:** Weak default encryption
- **Risk:** Insecure channels by default
- **Recommendation:**
  - Require explicit PSK configuration
  - Generate secure random PSKs
  - Warn about weak PSKs

### Medium Severity Issues

#### 6. **TCP Server Binding to Localhost Only** (MEDIUM)
- **Location:** `src/main_bridge.rs:57`
- **Issue:** TCP server binds to `127.0.0.1:4243` only
- **Impact:** Limited to local connections
- **Risk:** May be too restrictive for some deployment scenarios
- **Recommendation:**
  - Make bind address configurable
  - Consider network isolation requirements
  - Document security implications

#### 7. **No Transport Layer Security (TLS) for MQTT** (MEDIUM)
- **Location:** `src/mqtt.rs:33`
- **Issue:** MQTT connection uses plaintext (port 1883)
- **Impact:** Unencrypted network traffic
- **Risk:** Eavesdropping, man-in-the-middle attacks
- **Recommendation:**
  - Implement TLS for MQTT connections (port 8883)
  - Validate server certificates
  - Consider mutual TLS authentication

#### 8. **No Rate Limiting** (MEDIUM)
- **Location:** Throughout codebase
- **Issue:** No protection against flooding or DoS attacks
- **Impact:** Resource exhaustion
- **Risk:** Denial of service
- **Recommendation:**
  - Implement rate limiting per connection
  - Add message size limits
  - Monitor for abnormal traffic patterns

#### 9. **Insufficient Logging for Security Events** (MEDIUM)
- **Location:** Throughout codebase
- **Issue:** Limited security event logging
- **Impact:** Difficult forensic analysis
- **Risk:** Undetected security incidents
- **Recommendation:**
  - Log authentication attempts
  - Log message sending/receiving
  - Implement audit trail

### Low Severity Issues

#### 10. **Use of Deprecated or Unmaintained Dependencies** (LOW)
- **Location:** `Cargo.toml`
- **Issue:** Some dependencies may have security vulnerabilities
- **Impact:** Potential supply chain attacks
- **Risk:** Exploitation of known vulnerabilities
- **Recommendation:**
  - Run `cargo audit` regularly
  - Update dependencies frequently
  - Consider dependency pinning

#### 11. **Lack of Code Signing** (LOW)
- **Issue:** No code signing for binaries
- **Impact:** Cannot verify binary integrity
- **Risk:** Tampering with distributed binaries
- **Recommendation:**
  - Implement code signing
  - Provide checksums for releases
  - Use reproducible builds

## Code Quality Issues

### Error Handling Patterns
- **Problem:** 82 instances of `unwrap()`, `expect()`, and `panic!` found
- **Files with most issues:**
  - `src/mqtt.rs` - Multiple `unwrap()` calls on JSON serialization
  - `src/main_bridge.rs` - `unwrap()` on packet data writing
  - Various GUI files - `unwrap()` on channel operations

### Memory Safety
- **Status:** No `unsafe` blocks found in main application code
- **Note:** Reticulum-rs dependency contains `unsafe` code (needs separate audit)

## Network Security Assessment

### MQTT Implementation
- **Protocol:** MQTT 3.1/3.1.1 via rumqttc
- **Authentication:** Hardcoded username/password
- **Encryption:** None (plaintext)
- **QoS:** AtMostOnce for subscriptions, AtLeastOnce for publishing
- **Vulnerabilities:** Credential exposure, lack of encryption

### Reticulum Integration
- **Transport:** TCP to `RNS.MichMesh.net:7822`
- **Authentication:** Cryptographic identity-based
- **Encryption:** End-to-end via Reticulum protocol
- **Security:** Inherits Reticulum security model

### Local Communication
- **Interface:** TCP on `127.0.0.1:4243`
- **Protocol:** JSON over newline-delimited TCP
- **Security:** Local-only, no authentication between components

## Dependencies Analysis

### Key Dependencies
1. **rumqttc (0.24)** - MQTT client library
2. **tokio (1.x)** - Async runtime
3. **serde_json (1.x)** - JSON serialization
4. **chrono (0.4)** - Date/time handling
5. **image (0.24)** - Image processing
6. **meshtastic_protobufs (2.7)** - Protocol buffers

### Security Notes
- **chrono 0.4** has known issues with time parsing
- **image 0.24** has had security issues in past versions
- **No cargo audit** was run due to tool absence

## Configuration Security

### Files Reviewed
- `Cargo.toml` - Build configuration
- `start-meshtastic.sh` - Startup script
- No environment configuration files found

### Issues
- Startup script uses `gnome-terminal` with hardcoded paths
- No configuration file for sensitive settings
- No environment variable support for credentials

## Recommendations by Priority

### Immediate Actions (Critical/High)
1. **Remove hardcoded credentials** from source code
2. **Implement environment-based configuration** for secrets
3. **Replace all `unwrap()` calls** with proper error handling
4. **Add input validation** for all user/data inputs
5. **Implement TLS** for MQTT connections

### Short-term Actions (Medium)
6. **Make network bindings configurable**
7. **Implement rate limiting** and message size limits
8. **Add comprehensive security logging**
9. **Run regular dependency audits** with `cargo audit`

### Long-term Actions (Low)
10. **Implement code signing** for releases
11. **Add automated security testing** to CI/CD
12. **Create security documentation** for deployers
13. **Consider implementing mutual TLS** for component communication

## Testing Recommendations

1. **Fuzz testing** for JSON parsing and message handling
2. **Penetration testing** of network interfaces
3. **Dependency scanning** with `cargo audit`
4. **Static analysis** with `cargo clippy` and security-focused linters
5. **Integration testing** with invalid/malicious inputs

## Conclusion

The Meshtastic MQTT Reticulum Bridge has significant security issues that must be addressed before production deployment. The most critical issues are hardcoded credentials and poor error handling. While the Reticulum protocol provides strong cryptographic security, the MQTT integration and application layer have multiple vulnerabilities.

**Overall Security Rating: POOR**

**Next Steps:**
1. Address all Critical and High severity issues
2. Implement secure credential management
3. Conduct thorough testing of fixes
4. Consider a follow-up audit after remediation

---

*This audit was conducted automatically and should be reviewed by a human security expert before making deployment decisions.*