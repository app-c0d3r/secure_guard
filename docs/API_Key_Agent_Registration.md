# API Key-Based Agent Registration System

## 🔐 **Overview**

SecureGuard now supports **API key-based agent registration** that directly links agents to user accounts. This solves the problem of users managing multiple devices and provides a secure, professional deployment experience.

## 🎯 **Problem Solved**

**Before**: Agents were only linked to `tenant_id` with no direct user connection
**After**: Agents are linked to specific users via secure API keys, allowing users to:
- Register and manage their own devices
- Track multiple devices per user
- Generate secure API keys for device registration
- Monitor all their devices from their dashboard

## 🔧 **How It Works**

### **1. User Generates API Key**
```bash
# User creates API key in dashboard
POST /api/v1/users/api-keys
{
  "key_name": "Home Laptop",
  "expires_in_days": 365
}

# Response (API key shown only once!)
{
  "key_id": "uuid",
  "api_key": "sg_abcd1234_xyz789012345678901234567890",
  "key_prefix": "sg_abcd1234",
  "key_name": "Home Laptop", 
  "expires_at": "2025-08-18T00:00:00Z"
}
```

### **2. Agent Registration During Installation**
```bash
# Agent registers using API key
POST /api/v1/agents/register
{
  "api_key": "sg_abcd1234_xyz789012345678901234567890",
  "device_name": "John-Laptop",
  "hardware_fingerprint": "hw_12345...",
  "os_info": {"platform": "Windows 11"},
  "version": "1.0.0"
}
```

### **3. System Links Agent to User**
- API key is validated and user identified
- Agent is created with `user_id` link
- Agent shows up in user's device list
- User can manage this specific device

## 🛠️ **Database Schema**

### **API Keys Table**
```sql
CREATE TABLE users.api_keys (
    key_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users.users(user_id),
    key_hash TEXT NOT NULL,              -- bcrypt hash of actual key
    key_name VARCHAR(100) NOT NULL,       -- User-friendly name
    key_prefix VARCHAR(10) NOT NULL,      -- First 8 chars for identification
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    expires_at TIMESTAMPTZ,              -- Optional expiration
    last_used TIMESTAMPTZ,               -- Track usage
    usage_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### **Enhanced Agents Table**
```sql
ALTER TABLE agents.endpoints ADD COLUMN:
- user_id UUID REFERENCES users.users(user_id)        -- Direct user link
- device_name VARCHAR(100)                            -- User-friendly name
- registered_via_key_id UUID REFERENCES users.api_keys(key_id)  -- Audit trail
```

### **One-Time Registration Tokens** (Alternative)
```sql
CREATE TABLE users.registration_tokens (
    token_id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users.users(user_id),
    token_hash TEXT NOT NULL,
    device_name VARCHAR(100) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (now() + INTERVAL '24 hours'),
    is_used BOOLEAN NOT NULL DEFAULT FALSE
);
```

## 📋 **API Endpoints**

### **API Key Management**
```bash
# Generate new API key
POST /api/v1/users/api-keys
Authorization: Bearer {jwt_token}
{
  "key_name": "Device Name",
  "expires_in_days": 365  # optional
}

# List user's API keys  
GET /api/v1/users/api-keys
Authorization: Bearer {jwt_token}

# Revoke API key
DELETE /api/v1/users/api-keys/{key_id}
Authorization: Bearer {jwt_token}
```

### **Agent Registration**
```bash
# Register with API key
POST /api/v1/agents/register-with-key
{
  "api_key": "sg_...",
  "device_name": "My Laptop", 
  "hardware_fingerprint": "...",
  "os_info": {...},
  "version": "1.0.0"
}

# Register with one-time token (alternative)
POST /api/v1/agents/register-with-token  
{
  "registration_token": "rt_...",
  "hardware_fingerprint": "...",
  "os_info": {...},
  "version": "1.0.0"
}
```

### **User Device Management**
```bash
# List user's devices
GET /api/v1/users/agents
Authorization: Bearer {jwt_token}

# Get specific device
GET /api/v1/agents/{agent_id}
Authorization: Bearer {jwt_token}
```

## 🖥️ **Installer Integration**

### **NSIS Installer (EXE)**
- **Configuration Page**: Prompts for API key and device name
- **Validation**: Checks API key format (`sg_*`)  
- **Config Generation**: Creates `config.toml` with API key
- **Professional UX**: Clear instructions and validation

```nsis
# User sees:
"Enter your SecureGuard API key (from your dashboard):"
[sg_abcd1234_xyz789...]

"Device name:"  
[John-Laptop]

"You can generate API keys in your SecureGuard dashboard 
under 'Profile > API Keys'"
```

### **PowerShell Installer**
- **Required Parameter**: `-APIKey` (mandatory)
- **Validation**: Pattern matching for API key format
- **Automated**: Perfect for enterprise deployment

```powershell
# Usage:
.\Install-SecureGuardAgent.ps1 -APIKey "sg_abc123_xyz789" -DeviceName "Server-01"

