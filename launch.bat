@echo off
REM Windows launcher for Meshtastic MQTT Reticulum Bridge
REM Works on Windows 10/11 with Rust installed

setlocal enabledelayedexpansion

REM Get the directory where this script is located
set "SCRIPT_DIR=%~dp0"
cd /d "%SCRIPT_DIR%"

REM Check if cargo is available
where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: cargo (Rust) not found in PATH
    echo Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

REM Check if project is built
if not exist "target\release\bridge.exe" (
    if not exist "target\debug\bridge.exe" (
        echo Project not built. Building in release mode...
        call cargo build --release
    )
)

REM Function to open new command window
:open_cmd
set "title=%~1"
set "command=%~2"
set "unique_id=%random%%random%"
start "%title%" cmd /k "title %title% && %command%"
exit /b 0

REM Main launch function
:launch_all
echo Starting Meshtastic + Reticulum Bridge on Windows...
echo.

echo Launching Reticulum Bridge...
call :open_cmd "Reticulum Bridge" "cargo run --bin bridge"

REM Wait for bridge to start
timeout /t 3 /nobreak >nul

echo Launching Relay (Gateway)...
call :open_cmd "Relay (Gateway)" "cargo run --bin relay"

REM Wait a moment
timeout /t 2 /nobreak >nul

echo Launching Meshtastic GUI...
call :open_cmd "Meshtastic GUI" "cargo run --bin gui"

echo.
echo All components launched!
echo Components:
echo 1. Reticulum Bridge - Connects to Reticulum network
echo 2. Relay (Gateway) - Bridges Reticulum and Meshtastic
echo 3. Meshtastic GUI - User interface
echo.
echo Note: Close command windows to stop components.
goto :eof

REM Individual component launchers
:launch_bridge
call :open_cmd "Reticulum Bridge" "cargo run --bin bridge"
goto :eof

:launch_relay
call :open_cmd "Relay (Gateway)" "cargo run --bin relay"
goto :eof

:launch_gui
call :open_cmd "Meshtastic GUI" "cargo run --bin gui"
goto :eof

REM Headless mode (background processes)
:launch_headless
echo Starting headless mode...
echo Launching bridge and relay in background...
echo.

REM Run bridge in background
start /B "Reticulum Bridge" cargo run --bin bridge > bridge.log 2>&1
for /f "tokens=2" %%i in ('tasklist /fi "windowtitle eq Reticulum Bridge*" /fo csv /nh') do set "BRIDGE_PID=%%~i"
echo Bridge started (PID: !BRIDGE_PID!)

REM Wait for bridge
timeout /t 3 /nobreak >nul

REM Run relay in background
start /B "Relay (Gateway)" cargo run --bin relay > relay.log 2>&1
for /f "tokens=2" %%i in ('tasklist /fi "windowtitle eq Relay (Gateway)*" /fo csv /nh') do set "RELAY_PID=%%~i"
echo Relay started (PID: !RELAY_PID!)

echo.
echo Headless mode started!
echo Bridge PID: !BRIDGE_PID! (logs: bridge.log)
echo Relay PID: !RELAY_PID! (logs: relay.log)
echo.
echo To stop: taskkill /PID !BRIDGE_PID! /PID !RELAY_PID!
echo To view logs: type bridge.log or type relay.log
goto :eof

REM Show usage
:show_usage
echo Meshtastic MQTT Reticulum Bridge Launcher
echo Usage: %~n0 [OPTION]
echo.
echo Options:
echo   all              Launch all components (default)
echo   bridge           Launch only the bridge
echo   relay            Launch only the relay
echo   gui              Launch only the GUI
echo   headless         Launch bridge and relay in background
echo   help             Show this help message
echo.
echo Examples:
echo   %~n0             Launch all components
echo   %~n0 headless    Run in server/embedded mode
echo   %~n0 bridge      Run only the bridge component
echo.
echo Environment variables:
echo   MQTT_USERNAME    MQTT broker username (required)
echo   MQTT_PASSWORD    MQTT broker password (required)
echo   MQTT_USE_TLS     Use TLS (default: true)
echo   See CONFIGURATION_GUIDE.md for full configuration
goto :eof

REM Main script
set "COMMAND=%~1"
if "%COMMAND%"=="" set "COMMAND=all"

if "%COMMAND%"=="all" (
    call :launch_all
) else if "%COMMAND%"=="bridge" (
    call :launch_bridge
) else if "%COMMAND%"=="relay" (
    call :launch_relay
) else if "%COMMAND%"=="gui" (
    call :launch_gui
) else if "%COMMAND%"=="headless" (
    call :launch_headless
) else if "%COMMAND%"=="help" (
    call :show_usage
) else (
    echo Unknown command: %COMMAND%
    call :show_usage
    exit /b 1
)

pause
exit /b 0