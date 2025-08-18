# SecureGuard Agent Installation Guide

## Overview

The SecureGuard Agent is a security monitoring service that runs on client machines to detect threats, monitor system activities, and communicate with the SecureGuard management server.

## System Requirements

### Minimum Requirements
- **Operating System**: Windows 10/11 or Windows Server 2019/2022
- **RAM**: 512 MB available memory
- **Disk Space**: 100 MB free space
- **Network**: Internet connection for communication with SecureGuard server
- **Privileges**: Administrator access for installation

### Recommended Requirements
- **RAM**: 1 GB available memory
- **Disk Space**: 500 MB free space (for logs and updates)
- **Network**: Stable internet connection

## Pre-Installation Steps

1. **Download Agent Package**
   - Ensure you have the complete SecureGuard agent package
   - Verify the package contains:
     - `secureguard-agent.exe` (in `target\release\` folder)
     - `install_agent.bat`
     - `uninstall_agent.bat`
     - This installation guide

2. **Administrator Privileges**
   - The installation requires administrator privileges
   - Right-click on `install_agent.bat` and select "Run as administrator"

3. **Network Configuration**
   - Ensure the target machine can reach the SecureGuard server
   - Default server URL: `ws://localhost:8080/ws`
   - Default API URL: `http://localhost:8080/api/v1`

## Installation Process

### Automatic Installation (Recommended)

1. **Run Installer**
   ```batch
   Right-click install_agent.bat → "Run as administrator"
   ```

2. **Installation Steps**
   The installer will automatically:
   - Create installation directory: `C:\Program Files\SecureGuard\Agent\`
   - Copy agent executable
   - Create default configuration file
   - Install as Windows service
   - Configure service auto-start
   - Add firewall rules
   - Start the service

3. **Installation Verification**
   After installation, verify the service is running:
   ```batch
   sc query SecureGuardAgent
   ```

### Manual Installation (Advanced Users)

If you prefer manual installation:

1. **Create Installation Directory**
   ```batch
   mkdir "C:\Program Files\SecureGuard\Agent"
   ```

2. **Copy Agent Executable**
   ```batch
   copy target\release\secureguard-agent.exe "C:\Program Files\SecureGuard\Agent\"
   ```

3. **Create Configuration File**
   Create `config.toml` in the installation directory:
   ```toml
   [server]
   url = "ws://localhost:8080/ws"
   api_base = "http://localhost:8080/api/v1"

   [agent]
   name = "YOUR-COMPUTER-NAME"
   log_level = "info"
   heartbeat_interval = 30

   [security]
   encryption_enabled = true

   [logging]
   file_path = "C:\Program Files\SecureGuard\Agent\logs\agent.log"
   max_size = "10MB"
   max_files = 5
   ```

4. **Install Windows Service**
   ```batch
   sc create SecureGuardAgent binPath= "C:\Program Files\SecureGuard\Agent\secureguard-agent.exe" start= auto DisplayName= "SecureGuard Security Agent"
   ```

5. **Start Service**
   ```batch
   sc start SecureGuardAgent
   ```

## Configuration

### Configuration File Location
- **Path**: `C:\Program Files\SecureGuard\Agent\config.toml`
- **Format**: TOML configuration file

### Key Configuration Options

#### Server Settings
```toml
[server]
url = "ws://your-server:8080/ws"          # WebSocket URL for real-time communication
api_base = "http://your-server:8080/api/v1" # REST API base URL
timeout = 30                               # Connection timeout in seconds
retry_interval = 60                        # Retry interval for failed connections
```

#### Agent Settings
```toml
[agent]
name = "AGENT-001"                         # Unique agent identifier
log_level = "info"                         # Logging level: debug, info, warn, error
heartbeat_interval = 30                    # Heartbeat interval in seconds
scan_interval = 300                        # System scan interval in seconds
```

#### Security Settings
```toml
[security]
encryption_enabled = true                  # Enable end-to-end encryption
api_key = "your-api-key"                  # API key for server authentication
certificate_path = ""                      # Path to SSL certificate (optional)
```

#### Logging Settings
```toml
[logging]
file_path = "C:\Program Files\SecureGuard\Agent\logs\agent.log"
max_size = "10MB"                          # Maximum log file size
max_files = 5                              # Number of log files to retain
console_output = false                     # Enable console logging (for debugging)
```

## Service Management

### Using Windows Services Panel
1. Press `Windows + R`, type `services.msc`
2. Find "SecureGuard Security Agent"
3. Right-click for options: Start, Stop, Restart, Properties

### Using Command Line
```batch
# Start service
sc start SecureGuardAgent

# Stop service
sc stop SecureGuardAgent

# Check service status
sc query SecureGuardAgent

# View service configuration
sc qc SecureGuardAgent
```

### Using Management Tool
Run the management tool for easy service control:
```batch
"C:\Program Files\SecureGuard\Agent\manage.bat"
```

## Monitoring and Troubleshooting

### Log Files
- **Location**: `C:\Program Files\SecureGuard\Agent\logs\`
- **Main Log**: `agent.log`
- **Log Rotation**: Automatic (configurable)

### Common Issues

#### Service Won't Start
1. Check log files for error messages
2. Verify configuration file syntax
3. Ensure network connectivity to server
4. Check Windows Event Viewer for system errors

#### High CPU/Memory Usage
1. Check scan_interval setting (increase if too frequent)
2. Review log_level (set to "error" or "warn" for production)
3. Monitor log file sizes

#### Network Connectivity Issues
1. Verify server URL and port accessibility
2. Check firewall rules
3. Test network connectivity: `telnet server-ip 8080`

### Performance Tuning

#### For High-Performance Environments
```toml
[agent]
log_level = "warn"
heartbeat_interval = 60
scan_interval = 600

[logging]
console_output = false
max_size = "5MB"
```

#### For Development/Testing
```toml
[agent]
log_level = "debug"
heartbeat_interval = 10
scan_interval = 120

[logging]
console_output = true
max_size = "50MB"
```

## Uninstallation

### Automatic Uninstallation (Recommended)
1. Run the uninstaller as administrator:
   ```batch
   Right-click uninstall_agent.bat → "Run as administrator"
   ```

2. Or use the installed uninstaller:
   ```batch
   "C:\Program Files\SecureGuard\Agent\uninstall.bat"
   ```

### Manual Uninstallation
If automatic uninstallation fails:

1. **Stop the Service**
   ```batch
   sc stop SecureGuardAgent
   ```

2. **Remove the Service**
   ```batch
   sc delete SecureGuardAgent
   ```

3. **Remove Installation Directory**
   ```batch
   rmdir /s "C:\Program Files\SecureGuard"
   ```

4. **Remove Firewall Rules**
   ```batch
   netsh advfirewall firewall delete rule name="SecureGuard Agent"
   ```

## Security Considerations

### Network Security
- Agent communicates using WebSocket and HTTPS
- All data is encrypted in transit
- API key authentication required
- Optional SSL certificate validation

### System Security
- Agent runs as Windows service with minimal privileges
- No direct file system modifications outside designated directories
- All activities logged for audit purposes

### Data Privacy
- Agent collects only security-relevant system information
- No personal files or data are accessed
- All collected data is encrypted before transmission

## Support and Maintenance

### Log Monitoring
Regular monitoring of agent logs is recommended:
- Check for connectivity issues
- Monitor resource usage
- Review security alerts

### Updates
- Agent updates are handled automatically by the management server
- Manual updates can be performed by replacing the executable
- Always backup configuration before updating

### Backup Configuration
Important files to backup:
- `C:\Program Files\SecureGuard\Agent\config.toml`
- Agent logs (if needed for compliance)

## Troubleshooting Guide

### Installation Issues

| Problem | Solution |
|---------|----------|
| "Access Denied" error | Run installer as Administrator |
| Service creation fails | Check Windows services permissions |
| Firewall blocks agent | Add manual firewall exception |
| Configuration error | Validate TOML syntax |

### Runtime Issues

| Problem | Solution |
|---------|----------|
| High CPU usage | Increase scan_interval |
| Network timeouts | Check server connectivity |
| Log files too large | Reduce max_size setting |
| Service crashes | Check Windows Event Viewer |

## Contact Information

For additional support or questions:
- **Documentation**: Check project README and documentation
- **Logs**: Always include relevant log files when reporting issues
- **System Info**: Include OS version and hardware specifications

---

**Version**: 1.0.0  
**Last Updated**: August 2025  
**Compatibility**: Windows 10/11, Windows Server 2019/2022