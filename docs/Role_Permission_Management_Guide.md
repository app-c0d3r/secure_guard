# 🔐 **SecureGuard Role & Permission Management Guide**

## 🎯 **Admin Role Management Overview**

The SecureGuard Role-Based Access Control (RBAC) system provides comprehensive control over who can access what areas of the system. As an admin, you can precisely control user access to secrets, important areas, and other sensitive views through a granular permission system.

## 🏗️ **Role Hierarchy & System Roles**

### **📊 Role Hierarchy (100 = Highest Access)**

```
🏛️ System Administrator (100) - Full system access
├── 🛡️ Security Analyst (80) - Security monitoring & incident response  
├── 👤 Admin (70) - User & organizational management
├── 📋 Manager (50) - Team oversight & management
├── ⚡ Power User (30) - Advanced user capabilities
├── 👥 User (10) - Standard user access
├── 👁️ Read Only (5) - View-only access
└── 🚪 Guest (1) - Minimal access for temporary users
```

### **🔑 Permission Categories**

#### **🚨 Critical Permissions (Sensitivity Level 4)**
- **`system.admin`** - Full system administration
- **`secrets.read`** - View API keys and secrets
- **`secrets.delete`** - Delete secrets (highest risk)

#### **⚠️ High Sensitivity (Level 3)**  
- **`users.roles`** - Manage user roles and permissions
- **`users.delete`** - Delete user accounts
- **`secrets.create`** - Create new secrets
- **`secrets.update`** - Modify existing secrets
- **`subscriptions.delete`** - Delete subscription plans
- **`agents.control`** - Send commands to agents

#### **📋 Medium Sensitivity (Level 2)**
- **`users.create`** - Create user accounts
- **`users.update`** - Modify user profiles
- **`agents.update`** - Modify agent configurations
- **`security.monitoring`** - Access security dashboards

#### **📖 Low Sensitivity (Level 1)**
- **`users.read`** - View user information
- **`agents.read`** - View agent status
- **`dashboard.analytics`** - View analytics dashboards

## 🛠️ **Admin Interface: Managing User Access**

### **📋 View All Users with Their Permissions**

```bash
# Get all users with role summary
GET /api/admin/roles/users

# Filter users by access type
GET /api/admin/roles/users?filter=secrets    # Users with secret access
GET /api/admin/roles/users?filter=users      # Users who can manage other users  
GET /api/admin/roles/users?filter=system     # Users with system admin access
```

**Response Example:**
```json
{
  "success": true,
  "data": [
    {
      "user_id": "uuid-here",
      "username": "alice.smith", 
      "email": "alice@company.com",
      "primary_role": "Security Analyst",
      "all_roles": ["Security Analyst", "User"],
      "total_permissions": 25,
      "high_sensitivity_permissions": 8,
      "can_access_secrets": true,
      "can_manage_users": false,
      "can_admin_system": false
    }
  ],
  "meta": {
    "users_with_secrets_access": 5,
    "users_with_user_management": 3,
    "users_with_system_admin": 2
  }
}
```

### **🔍 Get Users with Sensitive Access**

```bash
# Get all users who have access to sensitive areas
GET /api/admin/roles/users/sensitive-access
```

This shows you exactly who can access:
- 🔑 **Secrets & API Keys**
- 👥 **User Management** 
- 🏛️ **System Administration**

### **👤 Manage Individual User Roles**

#### **View User's Current Roles:**
```bash
GET /api/admin/roles/users/{user_id}/roles
```

#### **Assign Role to User:**
```bash
POST /api/admin/roles/users/{user_id}/roles
{
  "role_id": "security-analyst-role-uuid",
  "expires_at": "2024-12-31T23:59:59Z",  // Optional expiration
  "context": "Temporary security analyst role for incident response"
}
```

#### **Remove Role from User:**
```bash
DELETE /api/admin/roles/users/{user_id}/roles/{role_id}?reason=Role no longer needed
```

#### **Set Primary Role:**
```bash
PUT /api/admin/roles/users/{user_id}/primary-role
{
  "role_id": "admin-role-uuid"
}
```

### **🔐 Check Secret Access**

```bash
# Check if specific user can access secrets
GET /api/admin/roles/users/{user_id}/secret-access

# Response:
{
  "user_id": "uuid",
  "can_access_secrets": true,
  "checked_at": "2024-01-15T10:30:00Z"
}
```

## 🎯 **Permission Management**

### **📋 View All Available Permissions**

```bash
GET /api/admin/roles/permissions
```

