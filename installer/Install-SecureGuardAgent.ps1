#Requires -Version 5.0
#Requires -RunAsAdministrator

<#
.SYNOPSIS
    SecureGuard Agent Professional Installer
    
.DESCRIPTION
    Professional PowerShell installer for SecureGuard Agent.
    Creates a complete Windows service installation with configuration,
    logging, firewall rules, and management tools.
    
.PARAMETER APIKey
    SecureGuard API key for agent registration (required)
    
.PARAMETER DeviceName
    User-friendly name for this device (defaults to computer name)
    
.PARAMETER ServerURL
    SecureGuard server base URL
    
.PARAMETER InstallPath
    Installation directory (defaults to Program Files)
    
.PARAMETER StartService
    Start the service immediately after installation
    
.PARAMETER CreateShortcuts
    Create desktop and start menu shortcuts
    
.EXAMPLE
    .\Install-SecureGuardAgent.ps1 -APIKey "sg_abcd1234_xyz789" -DeviceName "WorkStation-01" -StartService
    
.NOTES
    Version: 1.0.0
    Author: SecureGuard Technologies
    Requires: Windows 10/11 or Windows Server 2019/2022
    Requires: PowerShell 5.0 or higher
    Requires: Administrator privileges
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)]
    [ValidatePattern('^sg_[a-f0-9]{8}_[a-f0-9]{20}$')]
    [string]$APIKey,
    
    [Parameter()]
    [string]$DeviceName = $env:COMPUTERNAME,
    
    [Parameter()]
    [string]$ServerURL = "https://api.secureguard.com",
    
    [Parameter()]
    [string]$InstallPath = "$env:ProgramFiles\SecureGuard\Agent",
    
    [Parameter()]
    [switch]$StartService,
    
    [Parameter()]
    [switch]$CreateShortcuts = $true,
    
    [Parameter()]
    [switch]$Silent
)

# Script configuration
$ErrorActionPreference = "Stop"
$ServiceName = "SecureGuardAgent"
$ServiceDisplayName = "SecureGuard Security Agent"
$ProductName = "SecureGuard Agent"
$Version = "1.0.0"

# Embedded agent executable as Base64 (placeholder - will be replaced with actual binary)
$AgentExecutableB64 = @"
# This would contain the Base64-encoded agent executable
# For demo purposes, this is a placeholder
# In production, you would embed the actual binary here
"@

function Write-LogMessage {
    param([string]$Message, [string]$Level = "INFO")
    if (-not $Silent) {
        $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        Write-Host "[$timestamp] [$Level] $Message" -ForegroundColor $(
            switch ($Level) {
                "ERROR" { "Red" }
                "WARN" { "Yellow" }
                "SUCCESS" { "Green" }
                default { "White" }
            }
        )
    }
}

function Test-Prerequisites {
    Write-LogMessage "Checking system prerequisites..."
    
    # Check Windows version
    $os = Get-WmiObject Win32_OperatingSystem
    $version = [System.Version]$os.Version
    if ($version.Major -lt 10) {
        throw "Windows 10 or higher is required. Current version: $($os.Caption)"
    }
    
    # Check PowerShell version
    if ($PSVersionTable.PSVersion.Major -lt 5) {
        throw "PowerShell 5.0 or higher is required. Current version: $($PSVersionTable.PSVersion)"
    }
    
    # Check if running as administrator
    $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        throw "This installer must be run as Administrator"
    }
    
    Write-LogMessage "Prerequisites check passed" "SUCCESS"
}

function Stop-ExistingService {
    if (Get-Service -Name $ServiceName -ErrorAction SilentlyContinue) {
        Write-LogMessage "Stopping existing SecureGuard Agent service..."
        Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
        
        # Wait for service to stop
        $timeout = 30
        $elapsed = 0
        while ((Get-Service -Name $ServiceName).Status -ne "Stopped" -and $elapsed -lt $timeout) {
            Start-Sleep -Seconds 1
            $elapsed++
        }
        
        if ((Get-Service -Name $ServiceName).Status -ne "Stopped") {
            Write-LogMessage "Warning: Service did not stop within timeout period" "WARN"
        }
    }
}

