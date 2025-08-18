@echo off
setlocal enabledelayedexpansion

REM SecureGuard Service Control Script
REM Usage: myservice.bat [start|stop] [dev|prod]

set ACTION=%1
set ENVIRONMENT=%2

REM Default to dev if no environment specified
if "%ENVIRONMENT%"=="" set ENVIRONMENT=dev

REM Validate arguments
if "%ACTION%"=="" (
    echo Usage: myservice.bat [start^|stop] [dev^|prod]
    echo Examples:
    echo   myservice.bat start dev    - Start development environment
    echo   myservice.bat start prod   - Start production environment  
    echo   myservice.bat stop dev     - Stop development environment
    echo   myservice.bat stop prod    - Stop production environment
    exit /b 1
)

if not "%ACTION%"=="start" if not "%ACTION%"=="stop" (
    echo ERROR: Action must be 'start' or 'stop'
    exit /b 1
)

if not "%ENVIRONMENT%"=="dev" if not "%ENVIRONMENT%"=="prod" (
    echo ERROR: Environment must be 'dev' or 'prod'
    exit /b 1
)

echo SecureGuard Service Control - %ACTION% %ENVIRONMENT%
echo.

if "%ACTION%"=="start" (
    call :start_%ENVIRONMENT%
) else (
    call :stop_%ENVIRONMENT%
)
exit /b %ERRORLEVEL%

:start_dev
echo [DEV] Starting SecureGuard Development Environment...
set DATABASE_URL=postgresql://secureguard:password@localhost:5432/secureguard_dev
set RUST_LOG=secureguard_api=debug,tower_http=debug,axum=debug
set NODE_ENV=development

echo [1/3] Starting PostgreSQL Database (Development)...
cd /d "%~dp0.."
docker-compose up -d db
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Failed to start database
    exit /b 1
)

echo [2/3] Starting Rust Backend Server (Debug Mode)...
start "SecureGuard API [DEV]" cmd /k "cd /d "%~dp0.." && cd crates\secureguard-api && cargo run"

echo [3/3] Starting React Dashboard (Development)...
timeout /t 5 /nobreak > nul
start "SecureGuard Dashboard [DEV]" cmd /k "cd /d "%~dp0.." && cd dashboard && set PORT=3002 && npm start"

echo.
echo ✅ Development Environment Started
echo 🔗 Dashboard: http://localhost:3002
echo 🔗 API: http://localhost:3000/api
echo 📊 Database: localhost:5432 (secureguard_dev)
goto :eof

:start_prod
echo [PROD] Starting SecureGuard Production Environment...
set DATABASE_URL=postgresql://secureguard:password@localhost:5432/secureguard_prod
set RUST_LOG=secureguard_api=info
set NODE_ENV=production

echo [1/3] Starting PostgreSQL Database (Production)...
cd /d "%~dp0.."
docker-compose -f docker-compose.prod.yml up -d db 2>nul || docker-compose up -d db
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Failed to start database
    exit /b 1
)

echo [2/3] Building and Starting Rust Backend Server (Release Mode)...
start "SecureGuard API [PROD]" cmd /k "cd /d "%~dp0.." && cd crates\secureguard-api && cargo run --release"

echo [3/3] Building and Starting React Dashboard (Production)...
timeout /t 5 /nobreak > nul
start "SecureGuard Dashboard [PROD]" cmd /k "cd /d "%~dp0.." && cd dashboard && npm run build && npx serve -s build -l 3002"

echo.
echo ✅ Production Environment Started
echo 🔗 Dashboard: http://localhost:3002
echo 🔗 API: http://localhost:3000/api
echo 📊 Database: localhost:5432 (secureguard_prod)
goto :eof

:stop_dev
echo [DEV] Stopping Development Environment...
call :kill_processes
echo ✅ Development Environment Stopped
goto :eof

:stop_prod  
echo [PROD] Stopping Production Environment...
call :kill_processes
echo ✅ Production Environment Stopped
goto :eof

:kill_processes
echo Stopping React Dashboard (port 3002)...
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :3002') do (
    taskkill /f /pid %%a 2>nul
)

echo Stopping Rust Backend Server (port 3000)...
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :3000') do (
    taskkill /f /pid %%a 2>nul
)

echo Stopping PostgreSQL Database...
docker-compose down 2>nul
docker-compose -f docker-compose.prod.yml down 2>nul
goto :eof