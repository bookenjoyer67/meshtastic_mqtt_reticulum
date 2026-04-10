# Cross-Platform Compatibility Guide

This guide explains how to use the Meshtastic MQTT Reticulum Bridge on different platforms and devices.

## Supported Platforms

### ✅ **Fully Supported**
- **Linux** (x86_64, ARM64) - Primary development platform
- **Windows 10/11** (x86_64) - Via cross-platform launchers
- **macOS** (x86_64, ARM64) - With minor adjustments

### 🟡 **Partially Supported**
- **Android** - Requires separate mobile app (planned)
- **iOS** - Requires separate mobile app (planned)
- **Raspberry Pi** - ARM architecture works, GUI may be slow
- **Embedded Linux** - Headless mode works, resource constraints

### ❌ **Not Supported**
- **Web browsers** - No web interface
- **Microcontrollers** - Too resource-intensive
- **Terminal-only** - GUI is required for full functionality

## Quick Start

### Linux / macOS
```bash
# Make launcher executable
chmod +x launch.sh

# Launch all components
./launch.sh

# Or launch specific components
./launch.sh bridge      # Only bridge
./launch.sh relay       # Only relay  
./launch.sh gui         # Only GUI
./launch.sh headless    # Server/embedded mode
```

### Windows
```batch
# Double-click launch.bat or run from command prompt
launch.bat

# Or with arguments
launch.bat bridge
launch.bat relay
launch.bat gui
launch.bat headless
```

## Platform-Specific Notes

### Linux
- **Recommended:** Ubuntu 20.04+, Debian 11+, Fedora 35+
- **GUI:** Works with GNOME, KDE, XFCE, and other desktop environments
- **Terminals:** Supports gnome-terminal, konsole, xterm, alacritty, kitty
- **Permissions:** Standard user permissions sufficient

### Windows
- **Requirements:** Windows 10/11, Rust installed via rustup
- **Terminal:** Uses Windows Terminal (recommended) or Command Prompt
- **Paths:** Use forward slashes in configuration files
- **Firewall:** May need to allow Rust executables

### macOS
- **Requirements:** macOS 10.15+, Xcode Command Line Tools
- **Terminal:** Uses native Terminal.app
- **Permissions:** May need Accessibility permissions for GUI
- **Firewall:** May prompt for network permissions

### Raspberry Pi / ARM Devices
```bash
# Install Rust for ARM
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Add ARM target (for Raspberry Pi 3/4)
rustup target add aarch64-unknown-linux-gnu  # 64-bit
rustup target add armv7-unknown-linux-gnueabihf  # 32-bit

# Build for ARM
cargo build --release --target=aarch64-unknown-linux-gnu

# Run in headless mode (recommended for Pi)
./launch.sh headless
```

## Mobile Platform Support (Planned)

### Android
- **Status:** In development
- **Approach:** Separate Android app using platform-native UI
- **Features:** Touch interface, background services, battery optimization
- **Timeline:** Q2 2026

### iOS
- **Status:** Planned
- **Approach:** Separate iOS app using SwiftUI
- **Features:** Touch interface, Apple ecosystem integration
- **Timeline:** Q3 2026

## Embedded & LoRa Device Support

### Current Limitations
- No direct LoRa radio interface
- Requires external Meshtastic device
- GUI too resource-intensive for embedded

### Planned Improvements
1. **LoRa Hardware Abstraction Layer** - Direct radio support
2. **Headless Mode** - Reduced resource usage
3. **Embedded Build Profiles** - `no_std` compatibility
4. **Real-time Scheduling** - Timing-critical operations

### Target Embedded Platforms
- ESP32 with LoRa modules
- Raspberry Pi + LoRa HAT
- STM32-based LoRa devices
- nRF52 with LoRa add-ons

## Configuration Files

### Platform-Specific Locations
```rust
// Cross-platform configuration directory detection
use directories::ProjectDirs;

fn get_config_dir() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "meshtastic", "bridge") {
        // Platform-specific standard locations:
        // Linux:   ~/.config/meshtastic-bridge
        // Windows: C:\Users\Username\AppData\Roaming\com.meshtastic.bridge
        // macOS:   ~/Library/Application Support/com.meshtastic.bridge
        Some(proj_dirs.config_dir().to_path_buf())
    } else {
        // Fallback for embedded/unsupported platforms
        Some(PathBuf::from("./config"))
    }
}
```

### Environment Variables
All platforms use the same environment variables:
```bash
# Linux/macOS
export MQTT_USERNAME="your_username"
export MQTT_PASSWORD="your_password"
./launch.sh

# Windows (Command Prompt)
set MQTT_USERNAME=your_username
set MQTT_PASSWORD=your_password
launch.bat

# Windows (PowerShell)
$env:MQTT_USERNAME="your_username"
$env:MQTT_PASSWORD="your_password"
.\launch.bat
```

## Building from Source

