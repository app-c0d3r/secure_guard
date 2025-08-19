@echo off
setlocal enabledelayedexpansion

REM SecureGuard Cross-Platform Service Control Script - Windows Edition
REM Usage: myservice.bat [start|stop|dev|help]

REM Script configuration
set FRONTEND_PORT=3002
set BACKEND_PORT=3000
set DB_PORT=5432
set ACTION=%1

REM Show help if no parameters provided
if "%ACTION%"=="" (
    call :show_help
    exit /b 0
)

REM Validate arguments and execute
if "%ACTION%"=="start" (
    call :check_ports_and_cleanup
    call :check_docker
    call :start_prod
    exit /b %ERRORLEVEL%
)

if "%ACTION%"=="stop" (
    call :stop_all
    exit /b %ERRORLEVEL%
)

if "%ACTION%"=="dev" (
    call :check_ports_and_cleanup
    call :check_docker
    call :start_dev
    exit /b %ERRORLEVEL%
)

if "%ACTION%"=="help" (
    call :show_help
    exit /b 0
)

REM Invalid command
echo ❌ ERROR: Invalid command '%ACTION%'
echo.
call :show_help
exit /b 1

:show_help
echo.
echo ╔══════════════════════════════════════════════════════════════════════════╗
echo ║                       SecureGuard Service Control                       ║
echo ╚══════════════════════════════════════════════════════════════════════════╝
echo.
echo USAGE:
echo   myservice [COMMAND]
echo.
echo COMMANDS:
echo   start     🚀 Start production environment (optimized builds)
echo   dev       🔧 Start development environment (debug mode + hot reload)
echo   stop      ⛔ Stop all environments (development + production)
echo   help      ❓ Show this help message
echo.
echo EXAMPLES:
echo   myservice start          Start production environment
echo   myservice dev            Start development environment  
echo   myservice stop           Stop all services
echo   myservice                Show this help (same as 'myservice help')
echo.
echo ENVIRONMENTS:
echo   📦 Production  - Uses release builds, production database
echo   🔧 Development - Uses debug builds, dev database, hot reload
echo.
echo SERVICES MANAGED:
echo   🐘 PostgreSQL Database   (Port %DB_PORT% - Docker container)
echo   🦀 Rust Backend API      (Port %BACKEND_PORT%)
echo   ⚛️  React Frontend        (Port %FRONTEND_PORT%)
echo.
echo FEATURES:
echo   ✅ Automatic port conflict detection and cleanup
echo   ✅ Docker auto-start and health checking  
echo   ✅ Smart process management
echo   ✅ Cross-platform compatibility
echo.
goto :eof

:check_ports_and_cleanup
echo.
echo ╔══════════════════════════════════════════════════════════════════════════╗
echo ║                     🔍 CHECKING FOR PORT CONFLICTS                      ║
echo ╚══════════════════════════════════════════════════════════════════════════╝
call :check_and_kill_port %FRONTEND_PORT% "React Frontend"
call :check_and_kill_port %BACKEND_PORT% "Rust Backend API"
echo.
goto :eof

:check_and_kill_port
set PORT=%1
set SERVICE_NAME=%2
set SERVICE_NAME=%SERVICE_NAME:"=%

echo 🔍 Checking port %PORT% for %SERVICE_NAME%...

REM Check if port is in use
for /f "tokens=5" %%a in ('netstat -aon ^| findstr ":%PORT% "') do (
    set PID=%%a
    if defined PID (
        echo ⚠️  Port %PORT% is occupied by PID !PID! - killing %SERVICE_NAME% process...
        taskkill /f /pid !PID! >nul 2>&1
        if !ERRORLEVEL! == 0 (
            echo ✅ Successfully stopped existing %SERVICE_NAME% process
        ) else (
            echo ❌ Failed to stop process !PID! - you may need to stop it manually
        )
    )
)

REM Double-check the port is now free
timeout /t 2 /nobreak >nul
for /f "tokens=5" %%a in ('netstat -aon ^| findstr ":%PORT% " 2^>nul') do (
    echo ⚠️  Port %PORT% is still in use - please check manually
    goto :eof
)
echo ✅ Port %PORT% is now available for %SERVICE_NAME%
goto :eof

:start_dev
echo.
echo ╔══════════════════════════════════════════════════════════════════════════╗
echo ║                    🔧 STARTING DEVELOPMENT ENVIRONMENT                   ║
echo ╚══════════════════════════════════════════════════════════════════════════╝
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

echo [3/3] Starting React Frontend (Development)...
timeout /t 5 /nobreak > nul
start "SecureGuard Frontend [DEV]" cmd /k "cd /d "%~dp0.." && cd frontend && set PORT=3002 && npm run dev"

echo.
echo ✅ Development Environment Started
echo 🔗 Frontend: http://localhost:3002 (React + Vite)
echo 🔗 API: http://localhost:3000/api
echo 📊 Database: localhost:5432 (secureguard_dev)
echo 🎨 Themes: Dark/Light mode available in header navigation
goto :eof

:start_prod
echo.
echo ╔══════════════════════════════════════════════════════════════════════════╗
echo ║                    🚀 STARTING PRODUCTION ENVIRONMENT                    ║
echo ╚══════════════════════════════════════════════════════════════════════════╝
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

echo [3/3] Building and Starting React Frontend (Production)...
timeout /t 5 /nobreak > nul
start "SecureGuard Frontend [PROD]" cmd /k "cd /d "%~dp0.." && cd frontend && npm run build && npm run preview -- --port 3002"

echo.
echo ✅ Production Environment Started
echo 🔗 Frontend: http://localhost:3002 (Production Build)
echo 🔗 API: http://localhost:3000/api
echo 📊 Database: localhost:5432 (secureguard_prod)
echo 🎨 Themes: Dark/Light mode available in header navigation
echo 🔐 Demo Login: admin@company.com / SecurePass123!
goto :eof

:stop_all
echo.
echo ╔══════════════════════════════════════════════════════════════════════════╗
echo ║                      ⛔ STOPPING ALL ENVIRONMENTS                        ║
echo ╚══════════════════════════════════════════════════════════════════════════╝
call :kill_processes
echo.
echo ✅ All SecureGuard services have been stopped
echo   🐘 PostgreSQL Database - Stopped
echo   🦀 Rust Backend API - Stopped  
echo   ⚛️  React Frontend - Stopped
goto :eof

:kill_processes
echo [1/3] Stopping React Frontend (port 3002)...
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :3002') do (
    taskkill /f /pid %%a 2>nul
)

echo [2/3] Stopping Rust Backend Server (port 3000)...
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :3000') do (
    taskkill /f /pid %%a 2>nul
)

echo [3/3] Stopping PostgreSQL Database (Docker containers)...
docker-compose down 2>nul
docker-compose -f docker-compose.prod.yml down 2>nul

REM Also kill any remaining node or cargo processes that might be running
taskkill /f /im node.exe 2>nul
taskkill /f /im cargo.exe 2>nul
goto :eof

:check_docker
echo Checking Docker Desktop status...
docker version >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo Docker Desktop is not running. Starting Docker Desktop...
    start "" "C:\Program Files\Docker\Docker\Docker Desktop.exe"
    echo Waiting for Docker Desktop to start up...
    :docker_wait
    timeout /t 5 /nobreak >nul
    docker version >nul 2>&1
    if %ERRORLEVEL% NEQ 0 (
        echo Still waiting for Docker Desktop...
        goto docker_wait
    )
    echo ✅ Docker Desktop is now running
) else (
    echo ✅ Docker Desktop is already running
)
goto :eof