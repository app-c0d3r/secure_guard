# SecureGuard - Kamal Deployment Guide

**Document Version:** 1.0  
**Last Updated:** August 18, 2025  
**For:** Production deployment with Kamal

## 🚀 Overview

This guide covers deploying SecureGuard using Kamal (formerly MRSK) for production deployment to your servers.

## 📋 Prerequisites

### Local Setup
1. **Ruby 3.2+** (for Kamal)
2. **Docker** (for building images)
3. **Git** (for source control)

### Server Requirements
1. **Ubuntu 20.04+ or Debian 11+**
2. **Docker installed on server**
3. **SSH access with key-based authentication**
4. **Minimum 2GB RAM, 2 CPU cores**

## 🛠️ Installation Steps

### 1. Install Kamal Locally

```bash
# Install Ruby (if not already installed)
# On Windows with WSL or use Ruby installer

# Install Kamal
gem install kamal

# Verify installation
kamal version
```

### 2. Prepare Your Server

```bash
# SSH into your server
ssh your-user@your-server-ip

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Add your user to docker group
sudo usermod -aG docker $USER

# Create deploy user (recommended)
sudo adduser deploy
sudo usermod -aG docker deploy
sudo mkdir /home/deploy/.ssh
sudo cp ~/.ssh/authorized_keys /home/deploy/.ssh/
sudo chown -R deploy:deploy /home/deploy/.ssh
sudo chmod 700 /home/deploy/.ssh
sudo chmod 600 /home/deploy/.ssh/authorized_keys
```

### 3. Configure Kamal for SecureGuard

Edit `config/deploy.yml`:

```yaml
service: secureguard
image: your-registry/secureguard

ssh:
  user: deploy
  
servers:
  web:
    - YOUR_SERVER_IP

registry:
  server: ghcr.io  # or your registry
  username: YOUR_GITHUB_USERNAME
  password:
    - KAMAL_REGISTRY_PASSWORD

env:
  clear:
    NODE_ENV: production
    RUST_LOG: secureguard_api=info
  secret:
    - DATABASE_URL
    - JWT_SECRET
    - REDIS_URL

healthcheck:
  path: /health
  port: 3000
  max_attempts: 7
  interval: 20s
```

### 4. Set Environment Variables

Create `.kamal/secrets`:

```bash
# Database connection
DATABASE_URL=postgresql://user:password@localhost:5432/secureguard_prod

# JWT secret (generate a secure one)
JWT_SECRET=your-super-secure-jwt-secret-here

# Redis (if using)
REDIS_URL=redis://localhost:6379

# Registry password
KAMAL_REGISTRY_PASSWORD=your-github-token-or-registry-password
```

### 5. Initial Deployment

```bash
# Initialize Kamal configuration
kamal init

# Setup the server (first time only)
kamal setup

# Deploy the application
kamal deploy
```

## 🏗️ Current Issues & Solutions

### Issue 1: Port Configuration ✅ FIXED
- **Problem**: App was looking for old `dashboard` folder
- **Solution**: Updated `myservice.bat` to use `frontend` folder
- **Result**: Now correctly starts React app on port 3002

### Issue 2: Theme System Not Visible ✅ FIXED
- **Problem**: Theme classes not being applied on page load
- **Solution**: Added theme initialization script to `index.html`
- **Result**: Dark/light theme should now work properly

### Issue 3: Mock Data
- **Current**: Using demo/mock data for development
- **For Production**: You'll need to:
  1. Set up PostgreSQL database
  2. Run migrations: `sqlx migrate run`
  3. Configure real database connection

## 🔧 Local Development Fix

Your current setup should now work correctly:

```bash
# Start development environment
.\scripts\myservice.bat start dev

# Or start production environment  
.\scripts\myservice.bat start prod
```

**Fixed Issues:**
- ✅ Port 3002 is correct for frontend
- ✅ Script now uses `frontend/` instead of `dashboard/`
- ✅ Theme system initialized properly
- ✅ Vite config updated for proper PORT handling

## 🌐 Access Points

After running the script:
- **Frontend**: http://localhost:3002
- **Backend API**: http://localhost:3000
- **Demo Login**: admin@company.com / SecurePass123!
- **Theme Toggle**: Look for sun/moon icon in header navigation

## 📊 Kamal Commands Reference

```bash
# Deployment
kamal deploy                    # Deploy latest version
kamal deploy --skip-push       # Deploy without building new image

# Management
kamal app logs                 # View application logs
kamal app logs --follow        # Follow logs in real-time
kamal app console              # Open app console
kamal app exec bash            # SSH into running container

# Server management
kamal server reboot            # Reboot servers
kamal server upgrade           # Upgrade Kamal on servers

# Database (if using accessories)
kamal accessory boot db        # Start database
kamal accessory logs db        # View database logs
kamal accessory exec db "psql -U postgres"

# Rollback
kamal rollback VERSION         # Rollback to specific version
```

## 🐳 Docker Build Process

The Dockerfile includes:
1. **Multi-stage build**: Rust backend + React frontend
2. **Optimized layers**: Faster rebuilds
3. **Security**: Non-root user
4. **Health checks**: Built-in monitoring
5. **Static assets**: Frontend served by backend

## 🔒 Security Considerations

### SSL/TLS Setup
```bash
# Using Let's Encrypt with Caddy (recommended)
# Add to your server:
sudo apt install caddy

# Caddyfile
your-domain.com {
    reverse_proxy localhost:3000
}
```

### Environment Security
- Use strong JWT secrets (32+ characters)
- Secure database passwords
- Limit SSH access
- Enable firewall (ufw)
- Regular security updates

## 🚀 Production Deployment Steps

1. **Prepare your domain/server**
2. **Configure DNS** to point to your server
3. **Update `config/deploy.yml`** with your settings
4. **Set secrets** in `.kamal/secrets`
5. **Run `kamal setup`** (first time)
6. **Run `kamal deploy`**
7. **Configure SSL** (Caddy/nginx)
8. **Test the deployment**

## 🔄 Continuous Deployment

For automatic deployments:

```yaml
# .github/workflows/deploy.yml
name: Deploy
on:
  push:
    branches: [ main ]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Set up Ruby
      uses: ruby/setup-ruby@v1
      with:
        ruby-version: 3.2
    - name: Install Kamal
      run: gem install kamal
    - name: Deploy
      run: kamal deploy
      env:
        KAMAL_REGISTRY_PASSWORD: ${{ secrets.GITHUB_TOKEN }}
```

## 🆘 Troubleshooting

### Port Issues
```bash
# Check what's running on ports
netstat -tulpn | grep :3000
netstat -tulpn | grep :3002

# Kill processes if needed
sudo pkill -f "cargo run"
sudo pkill -f "npm run"
```

### Theme Issues
1. Open browser developer tools
2. Check if `dark` or `light` class is on `<html>` element
3. Verify localStorage has `secureguard-theme` key
4. Toggle theme switcher in header navigation

### Kamal Issues
```bash
# Reset Kamal state
kamal app remove
kamal setup

# Debug deployment
kamal app logs --follow
kamal server logs
```

---

**Next Steps**: Test local environment, then proceed with server setup and Kamal deployment!