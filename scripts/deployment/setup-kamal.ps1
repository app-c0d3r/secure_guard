# SecureGuard Kamal Setup Script
# PowerShell script to install and configure Kamal for deployment

param(
    [switch]$InstallRuby,
    [switch]$InstallDocker,
    [switch]$ConfigureServer,
    [string]$ServerIP,
    [string]$RegistryUsername
)

Write-Host "🚀 SecureGuard Kamal Setup Script" -ForegroundColor Green
Write-Host "=================================" -ForegroundColor Green

# Function to check if command exists
function Test-Command($cmdname) {
    return [bool](Get-Command -Name $cmdname -ErrorAction SilentlyContinue)
}

# Install Ruby if requested
if ($InstallRuby) {
    Write-Host "📦 Installing Ruby..." -ForegroundColor Yellow
    
    if (Test-Command "winget") {
        winget install RubyInstallerTeam.Ruby
    } else {
        Write-Host "❌ Winget not found. Please install Ruby manually from https://rubyinstaller.org/" -ForegroundColor Red
        Write-Host "   Download Ruby 3.2+ and install with DevKit" -ForegroundColor Gray
    }
}

# Check Ruby installation
if (Test-Command "ruby") {
    $rubyVersion = ruby --version
    Write-Host "✅ Ruby found: $rubyVersion" -ForegroundColor Green
} else {
    Write-Host "❌ Ruby not found. Install Ruby first with -InstallRuby flag" -ForegroundColor Red
    exit 1
}

# Install Docker if requested
if ($InstallDocker) {
    Write-Host "🐳 Installing Docker Desktop..." -ForegroundColor Yellow
    
    if (Test-Command "winget") {
        winget install Docker.DockerDesktop
        Write-Host "⚠️  Please restart your computer after Docker installation" -ForegroundColor Yellow
    } else {
        Write-Host "❌ Winget not found. Please install Docker manually from https://docker.com/get-started" -ForegroundColor Red
    }
}

# Check Docker installation
if (Test-Command "docker") {
    $dockerVersion = docker --version
    Write-Host "✅ Docker found: $dockerVersion" -ForegroundColor Green
} else {
    Write-Host "❌ Docker not found. Install Docker first with -InstallDocker flag" -ForegroundColor Red
}

# Install Kamal
Write-Host "💎 Installing Kamal..." -ForegroundColor Yellow
try {
    gem install kamal
    $kamalVersion = kamal version
    Write-Host "✅ Kamal installed: $kamalVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Failed to install Kamal. Check Ruby installation." -ForegroundColor Red
    exit 1
}

# Create Kamal configuration directory
$configDir = "config"
if (!(Test-Path $configDir)) {
    New-Item -ItemType Directory -Path $configDir
    Write-Host "📁 Created config directory" -ForegroundColor Green
}

# Create secrets directory
$secretsDir = ".kamal"
if (!(Test-Path $secretsDir)) {
    New-Item -ItemType Directory -Path $secretsDir
    Write-Host "🔒 Created .kamal directory" -ForegroundColor Green
}

# Generate sample secrets file
$secretsFile = ".kamal/secrets"
if (!(Test-Path $secretsFile)) {
    $secretsContent = @"
# SecureGuard Production Secrets
# IMPORTANT: Never commit this file to version control

# Database connection (update with your production database)
DATABASE_URL=postgresql://username:password@localhost:5432/secureguard_prod

# JWT secret - generate a secure random string
JWT_SECRET=$(New-Guid)

# Redis connection (if using Redis)
REDIS_URL=redis://localhost:6379

# Container registry password (GitHub token or Docker Hub password)
KAMAL_REGISTRY_PASSWORD=your-registry-password-here
"@
    
    Set-Content -Path $secretsFile -Value $secretsContent
    Write-Host "🔐 Created sample secrets file at .kamal/secrets" -ForegroundColor Green
    Write-Host "⚠️  Please update the secrets with your production values" -ForegroundColor Yellow
}

# Configure server if requested
if ($ConfigureServer -and $ServerIP) {
    Write-Host "🖥️  Configuring server: $ServerIP" -ForegroundColor Yellow
    
    # Update deploy.yml with server IP
    $deployFile = "config/deploy.yml"
    if (Test-Path $deployFile) {
        $content = Get-Content $deployFile -Raw
        $content = $content -replace "YOUR_SERVER_IP", $ServerIP
        
        if ($RegistryUsername) {
            $content = $content -replace "YOUR_GITHUB_USERNAME", $RegistryUsername
        }
        
        Set-Content -Path $deployFile -Value $content
        Write-Host "✅ Updated deploy.yml with server IP" -ForegroundColor Green
    }
}

# Summary
Write-Host ""
Write-Host "🎉 Kamal Setup Complete!" -ForegroundColor Green
Write-Host "========================" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "1. Update .kamal/secrets with your production values" -ForegroundColor Gray
Write-Host "2. Update config/deploy.yml with your server details" -ForegroundColor Gray
Write-Host "3. Set up your server with Docker: kamal setup" -ForegroundColor Gray
Write-Host "4. Deploy your application: kamal deploy" -ForegroundColor Gray
Write-Host ""
Write-Host "Local development:" -ForegroundColor Yellow
Write-Host "- Run: .\scripts\myservice.bat start prod" -ForegroundColor Gray
Write-Host "- Access: http://localhost:3002" -ForegroundColor Gray
Write-Host "- Login: admin@company.com / SecurePass123!" -ForegroundColor Gray
Write-Host ""
Write-Host "Documentation: docs/Kamal_Deployment_Guide.md" -ForegroundColor Cyan

# Check if git is configured for secrets
if (Test-Path ".gitignore") {
    $gitignore = Get-Content ".gitignore" -Raw
    if ($gitignore -notlike "*/.kamal/secrets*") {
        Add-Content ".gitignore" "`n# Kamal secrets`n.kamal/secrets"
        Write-Host "✅ Added .kamal/secrets to .gitignore" -ForegroundColor Green
    }
} else {
    Write-Host "⚠️  No .gitignore found. Make sure to never commit .kamal/secrets" -ForegroundColor Yellow
}