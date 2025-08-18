#Requires -Version 5.0

<#
.SYNOPSIS
    SecureGuard Agent Installer Build Script
    
.DESCRIPTION
    Automates the creation of professional Windows installers for SecureGuard Agent.
    Supports multiple installer formats: MSI (WiX), EXE (NSIS), and PowerShell.
    
.PARAMETER BuildType
    Type of installer to build: WiX, NSIS, PowerShell, or All
    
.PARAMETER Configuration
    Build configuration: Release or Debug
    
.PARAMETER OutputPath
    Output directory for built installers
    
.PARAMETER Version
    Version number for the installer
    
.EXAMPLE
    .\build-installer.ps1 -BuildType All -Configuration Release
    
.NOTES
    Requirements:
    - WiX Toolset 3.11+ (for MSI builds)
    - NSIS 3.0+ (for EXE builds)
    - PowerShell 5.0+ (for PowerShell builds)
#>

[CmdletBinding()]
param(
    [Parameter()]
    [ValidateSet("WiX", "NSIS", "PowerShell", "All")]
    [string]$BuildType = "All",
    
    [Parameter()]
    [ValidateSet("Release", "Debug")]
    [string]$Configuration = "Release",
    
    [Parameter()]
    [string]$OutputPath = ".\dist",
    
    [Parameter()]
    [string]$Version = "1.0.0"
)

# Configuration
$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path $PSScriptRoot -Parent
$InstallerPath = Join-Path $ProjectRoot "installer"
$AgentPath = Join-Path $ProjectRoot "agent-deployment"
$DistPath = Join-Path $ProjectRoot $OutputPath

function Write-BuildLog {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $color = switch ($Level) {
        "ERROR" { "Red" }
        "WARN" { "Yellow" }
        "SUCCESS" { "Green" }
        default { "Cyan" }
    }
    Write-Host "[$timestamp] [$Level] $Message" -ForegroundColor $color
}

function Test-BuildPrerequisites {
    Write-BuildLog "Checking build prerequisites..."
    
    # Check if agent executable exists
    $agentExe = Join-Path $AgentPath "secureguard-agent.exe"
    if (-not (Test-Path $agentExe)) {
        throw "Agent executable not found: $agentExe. Please compile the agent first."
    }
    
    # Check tools based on build type
    if ($BuildType -eq "WiX" -or $BuildType -eq "All") {
        if (-not (Get-Command "candle.exe" -ErrorAction SilentlyContinue)) {
            Write-BuildLog "WiX Toolset not found. Skipping MSI build." "WARN"
            if ($BuildType -eq "WiX") {
                throw "WiX Toolset is required for MSI builds. Install from: https://wixtoolset.org/"
            }
        }
    }
    
    if ($BuildType -eq "NSIS" -or $BuildType -eq "All") {
        if (-not (Get-Command "makensis.exe" -ErrorAction SilentlyContinue)) {
            Write-BuildLog "NSIS not found. Skipping NSIS build." "WARN"
            if ($BuildType -eq "NSIS") {
                throw "NSIS is required for EXE builds. Install from: https://nsis.sourceforge.io/"
            }
        }
    }
    
    Write-BuildLog "Prerequisites check completed" "SUCCESS"
}

function Initialize-BuildEnvironment {
    Write-BuildLog "Initializing build environment..."
    
    # Create output directory
    if (Test-Path $DistPath) {
        Remove-Item $DistPath -Recurse -Force
    }
    New-Item -ItemType Directory -Path $DistPath -Force | Out-Null
    
    # Create temporary build directory
    $script:TempBuildPath = Join-Path $env:TEMP "SecureGuardBuild_$(Get-Date -Format 'yyyyMMdd_HHmmss')"
    New-Item -ItemType Directory -Path $script:TempBuildPath -Force | Out-Null
    
    Write-BuildLog "Build environment initialized"
    Write-BuildLog "  Output: $DistPath"
    Write-BuildLog "  Temp: $script:TempBuildPath"
}

function Copy-BuildAssets {
    Write-BuildLog "Copying build assets..."
    
    # Copy agent executable
    $agentSource = Join-Path $AgentPath "secureguard-agent.exe"
    $agentDest = Join-Path $script:TempBuildPath "secureguard-agent.exe"
    Copy-Item $agentSource $agentDest -Force
    
    # Copy installer scripts
    Copy-Item "$InstallerPath\*" $script:TempBuildPath -Recurse -Force
    
    Write-BuildLog "Build assets copied successfully"
}

