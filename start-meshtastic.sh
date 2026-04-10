#!/bin/bash
# Start the Meshtastic + Reticulum bridge system

cd /home/computing/mqtt\ rust/meshtastic_mqtt_reticulum

# Open a new terminal for the bridge
gnome-terminal --title="Reticulum Bridge" -- bash -c "MQTT_USERNAME=dummy MQTT_PASSWORD=dummy cargo run --bin bridge; exec bash"

# Wait a moment for the bridge to start
sleep 2

# Open a new terminal for the relay
gnome-terminal --title="Relay (Gateway)" -- bash -c "cargo run --bin relay; exec bash"

# Open a new terminal for the GUI
gnome-terminal --title="Meshtastic GUI" -- bash -c "cargo run --bin gui; exec bash"