# Parameter validation ensures correct format
[ValidatePattern('^sg_[a-f0-9]{8}_[a-f0-9]{20}$')]
```

### **WiX MSI Installer** 
- **Properties**: `APIKEY` and `DEVICENAME` properties
- **Group Policy**: Can be deployed with pre-configured keys
- **Silent Install**: `msiexec /i agent.msi APIKEY=sg_... DEVICENAME=PC-01 /quiet`

## 🔒 **Security Features**

### **API Key Security**
- **Format**: `sg_{8_hex_chars}_{20_hex_chars}` (33 chars total)
- **Storage**: Keys are bcrypt hashed in database
- **Prefix Matching**: First 8 chars used for efficient lookup
- **One-Time Display**: Full key shown only during creation
- **Expiration**: Optional expiration dates
- **Usage Tracking**: Last used timestamp and usage count

### **Registration Security**
- **Hardware Fingerprinting**: Prevents duplicate registrations
- **API Key Validation**: Cryptographic verification
- **User Authentication**: API keys tied to authenticated users
- **Audit Trail**: Tracks which key registered which agent

### **Alternative: One-Time Tokens**
- **Single Use**: Token becomes invalid after registration
- **24-Hour Expiry**: Automatic cleanup
- **Device-Specific**: Pre-assigned device name
- **Secure Generation**: Cryptographically secure random tokens

## 📱 **User Experience Flow**

### **For End Users**
1. **Dashboard**: Go to "Profile > API Keys"
2. **Generate**: Click "Add Device" → Enter device name
3. **Copy Key**: API key shown once (copy immediately!)
4. **Install Agent**: Download installer, enter API key
5. **Verify**: Device appears in dashboard automatically

### **For IT Administrators** 
1. **Bulk Generation**: Generate multiple API keys
2. **Deployment**: Use PowerShell installer with API keys
3. **Group Policy**: Deploy MSI with API key properties
4. **Monitoring**: Track device registration in admin dashboard

### **For DevOps Teams**
1. **API Integration**: Programmatically generate keys
2. **Automation**: Integrate with deployment pipelines  
3. **Cloud Deployment**: Use in Azure/AWS automation
4. **Container Support**: Docker/Kubernetes ready

## 🎨 **Professional Features**

### **Dashboard Integration**
- **Device List**: "My Devices" page showing all user's agents
- **Device Details**: Status, last seen, version, registration method
- **Key Management**: Create, view, revoke API keys
- **Registration History**: Audit log of device registrations

### **Enterprise Features**
- **Bulk Operations**: Generate multiple keys at once
- **Access Control**: Role-based API key permissions
- **Expiration Management**: Automatic key rotation
- **Compliance Reporting**: Registration audit trails

## 📊 **Benefits**

### **For Users**
✅ **Self-Service**: Register devices without admin help  
✅ **Multi-Device**: Manage laptops, desktops, servers  
✅ **Visibility**: See all devices in one dashboard  
✅ **Control**: Revoke compromised keys instantly  

### **For Organizations**
✅ **Scalability**: Users manage their own devices  
✅ **Security**: Cryptographic device authentication  
✅ **Compliance**: Full audit trail of device access  
✅ **Automation**: API-driven deployment workflows  

### **For Support Teams**  
✅ **Reduced Tickets**: Users self-register devices
✅ **Clear Ownership**: Devices linked to specific users
✅ **Easy Troubleshooting**: Device history and registration details
✅ **Bulk Management**: API-driven operations

## 🚀 **Migration Path**

### **Existing Installations**
```sql
-- Migration script to link existing agents to users
UPDATE agents.endpoints SET user_id = (
    SELECT u.user_id FROM users.users u 
    WHERE u.tenant_id = agents.endpoints.tenant_id 
    LIMIT 1
) WHERE user_id IS NULL;
```

### **Backward Compatibility**
- **Legacy Method**: Old registration endpoint still works
- **Gradual Migration**: New installs use API keys automatically  
- **Admin Override**: Admins can still register agents directly

## 💡 **Usage Examples**

### **Home User**
```bash
# Sarah has 3 devices: laptop, desktop, home server
# She generates 3 API keys from her dashboard:
# - "Sarah-Laptop" → sg_abc123_...
# - "Sarah-Desktop" → sg_def456_...  
# - "Home-Server" → sg_ghi789_...
```

### **Small Business**
```powershell
# IT admin generates API keys for team
# Each employee gets their own key for self-registration
.\Install-SecureGuardAgent.ps1 -APIKey "sg_emp001_..." -DeviceName "John-Laptop"
.\Install-SecureGuardAgent.ps1 -APIKey "sg_emp002_..." -DeviceName "Jane-Desktop"  
```

### **Enterprise**
```bash
# Mass deployment with SCCM/Group Policy
msiexec /i SecureGuardAgent.msi APIKEY=sg_dept001_... DEVICENAME=%COMPUTERNAME% /quiet

# Or PowerShell with configuration management
Invoke-Command -ComputerName $servers -ScriptBlock {
    .\Install-SecureGuardAgent.ps1 -APIKey $using:apiKey -DeviceName $env:COMPUTERNAME
}
```

---

## 📞 **Support**

**✅ Ready for Production**: This system is fully implemented and tested  
**📚 Documentation**: Complete API documentation available  
**🔧 Management Tools**: Professional admin dashboard included  
**💬 Support**: Enterprise support available for deployment assistance  

Your SecureGuard deployment now supports **professional-grade device management** with secure API key authentication! 🎉