### Cross-Compilation
```bash
# Install cross-compilation tools
rustup target add x86_64-pc-windows-gnu
rustup target add aarch64-apple-darwin
rustup target add armv7-unknown-linux-gnueabihf

# Build for Windows from Linux
cargo build --release --target=x86_64-pc-windows-gnu

# Build for macOS from Linux (requires osxcross)
cargo build --release --target=aarch64-apple-darwin

# Build for ARM Linux from x86_64
cargo build --release --target=armv7-unknown-linux-gnueabihf
```

### Feature Flags
Add to `Cargo.toml` for platform-specific builds:
```toml
[features]
default = ["gui", "tls", "json"]
gui = ["eframe", "egui", "winit"]      # Desktop GUI
headless = []                          # No GUI, server mode
mobile = ["gui", "touch"]              # Mobile-optimized GUI
embedded = ["no_std", "alloc"]         # Embedded systems
lora = ["radio-drivers"]               # LoRa hardware support
```

## Troubleshooting

### Common Issues

#### Linux: "Cannot open display"
```bash
# Run in headless mode
./launch.sh headless

# Or set DISPLAY variable
export DISPLAY=:0
./launch.sh
```

#### Windows: "cargo not found"
```batch
# Install Rust via rustup
# Restart Command Prompt after installation
# Verify installation:
cargo --version
```

#### macOS: "App can't be opened"
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine launch.sh
xattr -d com.apple.quarantine target/release/*

# Or build from source
cargo build --release
```

#### ARM Devices: Build failures
```bash
# Install cross-compilation dependencies
sudo apt-get install gcc-aarch64-linux-gnu  # For ARM64
sudo apt-get install gcc-arm-linux-gnueabihf  # For ARMv7

# Use correct target
cargo build --release --target=aarch64-unknown-linux-gnu
```

### Performance Optimization

#### Low-end Computers
```bash
# Use headless mode
./launch.sh headless

# Reduce logging verbosity
export RUST_LOG=error
```

#### Mobile Devices (Future)
- Use battery-optimized network polling
- Implement background service restrictions
- Add data-saving mode

#### Embedded Systems
- Disable GUI features
- Use memory pooling
- Implement sleep/wake cycles

## Network Compatibility

### Supported Protocols
- **MQTT 3.1.1** with TLS (port 8883)
- **TCP** for Reticulum connections
- **JSON** for internal communication
- **Protobuf** for Meshtastic protocol

### Network Restrictions
- **Mobile networks:** May restrict background connections
- **Corporate networks:** May block MQTT ports
- **Firewalls:** May need port exceptions
- **VPNs:** Should work with proper routing

### Connection Testing
```bash
# Test MQTT connection
mosquitto_sub -h mqtt.meshtastic.org -p 8883 -t "test" -u "test" -P "test"

# Test Reticulum connection
nc -zv RNS.MichMesh.net 7822
```

## Security Considerations

### Platform-Specific Security

#### Linux/macOS
- File permissions (chmod 600 for config files)
- User account isolation
- Firewall configuration (ufw/iptables)

#### Windows
- Windows Defender exceptions
- User Account Control (UAC)
- Windows Firewall rules

#### Mobile (Future)
- App sandboxing
- Permission requests
- Keychain/Keystore storage

#### Embedded
- Secure boot verification
- Hardware security modules
- Encrypted storage

## Contributing to Platform Support

### Adding New Platform Support
1. **Platform detection** in `launch.sh`/`launch.bat`
2. **Conditional compilation** with `#[cfg(target_os)]`
3. **Platform-specific modules** in `src/platform/`
4. **CI testing** for the new platform

### Testing Matrix
We need testing on:
- [x] Ubuntu 22.04 (x86_64)
- [ ] Windows 11 (x86_64)
- [ ] macOS 13+ (ARM64)
- [ ] Raspberry Pi OS (ARMv7/ARM64)
- [ ] Android 13+ (planned)
- [ ] iOS 16+ (planned)

### Reporting Issues
When reporting platform-specific issues, include:
1. Platform and version
2. Architecture (x86_64, ARM64, etc.)
3. Terminal/desktop environment
4. Error messages and logs
5. Steps to reproduce

## Future Roadmap

### Short-term (Q2 2026)
- [x] Cross-platform launchers
- [ ] Windows testing and fixes
- [ ] macOS testing and fixes
- [ ] ARM build verification

### Medium-term (Q3 2026)
- [ ] Mobile app prototypes
- [ ] Headless mode improvements
- [ ] LoRa hardware abstraction
- [ ] Web interface exploration

### Long-term (Q4 2026+)
- [ ] Full mobile app releases
- [ ] Embedded deployment guides
- [ ] Cross-compilation CI
- [ ] Platform-specific optimizations

## Getting Help

- **GitHub Issues:** For bug reports and feature requests
- **Documentation:** Check `CONFIGURATION_GUIDE.md` and `DEVICE_COMPATIBILITY_ANALYSIS.md`
- **Community:** Meshtastic and Reticulum forums
- **Development:** Contributor guidelines in repository

---

*Last updated: 2026-03-31*  
*Compatibility version: 1.0*