# SecureGuard Service Control Scripts

Cross-platform service management scripts for SecureGuard with automatic OS detection and intelligent port conflict resolution.

## 🚀 Quick Start

### Linux / macOS / Git Bash (Windows)
```bash
./scripts/myservice          # Show help
./scripts/myservice dev      # Start development environment
./scripts/myservice start    # Start production environment
./scripts/myservice stop     # Stop all services
```

### Windows Command Prompt / PowerShell
```cmd
scripts\myservice.bat        # Show help
scripts\myservice.bat dev    # Start development environment  
scripts\myservice.bat start  # Start production environment
scripts\myservice.bat stop   # Stop all services
```

## 📋 Commands

| Command | Description | Environment |
|---------|-------------|-------------|
| `start` | 🚀 Start production environment | Release builds, prod database |
| `dev` | 🔧 Start development environment | Debug builds, dev database, hot reload |
| `stop` | ⛔ Stop all services | Kills both dev and prod environments |
| `help` | ❓ Show help message | - |

## 🔍 Key Features

### ✅ **Cross-Platform Compatibility**
- **Linux** - Native support with systemd/service management
- **macOS** - Full compatibility with Darwin systems  
- **Windows** - Both batch (.bat) and shell script versions
- **WSL** - Windows Subsystem for Linux support
- **Git Bash** - Automatically detected and supported

### ✅ **Intelligent Port Management**
- **Automatic port conflict detection** before starting services
- **Smart cleanup** of existing processes on ports 3000 and 3002
- **Graceful termination** with fallback to force-kill if needed
- **Port availability verification** after cleanup

### ✅ **Docker Integration**
- **Auto-detection** of Docker Desktop/Engine status
- **Automatic startup** of Docker Desktop on Windows if needed
- **Health checking** before attempting to start containers
- **Support for multiple docker-compose configurations**

### ✅ **Smart Process Management**
- **Service-specific terminals** for better debugging
- **Background process handling** with proper cleanup
- **PID tracking and management**
- **Graceful shutdown sequences**

## 🛠️ Technical Details

### OS Detection Logic
The script automatically detects:
```bash
# Linux distributions
if [[ "$OSTYPE" == "linux-gnu"* ]]; then OS="linux"

# macOS
elif [[ "$OSTYPE" == "darwin"* ]]; then OS="macos"

# Windows environments  
elif [[ "$OSTYPE" == "cygwin" || "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then OS="windows"

# WSL (Windows Subsystem for Linux)
elif [[ -n "$WSL_DISTRO_NAME" ]]; then OS="wsl"

# Git Bash detection
if [[ "$OS" == "windows" && -n "$MSYSTEM" ]]; then OS="gitbash"
```

### Port Checking Methods
Different tools used based on OS availability:

| OS | Primary | Secondary | Tertiary |
|----|---------|-----------|----------|
| Linux | `lsof -ti:PORT` | `netstat -tlpn` | `ss -tlpn` |
| macOS | `lsof -ti:PORT` | `netstat -tlpn` | - |
| Windows | `netstat -ano` | `tasklist` | - |
| WSL | `lsof -ti:PORT` | `netstat -tlpn` | `ss -tlpn` |

### Service Ports
- **Frontend (React)**: 3002
- **Backend (Rust)**: 3000  
- **Database (PostgreSQL)**: 5432

## 🔧 Configuration

### Environment Variables

#### Development Environment
```bash
DATABASE_URL="postgresql://secureguard:password@localhost:5432/secureguard_dev"
RUST_LOG="secureguard_api=debug,tower_http=debug,axum=debug"
NODE_ENV="development"
```

#### Production Environment  
```bash
DATABASE_URL="postgresql://secureguard:password@localhost:5432/secureguard_prod"
RUST_LOG="secureguard_api=info"
NODE_ENV="production"
```

### Docker Configuration
The script supports multiple docker-compose files:
- `docker-compose.yml` (default)
- `docker-compose.prod.yml` (production-specific)

## 🚨 Troubleshooting

### Port Already in Use
The script automatically handles this, but if you encounter issues:

```bash
# Manual port checking (Linux/macOS)
lsof -ti:3002
netstat -tlpn | grep 3002

# Manual port checking (Windows)  
netstat -ano | findstr :3002
```

### Docker Issues
```bash
# Check Docker status
docker version

# Restart Docker Desktop (Windows)
# The script does this automatically, but manual restart:
# Close Docker Desktop and restart from Start Menu

# Linux Docker service restart
sudo systemctl restart docker
```

### Permission Issues (Linux/macOS)
```bash
# Make scripts executable
chmod +x scripts/myservice
chmod +x scripts/demo_port_check

# Docker permission issues
sudo usermod -aG docker $USER
# Then log out and back in
```

## 📁 File Structure

```
scripts/
├── myservice           # Cross-platform shell script (Linux/macOS/Git Bash)
├── myservice.bat       # Windows batch script  
├── demo_port_check     # Port checking demonstration
├── README.md           # This documentation
└── test_new_script.bat # Test script for Windows
```

## 🎯 Usage Examples

### Development Workflow
```bash
# Start development environment with hot reload
./scripts/myservice dev

# Make code changes... frontend and backend will auto-reload

# Stop everything when done
./scripts/myservice stop
```

### Production Deployment
```bash
# Start optimized production build
./scripts/myservice start

# Services are now running:
# - Frontend: http://localhost:3002 (production build)
# - API: http://localhost:3000/api  
# - Database: localhost:5432

# Stop when done
./scripts/myservice stop
```

### Port Conflict Resolution
```bash
# If you see port conflicts:
./scripts/myservice dev
# Output:
# 🔍 Checking for existing React Frontend processes (port 3002)...
# ⚠️  Port 3002 is already in use by React Frontend
# 🔥 Killing existing React Frontend processes: 12345
# ✅ Port 3002 is now available for React Frontend
```

## 🔮 Advanced Features

### Multi-Terminal Support
The script intelligently opens services in separate terminals when available:
- **Linux**: `gnome-terminal`, `xterm`, `terminal` (fallback to background)
- **macOS**: Native terminal support  
- **Windows**: `cmd` windows with titles
- **Git Bash**: Windows terminal integration

### Cleanup Strategies
1. **Graceful termination** (SIGTERM)
2. **Force termination** (SIGKILL) after timeout  
3. **Process tree cleanup** to handle child processes
4. **Docker container cleanup** for all configurations

## 📊 Monitoring

Both scripts provide detailed feedback:
- ✅ Success indicators with green checkmarks
- ⚠️ Warnings with yellow alerts  
- ❌ Errors with red X marks
- 🔍 Process information with magnifying glass
- 🚀 Progress indicators with rockets

## 🔄 Migration from Old Script

### Old Usage
```bash
myservice.bat start dev    # Development
myservice.bat start prod   # Production  
myservice.bat stop dev     # Stop dev
myservice.bat stop prod    # Stop prod
```

### New Usage  
```bash
./myservice dev     # Development (was: start dev)
./myservice start   # Production (was: start prod)
./myservice stop    # Stop everything (was: stop dev + stop prod)
```

**Benefits of new approach:**
- 🎯 Simpler command structure
- 🔄 One stop command handles everything
- 🚀 Production is now default for `start`
- 🔍 Automatic conflict detection and resolution
- 🌍 True cross-platform compatibility

---

*SecureGuard Service Control Scripts - Professional-grade service management for any platform* 🛡️