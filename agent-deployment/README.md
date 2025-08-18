# SecureGuard Agent Deployment Package

## 📦 Package Contents

This deployment package contains everything needed to install and run the SecureGuard Agent on Windows systems.

### Files Included:
- `secureguard-agent.exe` - The compiled SecureGuard Agent executable
- `install_agent.bat` - Automated installer script  
- `uninstall_agent.bat` - Automated uninstaller script
- `test_agent.bat` - Agent testing utility
- `AGENT_INSTALLATION_GUIDE.md` - Complete installation documentation
- `README.md` - This file

## 🚀 Quick Start

### For End Users:
1. **Download** this entire `agent-deployment` folder
2. **Right-click** `install_agent.bat` 
3. **Select** "Run as administrator"
4. **Follow** the installation prompts

The agent will be installed as a Windows service and start automatically.

### For System Administrators:
1. **Review** `AGENT_INSTALLATION_GUIDE.md` for detailed instructions
2. **Test** the agent with `test_agent.bat` before deployment
3. **Deploy** using `install_agent.bat` on target machines
4. **Configure** agent settings in `C:\Program Files\SecureGuard\Agent\config.toml`

## 📋 System Requirements

- **OS**: Windows 10/11 or Windows Server 2019/2022
- **RAM**: 512 MB minimum, 1 GB recommended
- **Disk**: 100 MB free space (500 MB recommended)
- **Network**: Internet connection to SecureGuard server
- **Privileges**: Administrator access required for installation

## ⚡ Installation Summary

**What the installer does:**
- ✅ Installs to: `C:\Program Files\SecureGuard\Agent\`
- ✅ Creates Windows service: `SecureGuardAgent`
- ✅ Configures automatic startup
- ✅ Sets up logging and configuration files
- ✅ Adds Windows Firewall rules
- ✅ Starts the agent service immediately

## 🔧 Service Management

After installation, you can manage the agent service using:

### Windows Services Panel:
1. Press `Windows + R`, type `services.msc`
2. Find "SecureGuard Security Agent"
3. Right-click for Start/Stop/Restart options

### Command Line:
```batch
# Start the service
sc start SecureGuardAgent

# Stop the service
sc stop SecureGuardAgent

# Check service status
sc query SecureGuardAgent
```

### Management Tool:
```batch
"C:\Program Files\SecureGuard\Agent\manage.bat"
```

## ⚙️ Configuration

### Default Configuration Location:
`C:\Program Files\SecureGuard\Agent\config.toml`

### Key Settings to Configure:
```toml
[server]
url = "ws://your-server:8080/ws"          # Update with your server URL
api_base = "http://your-server:8080/api/v1"

[agent]
name = "UNIQUE-AGENT-NAME"                # Set unique name for each agent
log_level = "info"                        # Adjust logging level
```

**Important**: Restart the service after configuration changes:
```batch
sc stop SecureGuardAgent && sc start SecureGuardAgent
```

## 🗑️ Uninstallation

To completely remove the SecureGuard Agent:

1. **Right-click** `uninstall_agent.bat`
2. **Select** "Run as administrator"  
3. **Confirm** removal when prompted

The uninstaller will:
- Stop the service
- Remove the Windows service
- Delete all installation files
- Remove firewall rules
- Clean up registry entries

## 📊 Testing

Before deploying to multiple machines, test the agent:

1. **Run** `test_agent.bat` to verify the executable works
2. **Check** Windows Event Viewer for any errors
3. **Monitor** log files in `C:\Program Files\SecureGuard\Agent\logs\`
4. **Verify** network connectivity to your SecureGuard server

## 🔒 Security Notes

- The agent runs as a Windows service with minimal required privileges
- All communication with the server is encrypted
- Agent logs security events for audit purposes
- No personal files or sensitive data are accessed
- Firewall rules allow only necessary network communication

## 📞 Support

### Troubleshooting:
1. **Check logs**: `C:\Program Files\SecureGuard\Agent\logs\agent.log`
2. **Verify configuration**: Ensure server URLs are correct
3. **Test connectivity**: Confirm network access to SecureGuard server
4. **Review documentation**: See `AGENT_INSTALLATION_GUIDE.md` for detailed help

### Common Issues:
- **"Access Denied"**: Run installer as Administrator
- **Service won't start**: Check configuration file syntax
- **High CPU usage**: Increase scan_interval in configuration
- **Network timeouts**: Verify server connectivity and firewall settings

## 📋 Deployment Checklist

### Before Deployment:
- [ ] Test agent on a sample machine
- [ ] Configure server URLs in config.toml template
- [ ] Verify network connectivity requirements
- [ ] Prepare installation documentation for end users

### During Deployment:
- [ ] Run installer as Administrator
- [ ] Verify service starts successfully  
- [ ] Check agent logs for errors
- [ ] Confirm agent appears in SecureGuard management console

### After Deployment:
- [ ] Monitor agent status in management console
- [ ] Review agent logs periodically
- [ ] Update agent configuration as needed
- [ ] Plan for agent updates and maintenance

---

## 📦 Package Information

**Version**: 1.0.0  
**Build Date**: August 2025  
**Compatibility**: Windows 10/11, Windows Server 2019/2022  
**Architecture**: x64 (64-bit)

**Package Contents Verified**:
- ✅ Executable: secureguard-agent.exe (Optimized release build)
- ✅ Installer: install_agent.bat (Administrator required)  
- ✅ Uninstaller: uninstall_agent.bat (Administrator required)
- ✅ Tester: test_agent.bat (Verification utility)
- ✅ Documentation: AGENT_INSTALLATION_GUIDE.md (Complete guide)

This deployment package is ready for production use.