function Remove-ExistingInstallation {
    Write-LogMessage "Checking for existing installation..."
    
    # Stop service if running
    Stop-ExistingService
    
    # Remove existing service
    if (Get-Service -Name $ServiceName -ErrorAction SilentlyContinue) {
        Write-LogMessage "Removing existing service..."
        sc.exe delete $ServiceName | Out-Null
        Start-Sleep -Seconds 2
    }
    
    # Remove existing files (but preserve logs and config)
    if (Test-Path $InstallPath) {
        Write-LogMessage "Backing up existing configuration..."
        
        $backupPath = "$env:TEMP\SecureGuardBackup_$(Get-Date -Format 'yyyyMMdd_HHmmss')"
        New-Item -ItemType Directory -Path $backupPath -Force | Out-Null
        
        if (Test-Path "$InstallPath\config\config.toml") {
            Copy-Item "$InstallPath\config\config.toml" "$backupPath\config.toml" -ErrorAction SilentlyContinue
        }
        
        # Remove old executable but keep data
        Remove-Item "$InstallPath\secureguard-agent.exe" -Force -ErrorAction SilentlyContinue
        Remove-Item "$InstallPath\manage.bat" -Force -ErrorAction SilentlyContinue
        
        Write-LogMessage "Configuration backed up to: $backupPath"
    }
}

function New-InstallationDirectories {
    Write-LogMessage "Creating installation directories..."
    
    @("$InstallPath", "$InstallPath\config", "$InstallPath\logs", "$InstallPath\temp") | ForEach-Object {
        if (-not (Test-Path $_)) {
            New-Item -ItemType Directory -Path $_ -Force | Out-Null
            Write-LogMessage "Created directory: $_"
        }
    }
}

function Install-AgentExecutable {
    Write-LogMessage "Installing agent executable..."
    
    # For this demo, copy from the agent-deployment folder
    $sourcePath = Join-Path $PSScriptRoot "..\agent-deployment\secureguard-agent.exe"
    $destPath = "$InstallPath\secureguard-agent.exe"
    
    if (Test-Path $sourcePath) {
        Copy-Item $sourcePath $destPath -Force
        Write-LogMessage "Agent executable installed successfully"
    } else {
        throw "Source agent executable not found at: $sourcePath"
    }
    
    # In a real deployment, you would decode the embedded Base64 binary:
    # $bytes = [System.Convert]::FromBase64String($AgentExecutableB64)
    # [System.IO.File]::WriteAllBytes($destPath, $bytes)
}

function New-ConfigurationFile {
    Write-LogMessage "Creating configuration file..."
    
    $configPath = "$InstallPath\config\config.toml"
    
    $configContent = @"
# SecureGuard Agent Configuration
# Auto-generated on $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')

[server]
base_url = "$ServerURL"
timeout = 30
retry_interval = 60

[agent]
device_name = "$DeviceName"
api_key = "$APIKey"
log_level = "info"
heartbeat_interval = 30
data_collection_interval = 300

[monitoring]
heartbeat_interval = 30
data_collection_interval = 300

[security]
encryption_enabled = true

[logging]
file_path = "$InstallPath\\logs\\agent.log"
max_size = "10MB"
max_files = 5
console_output = false

[performance]
cpu_limit = 25
memory_limit = 512
bandwidth_limit = 1024

[monitoring]
monitor_file_changes = true
monitor_network_connections = true
monitor_process_creation = true
monitor_registry_changes = true

excluded_paths = [
    "C:\\Windows\\Temp",
    "C:\\Users\\*\\AppData\\Local\\Temp",
    "$InstallPath\\logs"
]
"@

    Set-Content -Path $configPath -Value $configContent -Encoding UTF8
    Write-LogMessage "Configuration file created: $configPath"
}

