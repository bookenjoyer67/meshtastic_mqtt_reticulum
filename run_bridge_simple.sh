#!/bin/bash
# Script to run the bridge binary with dummy MQTT credentials
# This bypasses the validation while keeping MQTT disabled

export MQTT_USERNAME="dummy"
export MQTT_PASSWORD="dummy"
export MQTT_HOST=""
export RETICULUM_SERVER="${RETICULUM_SERVER:-RNS.MichMesh.net:7822}"
export GUI_BIND_ADDRESS="${GUI_BIND_ADDRESS:-127.0.0.1}"
export GUI_PORT="${GUI_PORT:-4243}"
export LOG_TO_CONSOLE="${LOG_TO_CONSOLE:-true}"
export LOG_TO_FILE="${LOG_TO_FILE:-true}"

echo "Running bridge with dummy MQTT credentials..."
echo "MQTT is disabled (empty host), but credentials are provided to bypass validation"
echo "Reticulum server: $RETICULUM_SERVER"
echo "GUI bind address: $GUI_BIND_ADDRESS:$GUI_PORT"

# Run the bridge binary
exec ./target/debug/bridge