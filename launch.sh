#!/usr/bin/env bash
# Cross-platform launcher for Meshtastic MQTT Reticulum Bridge
# Works on Linux, macOS, and Windows (via WSL or Git Bash)

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Platform detection
detect_platform() {
    case "$(uname -s)" in
        Linux*)     echo "linux" ;;
        Darwin*)    echo "macos" ;;
        CYGWIN*|MINGW*|MSYS*) echo "windows" ;;
        *)          echo "unknown" ;;
    esac
}

# Terminal detection
detect_terminal() {
    local platform="$1"
    
    case "$platform" in
        linux)
            # Try common Linux terminals
            if command -v gnome-terminal &> /dev/null; then
                echo "gnome-terminal"
            elif command -v konsole &> /dev/null; then
                echo "konsole"
            elif command -v xterm &> /dev/null; then
                echo "xterm"
            elif command -v alacritty &> /dev/null; then
                echo "alacritty"
            elif command -v kitty &> /dev/null; then
                echo "kitty"
            else
                echo "unknown"
            fi
            ;;
        macos)
            # macOS - use AppleScript to open Terminal
            echo "terminal"
            ;;
        windows)
            # Windows - try Windows Terminal or fall back to cmd
            if command -v wt &> /dev/null; then
                echo "windows-terminal"
            else
                echo "cmd"
            fi
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

# Open terminal function
open_terminal() {
    local platform="$1"
    local terminal="$2"
    local title="$3"
    local command="$4"
    
    case "$terminal" in
        gnome-terminal)
            gnome-terminal --title="$title" -- bash -c "$command; exec bash"
            ;;
        konsole)
            konsole --new-tab -e bash -c "$command; exec bash"
            ;;
        xterm)
            xterm -title "$title" -e bash -c "$command; exec bash"
            ;;
        alacritty)
            alacritty --title "$title" -e bash -c "$command"
            ;;
        kitty)
            kitty --title "$title" bash -c "$command"
            ;;
        terminal)
            # macOS Terminal
            osascript <<EOF
tell application "Terminal"
    do script "$command"
    set currentTab to the result
    set custom title of currentTab to "$title"
    activate
end tell
EOF
            ;;
        windows-terminal)
            wt --title "$title" bash -c "$command"
            ;;
        cmd)
            # Windows cmd - limited functionality
            echo "Opening $title on Windows cmd"
            start cmd /k "title $title && $command"
            ;;
        *)
            echo "No suitable terminal found for $platform"
            echo "Please run manually: $command"
            ;;
    esac
}

# Check if cargo is available
check_cargo() {
    if ! command -v cargo &> /dev/null; then
        echo "Error: cargo (Rust) not found in PATH"
        echo "Please install Rust from https://rustup.rs/"
        exit 1
    fi
}

# Check if project is built
check_build() {
    if [ ! -f "target/release/bridge" ] && [ ! -f "target/debug/bridge" ]; then
        echo "Project not built. Building in release mode..."
        cargo build --release
    fi
}

# Main launch function
launch_components() {
    local platform="$1"
    local terminal="$2"
    
    echo "Starting Meshtastic + Reticulum Bridge on $platform..."
    echo "Using terminal: $terminal"
    echo ""
    
    # Launch bridge
    echo "Launching Reticulum Bridge..."
    open_terminal "$platform" "$terminal" "Reticulum Bridge" "MQTT_USERNAME=dummy MQTT_PASSWORD=dummy cargo run --bin bridge"
    
    # Wait for bridge to start
    sleep 3
    
    # Launch relay (gateway)
    echo "Launching Relay (Gateway)..."
    open_terminal "$platform" "$terminal" "Relay (Gateway)" "cargo run --bin relay"
    
    # Wait a moment
    sleep 2
    
    # Launch GUI
    echo "Launching Meshtastic GUI..."
    open_terminal "$platform" "$terminal" "Meshtastic GUI" "cargo run --bin gui"
    
    echo ""
    echo "All components launched!"
    echo "Components:"
    echo "1. Reticulum Bridge - Connects to Reticulum network"
    echo "2. Relay (Gateway) - Bridges Reticulum and Meshtastic"
    echo "3. Meshtastic GUI - User interface"
    echo ""
    echo "Note: Close terminals to stop components."
}