function Build-WiXInstaller {
    if (-not (Get-Command "candle.exe" -ErrorAction SilentlyContinue)) {
        Write-BuildLog "WiX Toolset not available, skipping MSI build" "WARN"
        return
    }
    
    Write-BuildLog "Building WiX MSI installer..."
    
    try {
        $wixSource = Join-Path $script:TempBuildPath "SecureGuardAgent.wxs"
        $wixObj = Join-Path $script:TempBuildPath "SecureGuardAgent.wixobj"
        $msiOutput = Join-Path $DistPath "SecureGuardAgent-$Version.msi"
        
        # Update version in WiX source
        $wixContent = Get-Content $wixSource -Raw
        $wixContent = $wixContent -replace 'Version="1\.0\.0"', "Version=`"$Version`""
        $wixContent = $wixContent -replace '\$\(var\.SourcePath\)', $script:TempBuildPath
        Set-Content $wixSource $wixContent -Encoding UTF8
        
        # Compile
        & candle.exe -arch x64 -out $wixObj $wixSource
        if ($LASTEXITCODE -ne 0) { throw "WiX compilation failed" }
        
        # Link
        & light.exe -out $msiOutput $wixObj -ext WixUIExtension -ext WixFirewallExtension
        if ($LASTEXITCODE -ne 0) { throw "WiX linking failed" }
        
        Write-BuildLog "WiX MSI installer created: $(Split-Path $msiOutput -Leaf)" "SUCCESS"
        
        # Create MSI info file
        $infoContent = @"
SecureGuard Agent MSI Installer
Version: $Version
Type: Windows Installer Package (.msi)
Requirements: Windows 10/11, Windows Server 2019/2022
Installation: Right-click and 'Run as administrator' or use msiexec

Command line installation:
msiexec /i SecureGuardAgent-$Version.msi /quiet

Command line uninstallation:
msiexec /x SecureGuardAgent-$Version.msi /quiet

Professional features:
- Windows Installer compliance
- Group Policy deployment ready
- Corporate environment friendly
- Automatic rollback on failure
- Advanced logging and diagnostics
"@
        Set-Content (Join-Path $DistPath "SecureGuardAgent-$Version.msi.txt") $infoContent
    }
    catch {
        Write-BuildLog "WiX build failed: $($_.Exception.Message)" "ERROR"
    }
}

function Build-NSISInstaller {
    if (-not (Get-Command "makensis.exe" -ErrorAction SilentlyContinue)) {
        Write-BuildLog "NSIS not available, skipping EXE build" "WARN"
        return
    }
    
    Write-BuildLog "Building NSIS EXE installer..."
    
    try {
        $nsisScript = Join-Path $script:TempBuildPath "SecureGuardAgent.nsi"
        $exeOutput = Join-Path $DistPath "SecureGuardAgentInstaller-$Version.exe"
        
        # Update version in NSIS script
        $nsisContent = Get-Content $nsisScript -Raw
        $nsisContent = $nsisContent -replace '!define PRODUCT_VERSION "1\.0\.0"', "!define PRODUCT_VERSION `"$Version`""
        Set-Content $nsisScript $nsisContent -Encoding UTF8
        
        # Build with NSIS
        & makensis.exe "/DVERSION=$Version" "/XOutFile $exeOutput" $nsisScript
        if ($LASTEXITCODE -ne 0) { throw "NSIS compilation failed" }
        
        Write-BuildLog "NSIS EXE installer created: $(Split-Path $exeOutput -Leaf)" "SUCCESS"
        
        # Create EXE info file
        $infoContent = @"
SecureGuard Agent EXE Installer
Version: $Version
Type: NSIS Executable Installer (.exe)
Requirements: Windows 10/11, Windows Server 2019/2022
Installation: Right-click and 'Run as administrator'

Features:
- Interactive installation wizard
- Custom configuration during setup
- Professional UI with branding
- Automatic service installation
- Desktop and Start Menu shortcuts
- Complete uninstaller included

Silent installation:
SecureGuardAgentInstaller-$Version.exe /S

The installer includes:
- Service installation and configuration
- Firewall rule setup
- Registry entries
- Start Menu shortcuts
- Management tools
- Complete uninstaller
"@
        Set-Content (Join-Path $DistPath "SecureGuardAgentInstaller-$Version.exe.txt") $infoContent
    }
    catch {
        Write-BuildLog "NSIS build failed: $($_.Exception.Message)" "ERROR"
    }
}

