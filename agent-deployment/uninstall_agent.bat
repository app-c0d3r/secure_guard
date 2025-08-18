@echo off
setlocal EnableDelayedExpansion

:: SecureGuard Agent Uninstaller
:: Copyright (c) 2025 SecureGuard Team

echo =========================================
echo SecureGuard Agent Uninstaller v1.0
echo =========================================
echo.

:: Check if running as administrator
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo ERROR: This uninstaller must be run as Administrator
    echo Right-click this file and select "Run as administrator"
    pause
    exit /b 1
)

:: Set variables
set "INSTALL_DIR=%ProgramFiles%\SecureGuard\Agent"
set "SERVICE_NAME=SecureGuardAgent"

echo This will completely remove SecureGuard Agent from your system.
echo Installation directory: %INSTALL_DIR%
echo.
set /p confirm="Are you sure you want to continue? (Y/N): "
if /i not "%confirm%"=="Y" (
    echo Uninstallation cancelled.
    pause
    exit /b 0
)

echo.
echo Uninstalling SecureGuard Agent...
echo.

:: Stop the service
echo Stopping SecureGuard Agent service...
sc query "%SERVICE_NAME%" >nul 2>&1
if %errorLevel% equ 0 (
    sc stop "%SERVICE_NAME%" >nul 2>&1
    if !errorLevel! equ 0 (
        echo Service stopped successfully
        timeout /t 3 /nobreak >nul
    ) else (
        echo Warning: Failed to stop service (may not be running)
    )
) else (
    echo Service not found or already removed
)

:: Remove the service
echo Removing Windows service...
sc delete "%SERVICE_NAME%" >nul 2>&1
if %errorLevel% equ 0 (
    echo Service removed successfully
) else (
    echo Warning: Failed to remove service (may already be removed)
)

:: Remove Windows Firewall rules
echo Removing firewall rules...
netsh advfirewall firewall delete rule name="SecureGuard Agent" >nul 2>&1
netsh advfirewall firewall delete rule name="SecureGuard Agent Inbound" >nul 2>&1
echo Firewall rules removed

:: Remove installation directory
echo Removing installation files...
if exist "%INSTALL_DIR%" (
    :: Force remove readonly and system attributes
    attrib -r -s -h "%INSTALL_DIR%\*" /s /d 2>nul
    
    :: Remove the directory
    rmdir /s /q "%INSTALL_DIR%" 2>nul
    if !errorLevel! equ 0 (
        echo Installation files removed successfully
    ) else (
        echo Warning: Some files could not be removed. Please manually delete:
        echo %INSTALL_DIR%
    )
) else (
    echo Installation directory not found (may already be removed)
)

:: Remove parent directory if empty
rmdir "%ProgramFiles%\SecureGuard" 2>nul >nul

:: Clean up registry entries (optional)
echo Cleaning up registry entries...
reg delete "HKLM\SYSTEM\CurrentControlSet\Services\%SERVICE_NAME%" /f >nul 2>&1

:: Remove any scheduled tasks (if any were created)
schtasks /delete /tn "SecureGuardAgent*" /f >nul 2>&1

:: Remove from startup (if it was added there)
reg delete "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run" /v "SecureGuardAgent" /f >nul 2>&1
reg delete "HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run" /v "SecureGuardAgent" /f >nul 2>&1

echo.
echo =========================================
echo Uninstallation completed successfully!
echo =========================================
echo.
echo SecureGuard Agent has been completely removed from your system:
echo - Service stopped and removed
echo - Installation files deleted
echo - Firewall rules removed
echo - Registry entries cleaned
echo.
echo If you want to reinstall SecureGuard Agent in the future,
echo simply run the installer again.
echo.
pause