# Individual component launchers
launch_bridge() {
    local platform="$1"
    local terminal="$2"
    open_terminal "$platform" "$terminal" "Reticulum Bridge" "MQTT_USERNAME=dummy MQTT_PASSWORD=dummy cargo run --bin bridge"
}

launch_relay() {
    local platform="$1"
    local terminal="$2"
    open_terminal "$platform" "$terminal" "Relay (Gateway)" "cargo run --bin relay"
}

launch_gui() {
    local platform="$1"
    local terminal="$2"
    open_terminal "$platform" "$terminal" "Meshtastic GUI" "cargo run --bin gui"
}

# Headless mode (no GUI terminals)
launch_headless() {
    echo "Starting headless mode..."
    echo "Launching bridge and relay in background..."
    
    # Run bridge in background
    MQTT_USERNAME=dummy MQTT_PASSWORD=dummy cargo run --bin bridge > bridge.log 2>&1 &
    BRIDGE_PID=$!
    echo "Bridge started (PID: $BRIDGE_PID)"
    
    # Wait for bridge
    sleep 3
    
    # Run relay in background
    cargo run --bin relay > relay.log 2>&1 &
    RELAY_PID=$!
    echo "Relay started (PID: $RELAY_PID)"
    
    echo ""
    echo "Headless mode started!"
    echo "Bridge PID: $BRIDGE_PID (logs: bridge.log)"
    echo "Relay PID: $RELAY_PID (logs: relay.log)"
    echo ""
    echo "To stop: kill $BRIDGE_PID $RELAY_PID"
    echo "To view logs: tail -f bridge.log relay.log"
}

# Show usage
show_usage() {
    echo "Meshtastic MQTT Reticulum Bridge Launcher"
    echo "Usage: $0 [OPTION]"
    echo ""
    echo "Options:"
    echo "  all              Launch all components (default)"
    echo "  bridge           Launch only the bridge"
    echo "  relay            Launch only the relay"
    echo "  gui              Launch only the GUI"
    echo "  headless         Launch bridge and relay in background (no GUI terminals)"
    echo "  help             Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0               # Launch all components"
    echo "  $0 headless      # Run in server/embedded mode"
    echo "  $0 bridge        # Run only the bridge component"
    echo ""
    echo "Environment variables:"
    echo "  MQTT_USERNAME    MQTT broker username (required)"
    echo "  MQTT_PASSWORD    MQTT broker password (required)"
    echo "  MQTT_USE_TLS     Use TLS (default: true)"
    echo "  See CONFIGURATION_GUIDE.md for full configuration"
}

# Main script
main() {
    # Check for cargo
    check_cargo
    
    # Detect platform
    PLATFORM=$(detect_platform)
    TERMINAL=$(detect_terminal "$PLATFORM")
    
    # Check build
    check_build
    
    # Parse command line argument
    COMMAND="${1:-all}"
    
    case "$COMMAND" in
        all)
            launch_components "$PLATFORM" "$TERMINAL"
            ;;
        bridge)
            launch_bridge "$PLATFORM" "$TERMINAL"
            ;;
        relay)
            launch_relay "$PLATFORM" "$TERMINAL"
            ;;
        gui)
            launch_gui "$PLATFORM" "$TERMINAL"
            ;;
        headless)
            launch_headless
            ;;
        help|--help|-h)
            show_usage
            ;;
        *)
            echo "Unknown command: $COMMAND"
            show_usage
            exit 1
            ;;
    esac
}

# Run main function
main "$@"