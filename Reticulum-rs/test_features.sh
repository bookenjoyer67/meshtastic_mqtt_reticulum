#!/bin/bash

# Test script for Reticulum-rs interfaces
# This script tests that the MQTT, serial, KISS, and I2P interfaces compile and can be used

echo "=== Testing Reticulum-rs Interfaces ==="
echo

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Not in Reticulum-rs directory"
    exit 1
fi

echo "1. Testing compilation with MQTT feature..."
cargo check --features mqtt
if [ $? -eq 0 ]; then
    echo "✓ MQTT feature compiles successfully"
else
    echo "✗ MQTT feature compilation failed"
    exit 1
fi

echo
echo "2. Testing compilation with serial feature..."
cargo check --features serial
if [ $? -eq 0 ]; then
    echo "✓ Serial feature compiles successfully"
else
    echo "✗ Serial feature compilation failed"
    exit 1
fi

echo
echo "3. Testing compilation with kiss feature..."
cargo check --features kiss
if [ $? -eq 0 ]; then
    echo "✓ KISS feature compiles successfully"
else
    echo "✗ KISS feature compilation failed"
    exit 1
fi

echo
echo "4. Testing compilation with i2p feature..."
cargo check --features i2p
if [ $? -eq 0 ]; then
    echo "✓ I2P feature compiles successfully"
else
    echo "✗ I2P feature compilation failed"
    exit 1
fi

echo
echo "5. Testing compilation with all features..."
cargo check --features "serial,mqtt,kiss,i2p"
if [ $? -eq 0 ]; then
    echo "✓ All features compile successfully"
else
    echo "✗ All features compilation failed"
    exit 1
fi

echo
echo "6. Building MQTT client example..."
cargo build --example mqtt_client --features mqtt
if [ $? -eq 0 ]; then
    echo "✓ MQTT client example builds successfully"
    echo "  Run with: cargo run --example mqtt_client --features mqtt"
else
    echo "✗ MQTT client example build failed"
    exit 1
fi

echo
echo "7. Building serial client example..."
cargo build --example serial_client --features serial
if [ $? -eq 0 ]; then
    echo "✓ Serial client example builds successfully"
    echo "  Run with: cargo run --example serial_client --features serial"
else
    echo "✗ Serial client example build failed"
    exit 1
fi

echo
echo "8. Building KISS client example..."
cargo build --example kiss_client --features kiss
if [ $? -eq 0 ]; then
    echo "✓ KISS client example builds successfully"
    echo "  Run with: cargo run --example kiss_client --features kiss"
else
    echo "✗ KISS client example build failed"
    exit 1
fi

echo
echo "9. Building I2P client example..."
cargo build --example i2p_client --features i2p
if [ $? -eq 0 ]; then
    echo "✓ I2P client example builds successfully"
    echo "  Run with: cargo run --example i2p_client --features i2p"
else
    echo "✗ I2P client example build failed"
    exit 1
fi

echo
echo "10. Building configuration loader example..."
cargo build --example config_loader
if [ $? -eq 0 ]; then
    echo "✓ Configuration loader example builds successfully"
    echo "  Run with: cargo run --example config_loader"
else
    echo "✗ Configuration loader example build failed"
    exit 1
fi
echo
echo "=== Summary ==="
echo "✓ All tests passed!"
echo
echo "To run the MQTT client example:"
echo
echo "To run the serial client example:"
echo "  SERIAL_PORT=/dev/ttyUSB0 BAUD_RATE=115200 cargo run --example serial_client --features serial"
echo
echo "To run the KISS client example:"
echo "  KISS_PORT=/dev/ttyUSB0 KISS_BAUD_RATE=9600 cargo run --example kiss_client --features kiss"
echo
echo "To run the I2P client example:"
echo "  I2P_SAM_ADDRESS=127.0.0.1 I2P_SAM_PORT=7656 cargo run --example i2p_client --features i2p"
echo
echo "Note: You may need to adjust environment variables based on your setup."