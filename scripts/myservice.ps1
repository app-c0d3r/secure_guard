#!/usr/bin/env pwsh
param(
    [Parameter(Mandatory=$true)]
    [ValidateSet("start", "stop")]
    [string]$Action,
    
    [Parameter(Mandatory=$false)]
    [ValidateSet("dev", "prod")]
    [string]$Environment = "dev"
)

# SecureGuard Service Control Script
# Usage: .\myservice.ps1 [start|stop] [dev|prod]

function Show-Usage {
    Write-Host "Usage: .\myservice.ps1 [start|stop] [dev|prod]" -ForegroundColor Yellow
    Write-Host "Examples:" -ForegroundColor Cyan
    Write-Host "  .\myservice.ps1 start dev    - Start development environment" -ForegroundColor White
    Write-Host "  .\myservice.ps1 start prod   - Start production environment" -ForegroundColor White
    Write-Host "  .\myservice.ps1 stop dev     - Stop development environment" -ForegroundColor White
    Write-Host "  .\myservice.ps1 stop prod    - Stop production environment" -ForegroundColor White
}

function Start-DevEnvironment {
    Write-Host "[DEV] Starting SecureGuard Development Environment..." -ForegroundColor Green
    
    $env:DATABASE_URL = "postgresql://secureguard:password@localhost:5432/secureguard_dev"
    $env:RUST_LOG = "secureguard_api=debug,tower_http=debug,axum=debug"
    $env:NODE_ENV = "development"

    try {
        Write-Host "[1/3] Starting PostgreSQL Database (Development)..." -ForegroundColor Yellow
        docker-compose up -d db
        if ($LASTEXITCODE -ne 0) { throw "Failed to start database" }

        Write-Host "[2/3] Starting Rust Backend Server (Debug Mode)..." -ForegroundColor Yellow
        Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd 'crates\secureguard-api'; cargo run" -WindowStyle Normal

        Write-Host "[3/3] Starting React Dashboard (Development)..." -ForegroundColor Yellow
        Start-Sleep -Seconds 5
        Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd 'dashboard'; `$env:PORT='3002'; npm start" -WindowStyle Normal

        Write-Host ""
        Write-Host "✅ Development Environment Started" -ForegroundColor Green
        Write-Host "🔗 Dashboard: http://localhost:3002" -ForegroundColor Cyan
        Write-Host "🔗 API: http://localhost:3000/api" -ForegroundColor Cyan
        Write-Host "📊 Database: localhost:5432 (secureguard_dev)" -ForegroundColor Cyan
        
    } catch {
        Write-Host "ERROR: $_" -ForegroundColor Red
        exit 1
    }
}

function Start-ProdEnvironment {
    Write-Host "[PROD] Starting SecureGuard Production Environment..." -ForegroundColor Magenta
    
    $env:DATABASE_URL = "postgresql://secureguard:password@localhost:5432/secureguard_prod"
    $env:RUST_LOG = "secureguard_api=info"
    $env:NODE_ENV = "production"

    try {
        Write-Host "[1/3] Starting PostgreSQL Database (Production)..." -ForegroundColor Yellow
        # Try production compose file first, fallback to dev
        docker-compose -f docker-compose.prod.yml up -d db 2>$null
        if ($LASTEXITCODE -ne 0) {
            docker-compose up -d db
            if ($LASTEXITCODE -ne 0) { throw "Failed to start database" }
        }

        Write-Host "[2/3] Building and Starting Rust Backend Server (Release Mode)..." -ForegroundColor Yellow
        Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd 'crates\secureguard-api'; cargo run --release" -WindowStyle Normal

        Write-Host "[3/3] Building and Starting React Dashboard (Production)..." -ForegroundColor Yellow
        Start-Sleep -Seconds 5
        Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd 'dashboard'; npm run build; npx serve -s build -l 3002" -WindowStyle Normal

        Write-Host ""
        Write-Host "✅ Production Environment Started" -ForegroundColor Magenta
        Write-Host "🔗 Dashboard: http://localhost:3002" -ForegroundColor Cyan
        Write-Host "🔗 API: http://localhost:3000/api" -ForegroundColor Cyan
        Write-Host "📊 Database: localhost:5432 (secureguard_prod)" -ForegroundColor Cyan
        
    } catch {
        Write-Host "ERROR: $_" -ForegroundColor Red
        exit 1
    }
}

function Stop-Environment {
    param([string]$EnvType)
    
    Write-Host "[$($EnvType.ToUpper())] Stopping $EnvType Environment..." -ForegroundColor Red
    
    try {
        Write-Host "Stopping React Dashboard (port 3002)..." -ForegroundColor Gray
        $dashboardProcesses = Get-NetTCPConnection -LocalPort 3002 -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess
        foreach ($pid in $dashboardProcesses) {
            Stop-Process -Id $pid -Force -ErrorAction SilentlyContinue
        }

        Write-Host "Stopping Rust Backend Server (port 3000)..." -ForegroundColor Gray
        $backendProcesses = Get-NetTCPConnection -LocalPort 3000 -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess
        foreach ($pid in $backendProcesses) {
            Stop-Process -Id $pid -Force -ErrorAction SilentlyContinue
        }

        Write-Host "Stopping PostgreSQL Database..." -ForegroundColor Gray
        docker-compose down 2>$null
        docker-compose -f docker-compose.prod.yml down 2>$null

        Write-Host "✅ $($EnvType.ToUpper()) Environment Stopped" -ForegroundColor Green
        
    } catch {
        Write-Host "ERROR: $_" -ForegroundColor Red
        exit 1
    }
}

# Main script logic
Write-Host "SecureGuard Service Control - $Action $Environment" -ForegroundColor Cyan
Write-Host ""

switch ($Action) {
    "start" {
        switch ($Environment) {
            "dev" { Start-DevEnvironment }
            "prod" { Start-ProdEnvironment }
        }
    }
    "stop" {
        Stop-Environment -EnvType $Environment
    }
}

Write-Host ""
Write-Host "Login credentials:" -ForegroundColor Magenta
Write-Host "  Username: admin" -ForegroundColor White
Write-Host "  Password: admin123" -ForegroundColor White