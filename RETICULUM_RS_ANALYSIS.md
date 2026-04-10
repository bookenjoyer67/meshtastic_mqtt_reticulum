# Reticulum-rs Analysis: Peer Status Information

## Issue Summary
The GUI shows peers with:
- Nickname: —
- Last seen: never  
- Signal strength: N/A
- Link quality: N/A

## Root Cause Analysis

### 1. Nickname (—)
- **Cause**: No nickname has been set for the peer
- **Solution**: Click the edit button (✏️) next to a peer in the GUI to set a nickname
- **Technical**: Nicknames are stored in `nicknames.txt` as `hash,nickname` pairs

### 2. Last seen (never)
- **Cause**: When GUI starts, it loads peers from `peers.txt` which only contains hash pairs without timestamps
- **Technical details**:
  - `peers.txt` format: `main_hash,file_hash` (no timestamp)
  - `load_peers()` creates `Peer` objects with `last_seen: None`
  - When `BridgeEvent::PeerDiscovered` is received, timestamp is updated
  - If GUI just started and no announces received yet, shows "never"
- **Solution**: 
  - Extend `peers.txt` format to include timestamps
  - Or create a separate peer metadata file
  - Or persist peer metadata in a structured format (JSON, SQLite)

### 3. Signal strength (N/A)
- **Cause**: Reticulum announce packets don't contain signal strength information
- **Technical details**:
  - Signal strength (RSSI) is a physical layer metric from radio hardware
  - TCP-based Reticulum connections (like `RNS.MichMesh.net:7822`) have no signal strength concept
  - Even with radio interfaces, Reticulum protocol doesn't include RSSI in announces
- **Solution**:
  - For radio interfaces: Could extract from hardware if interface provides it
  - For TCP: Not applicable, could show "TCP" or "N/A"

### 4. Link quality (N/A)
- **Cause**: Reticulum doesn't provide link quality metrics in announce packets
- **Technical details**:
  - Link quality would require tracking packet loss, latency, etc.
  - Not part of Reticulum protocol
  - Would need to be calculated from traffic patterns
- **Solution**:
  - Implement link quality estimation based on packet success rate
  - Or show placeholder/estimated values

## Reticulum-rs Implementation Analysis

### Current State
- **Announce packets**: Contain destination hash, identity, but no signal/link metrics
- **Interface types**: TCP, UDP, Serial, MQTT, KISS, I2P
- **Metrics available**: Performance counters (packets sent/received, timings) but not signal strength

### Limitations
1. **No physical layer metrics** in Reticulum protocol
2. **TCP interfaces** inherently lack signal strength concept
3. **Link quality** would require additional monitoring layer
4. **Peer metadata persistence** not built into protocol

## Recommendations

### Short-term fixes:
1. **Update GUI to show appropriate placeholders**:
   - For TCP: "Signal: TCP" instead of "N/A"
   - For radio: "Signal: Radio" or extract from hardware if possible
   
2. **Fix timestamp persistence**:
   ```rust
   // Extend peers.txt format:
   // main_hash,file_hash,timestamp_iso8601,signal_strength,link_quality,interface
   ```

3. **Improve GUI defaults**:
   - Show "Not available" instead of "N/A" for clarity
   - Add tooltip explaining why metrics aren't available

### Medium-term improvements:
1. **Extend Reticulum-rs with optional metrics**:
   - Add interface statistics (bytes sent/received, connection status)
   - Add estimated link quality based on packet success
   - For radio interfaces, expose hardware metrics if available

2. **Enhanced peer storage**:
   - Use SQLite or structured JSON for peer metadata
   - Store full peer history, not just current state

3. **GUI enhancements**:
   - Color-code peers by last seen (green=recent, yellow=old, red=stale)
   - Show interface type icon next to each peer
   - Add peer activity graph/timeline

### Long-term vision:
1. **Reticulum protocol extension** for optional metrics in announces
2. **Hardware abstraction layer** for radio metric collection
3. **Network health dashboard** with visualizations

## Code Changes Needed

### 1. Fix peer timestamp persistence:
```rust
// In save_peer() - extend to save metadata
pub fn save_peer(&self, peer: &Peer) {
    let line = format!("{},{},{},{},{},{}\n",
        peer.main_hash,
        peer.file_hash,
        peer.last_seen.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
        peer.signal_strength.map(|s| s.to_string()).unwrap_or("".to_string()),
        peer.link_quality.map(|q| q.to_string()).unwrap_or("".to_string()),
        peer.interface.as_deref().unwrap_or("")
    );
    // ... save to file
}
```

### 2. Update GUI display logic:
```rust
// Instead of "N/A", show context-appropriate labels
match peer.interface.as_deref() {
    Some(iface) if iface.contains("TCP") => "TCP Connection",
    Some(iface) if iface.contains("Radio") => "Radio (no RSSI)",
    _ => "Not available",
}
```

### 3. Add interface metrics collection (if possible):
```rust
// Would require changes to Reticulum-rs interfaces
pub trait InterfaceWithMetrics {
    fn get_metrics(&self) -> InterfaceMetrics;
}

struct InterfaceMetrics {
    bytes_sent: u64,
    bytes_received: u64,
    connected: bool,
    // Radio-specific metrics if available
    rssi: Option<i32>,
    snr: Option<i32>,
}
```

## Conclusion
The "N/A" values are expected given Reticulum's design and the use of TCP interfaces. The "never" for last seen is a data persistence issue. Significant changes to Reticulum protocol or hardware integration would be needed to provide actual signal strength and link quality metrics for TCP-based mesh networks.