**Response organized by category:**
```json
{
  "system": [
    {
      "permission_slug": "system.admin",
      "display_name": "System Administration", 
      "description": "Full system administration access",
      "sensitivity_level": 4
    }
  ],
  "security": [
    {
      "permission_slug": "secrets.read",
      "display_name": "View Secrets",
      "description": "View API keys and other secrets", 
      "sensitivity_level": 4
    }
  ],
  "users": [
    {
      "permission_slug": "users.roles",
      "display_name": "Manage User Roles",
      "description": "Assign and modify user roles",
      "sensitivity_level": 3
    }
  ]
}
```

### **🔧 Modify Role Permissions**

```bash
# Add/remove permissions from a role
PUT /api/admin/roles/roles/{role_id}/permissions
{
  "add_permissions": [
    "secrets-read-permission-uuid",
    "users-update-permission-uuid"  
  ],
  "remove_permissions": [
    "old-permission-uuid"
  ]
}
```

### **📊 Get Permission Categories**

```bash
GET /api/admin/roles/permissions/categories
```

Shows all permission categories with sensitivity levels:
- 🚨 **System** (Critical)
- 🔐 **Security & Secrets** (Critical) 
- 👥 **User Management** (High)
- 🤖 **Agent Management** (Medium)
- 💰 **Subscription Management** (High)
- 📋 **Audit & Compliance** (Medium)

## 🔍 **Audit & Monitoring**

### **📜 View Role Change Audit Trail**

```bash
# Get all role changes
GET /api/admin/roles/audit

# Get audit trail for specific user
GET /api/admin/roles/audit/users/{user_id}

# Limit results
GET /api/admin/roles/audit?limit=50
```

**Audit Trail Example:**
```json
{
  "data": [
    {
      "audit_id": "uuid",
      "action": "role_assigned",
      "user_id": "user-uuid", 
      "role_id": "role-uuid",
      "performed_by": "admin-uuid",
      "performed_at": "2024-01-15T10:30:00Z",
      "reason": "User promoted to security analyst",
      "ip_address": "192.168.1.100"
    },
    {
      "action": "permission_added",
      "role_id": "role-uuid",
      "permission_id": "permission-uuid", 
      "performed_by": "admin-uuid",
      "reason": "Added secret access to security role"
    }
  ]
}
```

## 🚨 **Security Best Practices**

### **🔒 Secret Access Control**

**Who Should Have Secret Access:**
- ✅ **System Administrators** - Full secret management
- ✅ **Security Analysts** - Read access for incident response
- ❌ **Regular Users** - No access to secrets
- ❌ **Guests/Read-Only** - Definitely no access

**Secret Permissions:**
- **`secrets.read`** - View API keys and secrets
- **`secrets.create`** - Generate new API keys  
- **`secrets.update`** - Rotate/modify secrets
- **`secrets.delete`** - Remove secrets (most dangerous)

### **👥 User Management Access**

**Who Should Manage Users:**
- ✅ **System Administrators** - Full user management
- ✅ **Admins** - Create/update users and assign basic roles
- ⚠️ **Managers** - Limited to their team members
- ❌ **Regular Users** - No user management access

**User Management Permissions:**
- **`users.create`** - Create new user accounts
- **`users.read`** - View user profiles and information
- **`users.update`** - Modify user profiles and settings
- **`users.delete`** - Delete user accounts
- **`users.roles`** - Assign/remove roles (most sensitive)

### **🏛️ System Administration**

**Who Should Have System Admin:**
- ✅ **System Administrators** only
- ❌ **Everyone else** - Even security analysts shouldn't have full system access

**System Admin Permissions:**
- **`system.admin`** - Full system control
- **`system.config`** - Modify system configuration  
- **`system.maintenance`** - Perform maintenance operations

## 🎯 **Common Admin Scenarios**

### **Scenario 1: New Employee Onboarding**

```bash
# 1. Create user account (done through user management)
# 2. Assign appropriate role
POST /api/admin/roles/users/{new_user_id}/roles
{
  "role_id": "user-role-uuid",
  "context": "New employee - standard user access"
}

# 3. Set as primary role
PUT /api/admin/roles/users/{new_user_id}/primary-role
{
  "role_id": "user-role-uuid"
}
```

### **Scenario 2: Promoting User to Security Analyst**

