@echo off
echo SecureGuard Agent Installer Build
echo =================================
echo.

:: Check if running as administrator
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo This script should be run as Administrator for best results
    echo.
)

:: Build the Rust agent first
echo Building Rust agent...
cargo build --release -p secureguard-agent
if %errorLevel% neq 0 (
    echo ERROR: Failed to build agent
    pause
    exit /b 1
)

:: Run the PowerShell build script
echo.
echo Building installers...
powershell -ExecutionPolicy Bypass -File "scripts\build-installer.ps1" -BuildType All -Configuration Release

echo.
echo Build complete! Check the 'dist' folder for installers.
pause