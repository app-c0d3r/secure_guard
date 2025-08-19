# SecureGuard Agent Deployment Guide

## Overview

SecureGuard Agent is a professional endpoint security monitoring solution that runs as a Windows service. This guide covers deployment options, installation methods, and management procedures for enterprise environments.

## 🚀 Quick Start

### For End Users
1. Download `SecureGuardAgentInstaller.exe` 
2. Right-click → "Run as administrator"
3. Follow the installation wizard
4. Agent starts automatically

### For System Administrators
1. Choose appropriate installer for your environment
2. Test on pilot systems first
3. Deploy using preferred method (GPO, SCCM, PowerShell, etc.)
4. Configure centralized monitoring

## 📦 Installer Options

We provide **three professional installer formats** to meet different deployment needs:

### 1. MSI Installer (Enterprise Recommended)
- **File**: `SecureGuardAgent-{version}.msi`
- **Best For**: Corporate/enterprise environments
- **Features**:
  - Windows Installer compliance
  - Group Policy deployment ready
  - SCCM/MECM compatible
  - Automatic rollback on failure
  - Advanced logging and diagnostics
  
**Installation**:
```batch
# Interactive
msiexec /i SecureGuardAgent-1.0.0.msi

# Silent installation
msiexec /i SecureGuardAgent-1.0.0.msi /quiet /l*v install.log

# Silent with custom configuration
msiexec /i SecureGuardAgent-1.0.0.msi /quiet SERVER_URL="ws://company.com:8080/ws"
```

### 2. EXE Installer (User-Friendly)
- **File**: `SecureGuardAgentInstaller-{version}.exe`
- **Best For**: Individual installations, small deployments
- **Features**:
  - Interactive installation wizard
  - Custom configuration during setup
  - Professional UI with branding
  - Built-in configuration validation
  
**Installation**:
```batch
# Interactive
SecureGuardAgentInstaller-1.0.0.exe

# Silent installation
SecureGuardAgentInstaller-1.0.0.exe /S
```

### 3. PowerShell Installer (Flexible)
- **File**: `Install-SecureGuardAgent-{version}.ps1`
- **Best For**: Automated deployments, DevOps pipelines
- **Features**:
  - Self-contained script (no external dependencies)
  - Embedded agent executable
  - Flexible configuration options
  - Comprehensive logging
  - Perfect for automation

**Installation**:
```powershell
# Basic installation
.\Install-SecureGuardAgent-1.0.0.ps1 -StartService

# Custom configuration
.\Install-SecureGuardAgent-1.0.0.ps1 `
    -ServerURL "ws://secureguard.company.com:8080/ws" `
    -APIBaseURL "https://secureguard.company.com/api/v1" `
    -AgentName "DC01-SecureGuard" `
    -StartService `
    -CreateShortcuts

# Silent installation
.\Install-SecureGuardAgent-1.0.0.ps1 -Silent -StartService
```

## 🏢 Enterprise Deployment

### Group Policy Deployment (MSI)

1. **Prepare MSI Package**:
   ```batch
   # Test installation first
   msiexec /i SecureGuardAgent-1.0.0.msi /quiet /l*v test.log
   ```

2. **Create GPO**:
   - Open Group Policy Management Console
   - Create new GPO: "SecureGuard Agent Deployment"
   - Edit GPO → Computer Configuration → Software Settings
   - Right-click "Software Installation" → New → Package
   - Browse to MSI file on network share

3. **Configure Deployment**:
   - Deployment Method: "Assigned"
   - Installation UI Options: "Basic"
   - Advanced → Deploy application at logon: Yes

### SCCM/MECM Deployment

1. **Create Application**:
   ```xml
   Detection Method: Registry
   Hive: HKEY_LOCAL_MACHINE
   Key: SOFTWARE\SecureGuard\Agent
   Value: Version
   ```

2. **Installation Command**:
   ```batch
   msiexec /i SecureGuardAgent-1.0.0.msi /quiet /l*v "%TEMP%\SecureGuard_Install.log"
   ```

3. **Uninstall Command**:
   ```batch
   msiexec /x SecureGuardAgent-1.0.0.msi /quiet
   ```

### PowerShell DSC Configuration

```powershell
Configuration SecureGuardAgent {
    param (
        [string]$ServerURL = "ws://secureguard.company.com:8080/ws",
        [string]$APIBaseURL = "https://secureguard.company.com/api/v1"
    )
    
    Node "localhost" {
        Script InstallSecureGuardAgent {
            GetScript = {
                $service = Get-Service -Name "SecureGuardAgent" -ErrorAction SilentlyContinue
                @{Result = if ($service) { "Present" } else { "Absent" }}
            }
            
            TestScript = {
                $service = Get-Service -Name "SecureGuardAgent" -ErrorAction SilentlyContinue
                return ($service -ne $null -and $service.Status -eq "Running")
            }
            
            SetScript = {
                $installerPath = "\\fileserver\software\SecureGuard\Install-SecureGuardAgent-1.0.0.ps1"
                & $installerPath -ServerURL $using:ServerURL -APIBaseURL $using:APIBaseURL -StartService -Silent
            }
        }
    }
}
```

## ⚙️ Configuration Management

### Default Configuration
```toml
[server]
url = "ws://localhost:8080/ws"
api_base = "http://localhost:8080/api/v1"
timeout = 30

[agent]
name = "COMPUTER-NAME"
log_level = "info"
heartbeat_interval = 30

[security]
encryption_enabled = true