function New-ManagementScripts {
    Write-LogMessage "Creating management scripts..."
    
    # PowerShell management script
    $managePsScript = @"
# SecureGuard Agent Management Script
param([string]`$Action)

`$ServiceName = "$ServiceName"

switch (`$Action.ToLower()) {
    "start" { 
        Start-Service `$ServiceName
        Write-Host "Service started" -ForegroundColor Green
    }
    "stop" { 
        Stop-Service `$ServiceName -Force
        Write-Host "Service stopped" -ForegroundColor Yellow
    }
    "restart" { 
        Restart-Service `$ServiceName -Force
        Write-Host "Service restarted" -ForegroundColor Green
    }
    "status" { 
        Get-Service `$ServiceName | Format-Table -AutoSize
    }
    "logs" { 
        Get-Content "$InstallPath\logs\agent.log" -Tail 50
    }
    "config" { 
        notepad "$InstallPath\config\config.toml"
    }
    default {
        Write-Host "SecureGuard Agent Management" -ForegroundColor Cyan
        Write-Host "Usage: .\manage.ps1 [action]"
        Write-Host "Actions: start, stop, restart, status, logs, config"
    }
}
"@

    Set-Content -Path "$InstallPath\manage.ps1" -Value $managePsScript -Encoding UTF8
    
    # Batch wrapper for convenience
    $manageBatScript = @"
@echo off
powershell -ExecutionPolicy Bypass -File "%~dp0manage.ps1" %*
pause
"@

    Set-Content -Path "$InstallPath\manage.bat" -Value $manageBatScript -Encoding ASCII
    Write-LogMessage "Management scripts created"
}

function Install-WindowsService {
    Write-LogMessage "Installing Windows service..."
    
    # Create service
    $result = sc.exe create $ServiceName binPath= "`"$InstallPath\secureguard-agent.exe`"" start= auto DisplayName= $ServiceDisplayName depend= Tcpip
    
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to create Windows service: $result"
    }
    
    # Set service description
    sc.exe description $ServiceName "SecureGuard security monitoring agent - monitors system for threats and security events" | Out-Null
    
    # Configure service recovery options
    sc.exe failure $ServiceName reset= 86400 actions= restart/30000/restart/60000/restart/120000 | Out-Null
    
    Write-LogMessage "Windows service installed successfully" "SUCCESS"
}

function Add-FirewallRules {
    Write-LogMessage "Configuring Windows Firewall rules..."
    
    try {
        # Remove existing rules
        Remove-NetFirewallRule -DisplayName "SecureGuard Agent*" -ErrorAction SilentlyContinue
        
        # Add new rules
        New-NetFirewallRule -DisplayName "SecureGuard Agent Outbound" -Direction Outbound -Program "$InstallPath\secureguard-agent.exe" -Action Allow | Out-Null
        New-NetFirewallRule -DisplayName "SecureGuard Agent Inbound" -Direction Inbound -Program "$InstallPath\secureguard-agent.exe" -Action Allow | Out-Null
        
        Write-LogMessage "Firewall rules configured successfully" "SUCCESS"
    }
    catch {
        Write-LogMessage "Warning: Could not configure firewall rules: $($_.Exception.Message)" "WARN"
    }
}

function New-RegistryEntries {
    Write-LogMessage "Creating registry entries..."
    
    $regPath = "HKLM:\SOFTWARE\SecureGuard\Agent"
    
    if (-not (Test-Path $regPath)) {
        New-Item -Path $regPath -Force | Out-Null
    }
    
    Set-ItemProperty -Path $regPath -Name "InstallPath" -Value $InstallPath
    Set-ItemProperty -Path $regPath -Name "Version" -Value $Version
    Set-ItemProperty -Path $regPath -Name "ServiceName" -Value $ServiceName
    Set-ItemProperty -Path $regPath -Name "InstallDate" -Value (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
    
    # Add to Programs and Features
    $uninstallPath = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\$ProductName"
    
    if (-not (Test-Path $uninstallPath)) {
        New-Item -Path $uninstallPath -Force | Out-Null
    }
    
    Set-ItemProperty -Path $uninstallPath -Name "DisplayName" -Value $ProductName
    Set-ItemProperty -Path $uninstallPath -Name "DisplayVersion" -Value $Version
    Set-ItemProperty -Path $uninstallPath -Name "Publisher" -Value "SecureGuard Technologies"
    Set-ItemProperty -Path $uninstallPath -Name "InstallLocation" -Value $InstallPath
    Set-ItemProperty -Path $uninstallPath -Name "UninstallString" -Value "powershell -ExecutionPolicy Bypass -File `"$InstallPath\Uninstall-SecureGuardAgent.ps1`""
    Set-ItemProperty -Path $uninstallPath -Name "NoModify" -Value 1
    Set-ItemProperty -Path $uninstallPath -Name "NoRepair" -Value 1
    
    Write-LogMessage "Registry entries created successfully" "SUCCESS"
}

function New-Shortcuts {
    if ($CreateShortcuts) {
        Write-LogMessage "Creating shortcuts..."
        
        $WshShell = New-Object -ComObject WScript.Shell
        
        # Desktop shortcut
        $desktopShortcut = $WshShell.CreateShortcut("$env:PUBLIC\Desktop\SecureGuard Agent Manager.lnk")
        $desktopShortcut.TargetPath = "$InstallPath\manage.bat"
        $desktopShortcut.WorkingDirectory = $InstallPath
        $desktopShortcut.Description = "SecureGuard Agent Management"
        $desktopShortcut.Save()
        
        # Start Menu folder
        $startMenuPath = "$env:ProgramData\Microsoft\Windows\Start Menu\Programs\SecureGuard Agent"
        if (-not (Test-Path $startMenuPath)) {
            New-Item -ItemType Directory -Path $startMenuPath -Force | Out-Null
        }
        
        # Start Menu shortcuts
        @(
            @{Name = "Agent Manager"; Target = "$InstallPath\manage.bat"; Description = "Manage SecureGuard Agent service"},
            @{Name = "Configuration"; Target = "notepad.exe"; Args = "$InstallPath\config\config.toml"; Description = "Edit agent configuration"},
            @{Name = "View Logs"; Target = "$InstallPath\logs"; Description = "View agent log files"}
        ) | ForEach-Object {
            $shortcut = $WshShell.CreateShortcut("$startMenuPath\$($_.Name).lnk")
            $shortcut.TargetPath = $_.Target
            if ($_.Args) { $shortcut.Arguments = $_.Args }
            $shortcut.WorkingDirectory = $InstallPath
            $shortcut.Description = $_.Description
            $shortcut.Save()
        }
        
        Write-LogMessage "Shortcuts created successfully" "SUCCESS"
    }
}

function New-UninstallScript {
    Write-LogMessage "Creating uninstall script..."
    
    $uninstallScript = @"
#Requires -Version 5.0
#Requires -RunAsAdministrator

# SecureGuard Agent Uninstaller
param([switch]`$Silent)

`$ServiceName = "$ServiceName"
`$InstallPath = "$InstallPath"

function Write-LogMessage {
    param([string]`$Message, [string]`$Level = "INFO")
    if (-not `$Silent) {
        Write-Host "`$Message" -ForegroundColor `$(switch (`$Level) { "ERROR" { "Red" } "WARN" { "Yellow" } "SUCCESS" { "Green" } default { "White" } })
    }
}

Write-LogMessage "Uninstalling SecureGuard Agent..."

# Stop and remove service
if (Get-Service -Name `$ServiceName -ErrorAction SilentlyContinue) {
    Write-LogMessage "Stopping service..."
    Stop-Service -Name `$ServiceName -Force -ErrorAction SilentlyContinue
    sc.exe delete `$ServiceName | Out-Null
}

# Remove firewall rules
Write-LogMessage "Removing firewall rules..."
Remove-NetFirewallRule -DisplayName "SecureGuard Agent*" -ErrorAction SilentlyContinue

# Remove installation directory
Write-LogMessage "Removing installation files..."
Remove-Item `$InstallPath -Recurse -Force -ErrorAction SilentlyContinue

# Remove registry entries
Write-LogMessage "Cleaning registry..."
Remove-Item "HKLM:\SOFTWARE\SecureGuard" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\$ProductName" -Force -ErrorAction SilentlyContinue

# Remove shortcuts
Write-LogMessage "Removing shortcuts..."
Remove-Item "`$env:PUBLIC\Desktop\SecureGuard Agent Manager.lnk" -Force -ErrorAction SilentlyContinue
Remove-Item "`$env:ProgramData\Microsoft\Windows\Start Menu\Programs\SecureGuard Agent" -Recurse -Force -ErrorAction SilentlyContinue

Write-LogMessage "SecureGuard Agent has been uninstalled successfully!" "SUCCESS"

if (-not `$Silent) {
    Read-Host "Press Enter to exit"
}
"@

    Set-Content -Path "$InstallPath\Uninstall-SecureGuardAgent.ps1" -Value $uninstallScript -Encoding UTF8
    Write-LogMessage "Uninstall script created"
}

function Start-AgentService {
    if ($StartService) {
        Write-LogMessage "Starting SecureGuard Agent service..."
        
        try {
            Start-Service -Name $ServiceName
            
            # Wait for service to start
            $timeout = 30
            $elapsed = 0
            while ((Get-Service -Name $ServiceName).Status -ne "Running" -and $elapsed -lt $timeout) {
                Start-Sleep -Seconds 1
                $elapsed++
            }
            
            if ((Get-Service -Name $ServiceName).Status -eq "Running") {
                Write-LogMessage "Service started successfully!" "SUCCESS"
            } else {
                Write-LogMessage "Service created but failed to start. Check logs for details." "WARN"
            }
        }
        catch {
            Write-LogMessage "Failed to start service: $($_.Exception.Message)" "ERROR"
        }
    }
}

function Show-InstallationSummary {
    Write-LogMessage "`n==========================================" "SUCCESS"
    Write-LogMessage "SecureGuard Agent Installation Complete!" "SUCCESS"
    Write-LogMessage "==========================================" "SUCCESS"
    
    Write-LogMessage "`nInstallation Details:"
    Write-LogMessage "  Product: $ProductName v$Version"
    Write-LogMessage "  Location: $InstallPath"
    Write-LogMessage "  Service: $ServiceName"
    Write-LogMessage "  Configuration: $InstallPath\config\config.toml"
    Write-LogMessage "  Logs: $InstallPath\logs\"
    
    Write-LogMessage "`nManagement:"
    Write-LogMessage "  PowerShell: $InstallPath\manage.ps1"
    Write-LogMessage "  Batch File: $InstallPath\manage.bat"
    Write-LogMessage "  Windows Services: services.msc"
    
    if ($CreateShortcuts) {
        Write-LogMessage "`nShortcuts created on Desktop and Start Menu"
    }
    
    $serviceStatus = (Get-Service -Name $ServiceName).Status
    Write-LogMessage "`nService Status: $serviceStatus" $(if ($serviceStatus -eq "Running") { "SUCCESS" } else { "WARN" })
    
    Write-LogMessage "`nFor support, visit: https://secureguard.com/support"
}

# Main Installation Process
try {
    Write-LogMessage "Starting SecureGuard Agent installation..." "SUCCESS"
    Write-LogMessage "Version: $Version"
    Write-LogMessage "Target: $InstallPath"
    
    Test-Prerequisites
    Remove-ExistingInstallation
    New-InstallationDirectories
    Install-AgentExecutable
    New-ConfigurationFile
    New-ManagementScripts
    Install-WindowsService
    Add-FirewallRules
    New-RegistryEntries
    New-Shortcuts
    New-UninstallScript
    Start-AgentService
    
    Show-InstallationSummary
    
    if (-not $Silent) {
        Write-Host "`nInstallation completed successfully!" -ForegroundColor Green
        Read-Host "Press Enter to exit"
    }
}
catch {
    Write-LogMessage "`nInstallation failed: $($_.Exception.Message)" "ERROR"
    
    if (-not $Silent) {
        Write-Host "`nInstallation failed. Check the error above for details." -ForegroundColor Red
        Read-Host "Press Enter to exit"
    }
    exit 1
}