function Build-PowerShellInstaller {
    Write-BuildLog "Building PowerShell installer package..."
    
    try {
        $psScript = Join-Path $script:TempBuildPath "Install-SecureGuardAgent.ps1"
        $psOutput = Join-Path $DistPath "Install-SecureGuardAgent-$Version.ps1"
        
        # Update version in PowerShell script
        $psContent = Get-Content $psScript -Raw
        $psContent = $psContent -replace '\$Version = "1\.0\.0"', "`$Version = `"$Version`""
        
        # Embed the agent executable as Base64 for self-contained deployment
        $agentBytes = [System.IO.File]::ReadAllBytes((Join-Path $script:TempBuildPath "secureguard-agent.exe"))
        $agentB64 = [System.Convert]::ToBase64String($agentBytes)
        $psContent = $psContent -replace '\$AgentExecutableB64 = @"[^"]*"@', "`$AgentExecutableB64 = @`"`n$agentB64`n`"@"
        
        Set-Content $psOutput $psContent -Encoding UTF8
        
        Write-BuildLog "PowerShell installer created: $(Split-Path $psOutput -Leaf)" "SUCCESS"
        
        # Create PowerShell installer batch wrapper
        $batchWrapper = @"
@echo off
echo SecureGuard Agent PowerShell Installer
echo ====================================
echo.
echo This installer requires PowerShell and Administrator privileges.
echo.
pause
echo.
echo Starting installation...
powershell -ExecutionPolicy Bypass -File "%~dp0Install-SecureGuardAgent-$Version.ps1"
pause
"@
        Set-Content (Join-Path $DistPath "Install-SecureGuardAgent-$Version.bat") $batchWrapper
        
        # Create PowerShell info file
        $infoContent = @"
SecureGuard Agent PowerShell Installer
Version: $Version
Type: PowerShell Script (.ps1)
Requirements: Windows 10/11, PowerShell 5.0+, Administrator privileges
Installation: Run as Administrator

Usage:
.\Install-SecureGuardAgent-$Version.ps1 -StartService -CreateShortcuts

Parameters:
-ServerURL       : WebSocket URL (default: ws://localhost:8080/ws)
-APIBaseURL      : API base URL (default: http://localhost:8080/api/v1)  
-AgentName       : Agent identifier (default: computer name)
-InstallPath     : Installation directory
-StartService    : Start service after installation
-CreateShortcuts : Create desktop/start menu shortcuts
-Silent          : Silent installation

Features:
- Self-contained (no external dependencies)
- Embedded agent executable
- Full service management
- Configuration backup/restore
- Comprehensive logging
- Professional uninstaller included

Example:
.\Install-SecureGuardAgent-$Version.ps1 -ServerURL "ws://company.com:8080/ws" -StartService
"@
        Set-Content (Join-Path $DistPath "Install-SecureGuardAgent-$Version.ps1.txt") $infoContent
    }
    catch {
        Write-BuildLog "PowerShell build failed: $($_.Exception.Message)" "ERROR"
    }
}

function Build-AllPackages {
    Write-BuildLog "Building all installer packages..."
    
    switch ($BuildType) {
        "WiX" { Build-WiXInstaller }
        "NSIS" { Build-NSISInstaller }
        "PowerShell" { Build-PowerShellInstaller }
        "All" {
            Build-WiXInstaller
            Build-NSISInstaller
            Build-PowerShellInstaller
        }
    }
}

function New-DeploymentPackage {
    Write-BuildLog "Creating deployment package..."
    
    # Create comprehensive README
    $readmeContent = @"
SecureGuard Agent Deployment Package
Version: $Version
Build Date: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')

INSTALLER OPTIONS:
================

1. MSI Installer (SecureGuardAgent-$Version.msi)
   - Professional Windows Installer package
   - Best for enterprise/corporate environments
   - Supports Group Policy deployment
   - Command line: msiexec /i SecureGuardAgent-$Version.msi

2. EXE Installer (SecureGuardAgentInstaller-$Version.exe)
   - Interactive installation wizard
   - User-friendly interface
   - Custom configuration during setup
   - Recommended for individual installations

3. PowerShell Installer (Install-SecureGuardAgent-$Version.ps1)
   - Self-contained script
   - Flexible configuration options
   - No additional tools required
   - Perfect for automated deployments

SYSTEM REQUIREMENTS:
==================
- Windows 10/11 or Windows Server 2019/2022
- Administrator privileges required
- 512 MB RAM (1 GB recommended)
- 100 MB disk space (500 MB recommended)
- Internet connection for server communication

QUICK START:
===========
1. Choose appropriate installer for your environment
2. Right-click installer and 'Run as administrator'
3. Follow installation prompts
4. Configure server connection settings
5. Service starts automatically

POST-INSTALLATION:
=================
- Service Name: SecureGuardAgent
- Install Location: C:\Program Files\SecureGuard\Agent
- Configuration: config\config.toml
- Logs: logs\agent.log
- Management: manage.bat or manage.ps1

SUPPORT:
========
Documentation: See .txt files for detailed installer information
Website: https://secureguard.com
Support: https://secureguard.com/support

BUILD INFORMATION:
================
Build Type: $BuildType
Configuration: $Configuration
Build Date: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')
Agent Version: $Version
"@

    Set-Content (Join-Path $DistPath "README.txt") $readmeContent
    
    # Create deployment verification script
    $verifyScript = @"
# SecureGuard Agent Deployment Verification
# Checks if deployment package is complete and valid

Write-Host "SecureGuard Agent Deployment Package Verification" -ForegroundColor Cyan
Write-Host "Version: $Version" -ForegroundColor Green
Write-Host

`$packagePath = Split-Path `$MyInvocation.MyCommand.Path
`$allGood = `$true

# Check for installers
`$installers = @(
    @{Name="MSI Installer"; File="SecureGuardAgent-$Version.msi"; Required=`$false},
    @{Name="EXE Installer"; File="SecureGuardAgentInstaller-$Version.exe"; Required=`$false},
    @{Name="PowerShell Installer"; File="Install-SecureGuardAgent-$Version.ps1"; Required=`$true}
)

foreach (`$installer in `$installers) {
    `$path = Join-Path `$packagePath `$installer.File
    if (Test-Path `$path) {
        `$size = (Get-Item `$path).Length
        Write-Host "✓ `$(`$installer.Name): Found (`$([math]::Round(`$size/1MB, 2)) MB)" -ForegroundColor Green
    } else {
        `$status = if (`$installer.Required) { "ERROR" } else { "WARN" }
        `$color = if (`$installer.Required) { "Red" } else { "Yellow" }
        Write-Host "✗ `$(`$installer.Name): Missing" -ForegroundColor `$color
        if (`$installer.Required) { `$allGood = `$false }
    }
}

