# IMMEDIATE SECURITY FIXES REQUIRED

## Critical Issues That Must Be Fixed NOW

### 1. HARDCODED CREDENTIALS - FIX IMMEDIATELY
**File:** `src/mqtt.rs`
**Line 35:** `mqttoptions.set_credentials("meshdev", "large4cats");`

**Quick Fix:**
```rust
// Replace with:
let username = std::env::var("MQTT_USERNAME").unwrap_or_else(|_| "meshdev".to_string());
let password = std::env::var("MQTT_PASSWORD").unwrap_or_else(|_| "large4cats".to_string());
mqttoptions.set_credentials(&username, &password);
```

**Better Fix:** Use proper configuration file or secrets manager.

### 2. HARDCODED PSK - FIX IMMEDIATELY
**File:** `src/mqtt.rs`
**Line 28:** `channels.insert("STLIW-MC".to_string(), "bB4gkiCaZhFRtAHBDYmwssojzLxFSBuERQMvmmgESfs=".to_string());`

**Quick Fix:** Remove hardcoded PSK and load from configuration.

### 3. UNSAFE UNWRAP() - FIX HIGH PRIORITY
**Most dangerous instances:**

1. `src/mqtt.rs:52` - `let json_string = serde_json::to_string(&json_payload).unwrap();`
   **Fix:** `let json_string = serde_json::to_string(&json_payload)?;`

2. `src/main_bridge.rs:103` - `packet_data.write(text.as_bytes()).unwrap();`
   **Fix:** Handle write errors properly.

3. Multiple `unwrap()` calls on JSON parsing - add proper error handling.

### 4. NO TLS FOR MQTT - MEDIUM PRIORITY
**File:** `src/mqtt.rs`
**Line 33:** `let mut mqttoptions = MqttOptions::new("meshtastic-gui", "mqtt.meshtastic.org", 1883);`

**Fix:** Use port 8883 for TLS and configure TLS options.

## Emergency Action Plan

1. **Today:** Remove hardcoded credentials from source code
2. **This Week:** Fix all `unwrap()` calls with proper error handling  
3. **Next Week:** Implement TLS for MQTT connections
4. **Next Month:** Complete all security fixes from audit report

## Testing After Fixes
- Run `cargo test` to ensure no regressions
- Test with invalid inputs to verify error handling
- Verify credentials are not in compiled binary
- Test TLS connection to MQTT broker