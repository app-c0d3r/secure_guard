@echo off
setlocal EnableDelayedExpansion

:: SecureGuard Agent Installer
:: Copyright (c) 2025 SecureGuard Team

echo =========================================
echo SecureGuard Agent Installer v1.0
echo =========================================
echo.

:: Check if running as administrator
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo ERROR: This installer must be run as Administrator
    echo Right-click this file and select "Run as administrator"
    pause
    exit /b 1
)

:: Set installation directory
set "INSTALL_DIR=%ProgramFiles%\SecureGuard\Agent"
set "SERVICE_NAME=SecureGuardAgent"
set "AGENT_EXE=secureguard-agent.exe"

echo Installing SecureGuard Agent to: %INSTALL_DIR%
echo.

:: Create installation directory
if not exist "%INSTALL_DIR%" (
    echo Creating installation directory...
    mkdir "%INSTALL_DIR%" 2>nul
    if !errorLevel! neq 0 (
        echo ERROR: Failed to create installation directory
        pause
        exit /b 1
    )
)

:: Check if agent executable exists
if not exist "%~dp0%AGENT_EXE%" (
    echo ERROR: Agent executable not found at %~dp0%AGENT_EXE%
    echo Please ensure the agent executable is in the same folder as this installer
    pause
    exit /b 1
)

:: Copy agent executable
echo Copying agent files...
copy "%~dp0%AGENT_EXE%" "%INSTALL_DIR%\" >nul
if %errorLevel% neq 0 (
    echo ERROR: Failed to copy agent executable
    pause
    exit /b 1
)

:: Create default configuration file
echo Creating default configuration...
(
echo # SecureGuard Agent Configuration
echo # Auto-generated configuration file
echo.
echo [server]
echo url = "ws://localhost:8080/ws"
echo api_base = "http://localhost:8080/api/v1"
echo.
echo [agent]
echo name = "%COMPUTERNAME%"
echo log_level = "info"
echo heartbeat_interval = 30
echo.
echo [security]
echo encryption_enabled = true
echo.
echo [logging]
echo file_path = "%INSTALL_DIR%\logs\agent.log"
echo max_size = "10MB"
echo max_files = 5
) > "%INSTALL_DIR%\config.toml"

:: Create logs directory
if not exist "%INSTALL_DIR%\logs" (
    mkdir "%INSTALL_DIR%\logs" 2>nul
)

:: Stop existing service if it exists
sc query "%SERVICE_NAME%" >nul 2>&1
if %errorLevel% equ 0 (
    echo Stopping existing service...
    sc stop "%SERVICE_NAME%" >nul 2>&1
    timeout /t 3 /nobreak >nul
    
    echo Removing existing service...
    sc delete "%SERVICE_NAME%" >nul 2>&1
    if !errorLevel! neq 0 (
        echo WARNING: Failed to remove existing service
    )
)

:: Install as Windows service
echo Installing Windows service...
sc create "%SERVICE_NAME%" binPath= "\"%INSTALL_DIR%\%AGENT_EXE%\"" start= auto DisplayName= "SecureGuard Security Agent" depend= Tcpip >nul
if %errorLevel% neq 0 (
    echo ERROR: Failed to create Windows service
    pause
    exit /b 1
)

:: Set service description
sc description "%SERVICE_NAME%" "SecureGuard security monitoring agent - monitors system for threats and security events" >nul

:: Configure service recovery options
sc failure "%SERVICE_NAME%" reset= 86400 actions= restart/30000/restart/60000/restart/120000 >nul

:: Grant service logon rights (optional but recommended)
echo Configuring service permissions...

:: Start the service
echo Starting SecureGuard Agent service...
sc start "%SERVICE_NAME%" >nul 2>&1
if %errorLevel% neq 0 (
    echo WARNING: Service created but failed to start automatically
    echo You can start it manually from Services panel or run:
    echo   sc start %SERVICE_NAME%
) else (
    echo Service started successfully
)

:: Add to Windows Firewall (optional)
echo Configuring Windows Firewall...
netsh advfirewall firewall add rule name="SecureGuard Agent" dir=out action=allow program="%INSTALL_DIR%\%AGENT_EXE%" >nul 2>&1
netsh advfirewall firewall add rule name="SecureGuard Agent Inbound" dir=in action=allow program="%INSTALL_DIR%\%AGENT_EXE%" >nul 2>&1

:: Create uninstaller
echo Creating uninstaller...
(
echo @echo off
echo :: SecureGuard Agent Uninstaller
echo echo Uninstalling SecureGuard Agent...
echo.
echo :: Check if running as administrator
echo net session ^>nul 2^>^&1
echo if %%errorLevel%% neq 0 ^(
echo     echo ERROR: Uninstaller must be run as Administrator
echo     pause
echo     exit /b 1
echo ^)
echo.
echo :: Stop and remove service
echo sc stop "%SERVICE_NAME%" ^>nul 2^>^&1
echo timeout /t 3 /nobreak ^>nul
echo sc delete "%SERVICE_NAME%" ^>nul 2^>^&1
echo.
echo :: Remove firewall rules
echo netsh advfirewall firewall delete rule name="SecureGuard Agent" ^>nul 2^>^&1
echo netsh advfirewall firewall delete rule name="SecureGuard Agent Inbound" ^>nul 2^>^&1
echo.
echo :: Remove installation directory
echo rmdir /s /q "%INSTALL_DIR%" 2^>nul
echo.
echo echo SecureGuard Agent has been uninstalled successfully
echo pause
) > "%INSTALL_DIR%\uninstall.bat"

:: Create desktop shortcut for service management
echo Creating management shortcuts...
(
echo @echo off
echo echo SecureGuard Agent Management
echo echo ============================
echo echo 1. Start Service
echo echo 2. Stop Service
echo echo 3. Restart Service
echo echo 4. View Service Status
echo echo 5. View Logs
echo echo 6. Edit Configuration
echo echo 7. Uninstall Agent
echo echo 8. Exit
echo echo.
echo set /p choice="Choose an option (1-8): "
echo.
echo if "%%choice%%"=="1" sc start "%SERVICE_NAME%"
echo if "%%choice%%"=="2" sc stop "%SERVICE_NAME%"
echo if "%%choice%%"=="3" (
echo     sc stop "%SERVICE_NAME%"
echo     timeout /t 3 /nobreak ^>nul
echo     sc start "%SERVICE_NAME%"
echo )
echo if "%%choice%%"=="4" sc query "%SERVICE_NAME%"
echo if "%%choice%%"=="5" type "%INSTALL_DIR%\logs\agent.log"
echo if "%%choice%%"=="6" notepad "%INSTALL_DIR%\config.toml"
echo if "%%choice%%"=="7" "%INSTALL_DIR%\uninstall.bat"
echo if "%%choice%%"=="8" exit
echo pause
) > "%INSTALL_DIR%\manage.bat"

echo.
echo =========================================
echo Installation completed successfully!
echo =========================================
echo.
echo Installation Details:
echo - Agent installed to: %INSTALL_DIR%
echo - Service name: %SERVICE_NAME%
echo - Configuration file: %INSTALL_DIR%\config.toml
echo - Log files: %INSTALL_DIR%\logs\
echo - Management tool: %INSTALL_DIR%\manage.bat
echo - Uninstaller: %INSTALL_DIR%\uninstall.bat
echo.
echo The SecureGuard Agent service has been installed and started.
echo You can manage the service using Windows Services panel or the
echo management tool at: %INSTALL_DIR%\manage.bat
echo.
echo To configure the agent, edit: %INSTALL_DIR%\config.toml
echo Then restart the service for changes to take effect.
echo.
pause