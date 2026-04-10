# Reticulum Protocol Extension Proposal: Network Metrics

## Overview
This document proposes an extension to the Reticulum protocol to include optional network metrics in packet headers. These metrics would allow nodes to exchange information about link quality, signal strength, latency, and other performance characteristics.

## Motivation
Currently, Reticulum provides basic connectivity but lacks standardized mechanisms for:
1. Real-time link quality assessment
2. Signal strength reporting (RSSI/SNR for radio interfaces)
3. Network latency measurement
4. Packet loss statistics
5. Bandwidth estimation

Adding these metrics would enable:
- Better route selection in multi-path scenarios
- Adaptive transmission parameters
- Improved user experience with visual signal indicators
- Network health monitoring and diagnostics

## Proposed Changes

### 1. Extended Packet Header Format

Add an optional metrics section to packet headers:

```
Packet Header Extension:
+----------------+----------------+----------------+----------------+
|  Metrics Flag  |  Metrics Type  |  Metrics Length |  Metrics Data  |
|    (1 byte)    |    (1 byte)    |    (2 bytes)   |   (variable)   |
+----------------+----------------+----------------+----------------+
```

- **Metrics Flag**: Bitfield indicating which metrics are present
- **Metrics Type**: Type of metrics (radio, network, hybrid, etc.)
- **Metrics Length**: Length of metrics data in bytes
- **Metrics Data**: Actual metrics data

### 2. Metrics Flag Bitfield

```
Bit 0: Radio Metrics (RSSI, SNR, etc.)
Bit 1: Network Metrics (latency, packet loss, etc.)
Bit 2: Link Quality Metrics
Bit 3: Bandwidth Metrics
Bit 4-7: Reserved for future use
```

### 3. Metrics Data Structures

#### 3.1 Radio Metrics (Type 0x01)
For LoRa, WiFi, Bluetooth, and other radio interfaces:

```
+----------------+----------------+----------------+----------------+
|   RSSI (dBm)   |    SNR (dB)    |   Link Quality  |   Reserved    |
|    (1 byte)    |    (1 byte)    |    (1 byte)    |    (1 byte)    |
+----------------+----------------+----------------+----------------+
```

- **RSSI**: Received Signal Strength Indicator (-128 to 127 dBm)
- **SNR**: Signal-to-Noise Ratio (-128 to 127 dB)
- **Link Quality**: 0-100% link quality estimation
- **Reserved**: For future radio-specific metrics

#### 3.2 Network Metrics (Type 0x02)
For TCP, UDP, and other network interfaces:

```
+----------------+----------------+----------------+----------------+
|  Latency (ms)  | Packet Loss (%)|    Jitter (ms)  |  Quality Score|
|    (2 bytes)   |    (1 byte)    |    (2 bytes)   |    (1 byte)    |
+----------------+----------------+----------------+----------------+
```

- **Latency**: Round-trip time in milliseconds (0-65535 ms)
- **Packet Loss**: Percentage (0-100%)
- **Jitter**: Latency variation in milliseconds (0-65535 ms)
- **Quality Score**: Overall connection quality (0-100)

#### 3.3 Hybrid Metrics (Type 0x03)
For interfaces with both radio and network characteristics:

```
+----------------+----------------+----------------+----------------+
|   RSSI (dBm)   |  Latency (ms)  | Packet Loss (%)|  Quality Score|
|    (1 byte)    |    (2 bytes)   |    (1 byte)    |    (1 byte)    |
+----------------+----------------+----------------+----------------+
+----------------+----------------+
|   Reserved     |   Timestamp    |
|    (2 bytes)   |    (4 bytes)   |
+----------------+----------------+
```

- **Timestamp**: Unix timestamp of measurement (seconds since epoch)

### 4. Announce Packet Extension

Extend announce packets to include node capabilities and current metrics:

```
Announce Extension:
+----------------+----------------+----------------+----------------+
|  Capabilities  |  Current RSSI  | Current Latency|  Interface Type|
|    (1 byte)    |    (1 byte)    |    (2 bytes)   |    (1 byte)    |
+----------------+----------------+----------------+----------------+
```

- **Capabilities**: Bitfield of supported metrics
- **Current RSSI**: Current signal strength if applicable
- **Current Latency**: Current network latency if applicable
- **Interface Type**: Type of primary interface (radio, network, hybrid)

### 5. Protocol Implementation Details

#### 5.1 Backward Compatibility
- Metrics are optional; existing implementations ignore unknown header extensions
- Default behavior when metrics not present
- Graceful degradation to current protocol behavior

#### 5.2 Metric Collection
- Nodes collect metrics from their interfaces
- Metrics are averaged over a sliding window (e.g., last 10 measurements)
- Metrics are updated periodically (e.g., every 5 seconds)
- Stale metrics are marked as invalid

#### 5.3 Metric Validation
- Range checking for all metric values
- Timestamp validation to prevent stale data
- Consistency checks between different metric types

