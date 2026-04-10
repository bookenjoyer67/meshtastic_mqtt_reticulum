#!/bin/bash
# Script to run the bridge binary without needing MQTT credentials
# This sets MQTT_HOST to empty string, which disables MQTT validation

export MQTT_HOST=""
export RETICULUM_SERVER="${RETICULUM_SERVER:-RNS.MichMesh.net:7822}"
export GUI_BIND_ADDRESS="${GUI_BIND_ADDRESS:-127.0.0.1}"
export GUI_PORT="${GUI_PORT:-4243}"
export LOG_TO_CONSOLE="${LOG_TO_CONSOLE:-true}"
export LOG_TO_FILE="${LOG_TO_FILE:-true}"

echo "Running bridge without MQTT credentials..."
echo "MQTT_HOST is set to empty string (MQTT disabled)"
echo "Reticulum server: $RETICULUM_SERVER"
echo "GUI bind address: $GUI_BIND_ADDRESS:$GUI_PORT"

# Run the bridge binary
exec ./target/debug/bridge