# Check documentation
`$docs = Get-ChildItem `$packagePath -Filter "*.txt" | Measure-Object
Write-Host "📄 Documentation files: `$(`$docs.Count)" -ForegroundColor Cyan

if (`$allGood) {
    Write-Host "`nDeployment package is ready for distribution! ✓" -ForegroundColor Green
} else {
    Write-Host "`nDeployment package has issues that need attention! ✗" -ForegroundColor Red
}

Read-Host "`nPress Enter to exit"
"@

    Set-Content (Join-Path $DistPath "verify-package.ps1") $verifyScript
}

function Remove-TempFiles {
    Write-BuildLog "Cleaning up temporary files..."
    
    if (Test-Path $script:TempBuildPath) {
        Remove-Item $script:TempBuildPath -Recurse -Force -ErrorAction SilentlyContinue
    }
}

function Show-BuildSummary {
    Write-BuildLog "`n==========================================" "SUCCESS"
    Write-BuildLog "Build Complete!" "SUCCESS"  
    Write-BuildLog "==========================================" "SUCCESS"
    
    Write-BuildLog "`nBuild Summary:"
    Write-BuildLog "  Version: $Version"
    Write-BuildLog "  Configuration: $Configuration"
    Write-BuildLog "  Build Type: $BuildType"
    Write-BuildLog "  Output: $DistPath"
    
    $files = Get-ChildItem $DistPath | Where-Object { $_.Extension -in @('.msi', '.exe', '.ps1') -and $_.Name -notlike '*verify*' }
    Write-BuildLog "`nGenerated Installers:"
    foreach ($file in $files) {
        $size = [math]::Round($file.Length / 1MB, 2)
        Write-BuildLog "  📦 $($file.Name) ($size MB)"
    }
    
    Write-BuildLog "`nNext Steps:"
    Write-BuildLog "  1. Test installers on target systems"
    Write-BuildLog "  2. Distribute appropriate installer for each environment"
    Write-BuildLog "  3. Monitor installation logs for issues"
    Write-BuildLog "`nRun verify-package.ps1 to validate the deployment package"
}

# Main Build Process
try {
    Write-BuildLog "SecureGuard Agent Installer Build" "SUCCESS"
    Write-BuildLog "Version: $Version | Type: $BuildType | Config: $Configuration"
    
    Test-BuildPrerequisites
    Initialize-BuildEnvironment
    Copy-BuildAssets
    Build-AllPackages
    New-DeploymentPackage
    Remove-TempFiles
    
    Show-BuildSummary
}
catch {
    Write-BuildLog "`nBuild failed: $($_.Exception.Message)" "ERROR"
    Remove-TempFiles
    exit 1
}
finally {
    Remove-TempFiles
}