### 6. API Changes

#### 6.1 Rust API (Reticulum-rs)

```rust
// New struct for network metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub latency_ms: u16,
    pub packet_loss_percent: u8,
    pub jitter_ms: u16,
    pub quality_score: u8,
    pub timestamp: u32,
}

// New struct for radio metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioMetrics {
    pub rssi_dbm: i8,
    pub snr_db: i8,
    pub link_quality: u8,
    pub timestamp: u32,
}

// Extended packet header
pub struct PacketHeader {
    // Existing fields...
    pub metrics_flag: u8,
    pub metrics_type: u8,
    pub metrics_length: u16,
    pub metrics_data: Vec<u8>,
}

// New trait for metric collection
pub trait MetricsCollector {
    fn collect_radio_metrics(&self) -> Option<RadioMetrics>;
    fn collect_network_metrics(&self) -> Option<NetworkMetrics>;
    fn get_interface_type(&self) -> InterfaceType;
}

// New interface type enum
pub enum InterfaceType {
    Radio,
    Network,
    Hybrid,
    Unknown,
}
```

#### 6.2 Python API (Original Reticulum)

```python
# New classes for metrics
class NetworkMetrics:
    def __init__(self, latency_ms=0, packet_loss_percent=0, 
                 jitter_ms=0, quality_score=0):
        self.latency_ms = latency_ms
        self.packet_loss_percent = packet_loss_percent
        self.jitter_ms = jitter_ms
        self.quality_score = quality_score
        self.timestamp = time.time()

class RadioMetrics:
    def __init__(self, rssi_dbm=0, snr_db=0, link_quality=0):
        self.rssi_dbm = rssi_dbm
        self.snr_db = snr_db
        self.link_quality = link_quality
        self.timestamp = time.time()

# Extended packet class
class Packet:
    def __init__(self, ...):
        # Existing initialization...
        self.metrics_flag = 0
        self.metrics_type = 0
        self.metrics_data = None
    
    def set_metrics(self, metrics):
        # Set metrics based on type
        pass
    
    def get_metrics(self):
        # Parse and return metrics
        pass
```

### 7. Integration with Meshtastic Reticulum Bridge

#### 7.1 Enhanced Peer Discovery

```rust
// Enhanced peer structure with metrics
pub struct EnhancedPeer {
    pub main_hash: String,
    pub file_hash: String,
    pub name: Option<String>,
    pub last_seen: Option<DateTime<Local>>,
    pub signal_strength: Option<i32>, // RSSI in dBm
    pub link_quality: Option<u8>,     // 0-100
    pub interface: Option<String>,    // interface name
    pub network_metrics: Option<NetworkMetrics>,
    pub radio_metrics: Option<RadioMetrics>,
    pub last_metrics_update: Option<DateTime<Local>>,
}
```

#### 7.2 Metrics-Aware Routing

- Use metrics to select best path in multi-hop scenarios
- Adaptive transmission parameters based on link quality
- Fallback to alternative interfaces when primary degrades

#### 7.3 GUI Integration

- Visual signal strength indicators
- Real-time metrics display
- Historical metrics graphs
- Connection quality warnings

### 8. Security Considerations

1. **Metric Authentication**: Ensure metrics come from trusted sources
2. **Metric Validation**: Prevent malicious nodes from injecting false metrics
3. **Privacy Considerations**: Some metrics might reveal node location or capabilities
4. **Denial of Service**: Prevent metric flooding attacks

### 9. Performance Impact

1. **Header Overhead**: Additional 4-16 bytes per packet
2. **Processing Overhead**: Minimal additional CPU usage
3. **Memory Usage**: Small increase for metric storage
4. **Network Traffic**: Negligible increase for metric exchange

### 10. Implementation Phases

#### Phase 1: Basic Metrics Support
- Add optional metrics header extension
- Implement basic metric collection for LoRa and TCP
- Update announce packets to include capabilities

#### Phase 2: Enhanced Routing
- Metrics-aware path selection
- Adaptive transmission parameters
- Fallback mechanisms

#### Phase 3: Advanced Features
- Historical metrics storage
- Predictive link quality
- Automated network optimization

### 11. Testing Plan

1. **Unit Tests**: Individual metric collection and parsing
2. **Integration Tests**: End-to-end metric exchange
3. **Performance Tests**: Impact on throughput and latency
4. **Compatibility Tests**: Backward compatibility verification
5. **Field Tests**: Real-world deployment validation

### 12. Conclusion

Adding network metrics to the Reticulum protocol would significantly enhance its capabilities for mesh networking applications. The proposed extension is:
- **Backward compatible**: Existing implementations continue to work
- **Optional**: Nodes can implement metrics as needed
- **Extensible**: New metric types can be added in the future
- **Practical**: Provides real value for route selection and user experience

This extension would be particularly valuable for the Meshtastic Reticulum Bridge, enabling better integration with LoRa hardware and providing users with meaningful signal quality information.