```bash
# 1. Assign security analyst role
POST /api/admin/roles/users/{user_id}/roles
{
  "role_id": "security-analyst-role-uuid",
  "context": "Promoted to security analyst role"
}

# 2. Make it their primary role
PUT /api/admin/roles/users/{user_id}/primary-role
{
  "role_id": "security-analyst-role-uuid"
}

# 3. Verify they now have secret access
GET /api/admin/roles/users/{user_id}/secret-access
```

### **Scenario 3: Temporary Admin Access**

```bash
# Assign admin role with expiration
POST /api/admin/roles/users/{user_id}/roles
{
  "role_id": "admin-role-uuid",
  "expires_at": "2024-02-01T00:00:00Z",
  "context": "Temporary admin access for project setup"
}
```

### **Scenario 4: Emergency Secret Access Revocation**

```bash
# 1. Find all users with secret access
GET /api/admin/roles/users?filter=secrets

# 2. Remove specific user's access by removing role
DELETE /api/admin/roles/users/{user_id}/roles/{security_analyst_role_id}?reason=Security incident - access revoked

# 3. Verify access removed
GET /api/admin/roles/users/{user_id}/secret-access
```

### **Scenario 5: Auditing Who Can Access Secrets**

```bash
# Get all users with sensitive access
GET /api/admin/roles/users/sensitive-access

# Review audit trail for secret-related changes
GET /api/admin/roles/audit?limit=100
```

## 🛡️ **Security Implementation**

### **Row Level Security (RLS)**

The system implements PostgreSQL Row Level Security to ensure:
- ✅ Users can only see their own data
- ✅ Admins with proper permissions can see what they're authorized for
- ✅ Cross-user data access is automatically blocked
- ✅ All access attempts are logged

### **Permission Validation**

Every API request is validated:
```rust
// Example: Protecting secret access endpoints
#[route("/api/secrets")]
async fn get_secrets(
    _: RequireSecretAccess, // Automatically checks secrets.read permission
    auth_user: AuthUser,
) -> Result<Json<SecretsResponse>> {
    // Only executes if user has secret access
}

// Example: Protecting user management
#[route("/api/admin/users")]
async fn manage_users(
    _: RequirePermission("users.roles"), // Requires specific permission
    auth_user: AuthUser,
) -> Result<Json<UsersResponse>> {
    // Only executes if user can manage roles
}
```

### **Audit Logging**

All role and permission changes are automatically logged:
- ✅ **Who** made the change
- ✅ **What** was changed
- ✅ **When** it happened
- ✅ **Why** (if reason provided)
- ✅ **Where** (IP address)

## 🚀 **Advanced Features**

### **Role Inheritance**

Roles have hierarchy levels - higher-level roles automatically inherit permissions from lower levels:
- **System Admin (100)** inherits all permissions
- **Admin (70)** inherits User (10) permissions  
- **Manager (50)** inherits User (10) permissions

### **Permission Expiration**

- ✅ Set expiration dates on role assignments
- ✅ Automatic cleanup of expired permissions
- ✅ Notifications before expiration

### **Session-Based Role Switching**

Users with multiple roles can switch between them:
- ✅ Active role stored in session
- ✅ Permissions based on current active role
- ✅ Audit trail of role switches

## 📊 **System Health Monitoring**

```bash
# Check role system health
GET /api/admin/roles/health

# Response:
{
  "status": "healthy",
  "metrics": {
    "active_roles": 8,
    "active_permissions": 45,
    "active_user_roles": 150,
    "users_with_roles": 75,
    "users_with_sensitive_access": 12
  }
}
```

## 🎯 **Summary**

The SecureGuard RBAC system gives you complete control over:

✅ **Who can access secrets** - Granular control over API keys and sensitive data
✅ **Who can manage users** - Control over user creation, modification, and role assignment  
✅ **Who can admin the system** - Strict control over system-level operations
✅ **Audit trail** - Complete visibility into all permission changes
✅ **Flexible role assignment** - Multiple roles per user with expiration support
✅ **Security enforcement** - Automatic permission validation and access logging

**You now have enterprise-grade access control that ensures only authorized users can access sensitive areas of your SecureGuard platform!** 🚀

## 🔧 **Quick Reference Commands**

```bash
# View all users and their access levels
GET /api/admin/roles/users

# Find users with secret access  
GET /api/admin/roles/users?filter=secrets

# Assign role to user
POST /api/admin/roles/users/{user_id}/roles

# Remove role from user
DELETE /api/admin/roles/users/{user_id}/roles/{role_id}

# Check if user can access secrets
GET /api/admin/roles/users/{user_id}/secret-access

# View audit trail
GET /api/admin/roles/audit

# Check system health
GET /api/admin/roles/health
```