[logging]
file_path = "C:\\Program Files\\SecureGuard\\Agent\\logs\\agent.log"
max_size = "10MB"
max_files = 5
```

### Mass Configuration Update
```powershell
# Update all agents' server configuration
$computers = Get-ADComputer -Filter "Name -like '*-DESKTOP-*'"
foreach ($computer in $computers) {
    Invoke-Command -ComputerName $computer.Name -ScriptBlock {
        $configPath = "C:\Program Files\SecureGuard\Agent\config\config.toml"
        $config = Get-Content $configPath -Raw
        $config = $config -replace 'url = "ws://localhost:8080/ws"', 'url = "ws://prod.secureguard.com:8080/ws"'
        Set-Content $configPath $config
        Restart-Service SecureGuardAgent
    }
}
```

## 🔍 Post-Installation Verification

### Automated Health Check
```powershell
# Check agent health across multiple systems
function Test-SecureGuardAgent {
    param([string[]]$ComputerNames)
    
    foreach ($computer in $ComputerNames) {
        $result = Invoke-Command -ComputerName $computer -ScriptBlock {
            $service = Get-Service -Name "SecureGuardAgent" -ErrorAction SilentlyContinue
            $config = Test-Path "C:\Program Files\SecureGuard\Agent\config\config.toml"
            $logs = Test-Path "C:\Program Files\SecureGuard\Agent\logs\agent.log"
            
            [PSCustomObject]@{
                ComputerName = $env:COMPUTERNAME
                ServiceStatus = if ($service) { $service.Status } else { "Not Found" }
                ConfigExists = $config
                LogsExist = $logs
                LastLogEntry = if ($logs) { 
                    (Get-Content "C:\Program Files\SecureGuard\Agent\logs\agent.log" -Tail 1) 
                } else { "No logs" }
            }
        }
        Write-Output $result
    }
}

# Usage
$computers = @("PC001", "PC002", "PC003")
Test-SecureGuardAgent -ComputerNames $computers | Format-Table -AutoSize
```

### Service Management Commands
```batch
# Check service status
sc query SecureGuardAgent

# Start service
sc start SecureGuardAgent

# Stop service
sc stop SecureGuardAgent

# Restart service
sc stop SecureGuardAgent && timeout /t 3 && sc start SecureGuardAgent

# View service configuration
sc qc SecureGuardAgent

# Check service logs
type "C:\Program Files\SecureGuard\Agent\logs\agent.log"
```

## 🚨 Troubleshooting

### Common Installation Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| "Access Denied" | Not running as Administrator | Right-click installer → "Run as administrator" |
| Service won't start | Configuration file issues | Check config.toml syntax |
| High CPU usage | Scan interval too frequent | Increase `scan_interval` in config |
| Network connectivity errors | Firewall/proxy issues | Configure firewall exceptions |

### Log Analysis
```powershell
# Parse agent logs for errors
Get-Content "C:\Program Files\SecureGuard\Agent\logs\agent.log" | 
    Where-Object { $_ -match "ERROR|WARN" } | 
    Select-Object -Last 20
```

### Performance Monitoring
```powershell
# Monitor agent performance
Get-Process -Name "secureguard-agent" | 
    Select-Object Name, CPU, WorkingSet, Handles
```

## 🔄 Updates and Maintenance

### Manual Update Process
1. Download new installer version
2. Run installer (will automatically handle upgrade)
3. Service restarts automatically
4. Verify new version: Check logs or registry

### Automated Update Script
```powershell
# Automated agent update script
function Update-SecureGuardAgent {
    param(
        [string]$UpdaterPath,
        [string[]]$ComputerNames
    )
    
    foreach ($computer in $ComputerNames) {
        Invoke-Command -ComputerName $computer -ScriptBlock {
            # Stop service
            Stop-Service SecureGuardAgent -Force
            
            # Backup configuration
            $backupPath = "$env:TEMP\sg_config_backup.toml"
            Copy-Item "C:\Program Files\SecureGuard\Agent\config\config.toml" $backupPath
            
            # Run update
            & $using:UpdaterPath -Silent
            
            # Restore configuration if needed
            if (Test-Path $backupPath) {
                Copy-Item $backupPath "C:\Program Files\SecureGuard\Agent\config\config.toml" -Force
            }
            
            # Start service
            Start-Service SecureGuardAgent
        }
    }
}
```

## 🛡️ Security Considerations

### Network Security
- Agent communicates via encrypted WebSocket and HTTPS
- Configurable SSL certificate validation
- API key authentication required
- Network traffic uses standard ports (configurable)

### System Security
- Service runs with minimal required privileges
- No direct file system modifications outside designated folders
- All activities logged for audit compliance
- Configuration changes require administrator privileges

### Compliance Features
- Comprehensive audit logging
- Configuration backup and restore
- Tamper-evident installation
- Digital signature verification (coming soon)

## 📊 Monitoring and Analytics

### Central Monitoring Dashboard
Access the SecureGuard management dashboard to:
- View all deployed agents
- Monitor agent health and status
- Review security events and alerts
- Manage agent configurations
- Generate compliance reports

### Key Metrics
- Agent uptime and availability
- System resource usage
- Security events detected
- Network connectivity status
- Configuration compliance

## 📞 Support and Documentation

### Installation Support
- **Documentation**: Complete guides in `/docs` folder
- **Logs**: Always include agent logs when reporting issues
- **System Info**: Include OS version and hardware specifications

### Professional Support
- **Enterprise Support**: Available for enterprise customers
- **Community Forums**: Public community support
- **Documentation Portal**: Comprehensive online documentation

---

**Document Version**: 1.0.0  
**Last Updated**: August 2025  
**Compatibility**: Windows 10/11, Windows Server 2019/2022  
**Next Review**: September 2025