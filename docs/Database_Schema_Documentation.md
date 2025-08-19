# SecureGuard Database Schema Documentation

**Document Version:** 2.0  
**Last Updated:** August 19, 2025  
**Status:** Production-Ready  
**Author:** SecureGuard Development Team

## Overview

SecureGuard uses PostgreSQL as its primary database with a comprehensive schema designed for cybersecurity monitoring, user management, and threat detection. This document details all database tables, relationships, migrations, and security features.

## Database Architecture

### Schema Organization

SecureGuard organizes data into logical schemas:

- **`users`** - User management, authentication, and security
- **`agents`** - Agent endpoints and monitoring data
- **`threats`** - Threat detection and incident management
- **`audit`** - Security audit logs and compliance tracking

## 📋 Migration History

### Migration Overview

| Migration | Description | Date | Status |
|-----------|-------------|------|---------|
| 001 | Initial schema creation | 2025-08-01 | ✅ Applied |
| 002 | Threats schema | 2025-08-05 | ✅ Applied |
| 003 | User-agent linking | 2025-08-08 | ✅ Applied |
| 004 | Subscription system | 2025-08-10 | ✅ Applied |
| 005 | Remote commands | 2025-08-12 | ✅ Applied |
| 006 | Security monitoring | 2025-08-15 | ✅ Applied |
| 007 | Role permission system | 2025-08-16 | ✅ Applied |
| 008 | Password security system | 2025-08-19 | ✅ Applied |

### Latest Migration: 008 - Password Security System

Migration 008 implements comprehensive password security features:

```sql
-- Enhanced users table with security columns
ALTER TABLE users.users 
ADD COLUMN IF NOT EXISTS must_change_password BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS password_last_changed TIMESTAMPTZ DEFAULT now(),
ADD COLUMN IF NOT EXISTS failed_login_attempts INTEGER DEFAULT 0,
ADD COLUMN IF NOT EXISTS last_failed_login TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS account_locked_until TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS role VARCHAR(50) NOT NULL DEFAULT 'user',
ADD COLUMN IF NOT EXISTS password_history JSONB DEFAULT '[]'::jsonb;

-- Password policy settings table
CREATE TABLE IF NOT EXISTS users.password_policies (
    policy_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    min_length INTEGER NOT NULL DEFAULT 12,
    require_uppercase BOOLEAN NOT NULL DEFAULT TRUE,
    require_lowercase BOOLEAN NOT NULL DEFAULT TRUE,
    require_numbers BOOLEAN NOT NULL DEFAULT TRUE,
    require_special_chars BOOLEAN NOT NULL DEFAULT TRUE,
    max_age_days INTEGER NOT NULL DEFAULT 90,
    history_count INTEGER NOT NULL DEFAULT 5,
    max_failed_attempts INTEGER NOT NULL DEFAULT 5,
    lockout_duration_minutes INTEGER NOT NULL DEFAULT 30,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

## 👥 Users Schema

### users.users Table

Core user management and authentication table.

```sql
CREATE TABLE users.users (
    user_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    full_name VARCHAR(255),
    
    -- Security columns (Migration 008)
    must_change_password BOOLEAN NOT NULL DEFAULT FALSE,
    password_last_changed TIMESTAMPTZ DEFAULT now(),
    failed_login_attempts INTEGER DEFAULT 0,
    last_failed_login TIMESTAMPTZ,
    account_locked_until TIMESTAMPTZ,
    password_history JSONB DEFAULT '[]'::jsonb,
    
    -- User management
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    tenant_id UUID,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login TIMESTAMPTZ
);
```

#### Security Features

**Password Security**:
- `must_change_password`: Forces password change on next login
- `password_last_changed`: Tracks password age for expiration policies
- `password_history`: JSONB array of previous password hashes (prevents reuse)

**Account Lockout**:
- `failed_login_attempts`: Counter for consecutive failed logins
- `last_failed_login`: Timestamp of most recent failed attempt
- `account_locked_until`: Lockout expiration timestamp

**Roles**: `user`, `manager`, `admin`, `system_admin`

### users.password_policies Table

Configurable password policy settings.

```sql
CREATE TABLE users.password_policies (
    policy_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Password requirements
    min_length INTEGER NOT NULL DEFAULT 12,
    require_uppercase BOOLEAN NOT NULL DEFAULT TRUE,
    require_lowercase BOOLEAN NOT NULL DEFAULT TRUE,
    require_numbers BOOLEAN NOT NULL DEFAULT TRUE,
    require_special_chars BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Policy enforcement
    max_age_days INTEGER NOT NULL DEFAULT 90,
    history_count INTEGER NOT NULL DEFAULT 5,
    max_failed_attempts INTEGER NOT NULL DEFAULT 5,
    lockout_duration_minutes INTEGER NOT NULL DEFAULT 30,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

#### Default Policy Values

- **Minimum Length**: 12 characters
- **Character Requirements**: All enabled (uppercase, lowercase, numbers, special)
- **Password Age**: 90 days maximum
- **History**: Remembers last 5 passwords
- **Lockout**: 5 failed attempts = 30-minute lockout

### users.api_keys Table

API key management for agent registration.

```sql
CREATE TABLE users.api_keys (
    key_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users.users(user_id),
    key_hash TEXT NOT NULL,
    key_name VARCHAR(100) NOT NULL,
    key_prefix VARCHAR(10) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    expires_at TIMESTAMPTZ,
    last_used TIMESTAMPTZ,
    usage_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

## 🖥️ Agents Schema

### agents.endpoints Table

Registered agent endpoints and monitoring data.

```sql
CREATE TABLE agents.endpoints (
    agent_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID,
    user_id UUID REFERENCES users.users(user_id),  -- Added in Migration 003
    
    -- Agent identification
    hostname VARCHAR(255) NOT NULL,
    ip_address INET,
    mac_address MACADDR,
    device_name VARCHAR(100),  -- User-friendly name
    
    -- System information
    os_info TEXT,
    version VARCHAR(50),
    hardware_fingerprint TEXT,
    
    -- Registration tracking
    registered_via_key_id UUID REFERENCES users.api_keys(key_id),
    
    -- Status and monitoring
    status VARCHAR(20) DEFAULT 'active',
    last_seen TIMESTAMPTZ,
    heartbeat_interval INTEGER DEFAULT 300,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### agents.system_info Table

Detailed system metrics from agents.

```sql
CREATE TABLE agents.system_info (
    info_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents.endpoints(agent_id),
    
    -- Performance metrics
    cpu_usage DECIMAL(5,2),
    memory_usage DECIMAL(5,2),
    disk_usage DECIMAL(5,2),
    network_usage JSONB,
    
    -- System status
    uptime BIGINT,
    process_count INTEGER,
    service_status JSONB,
    
    -- Timestamp
    collected_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

## 🚨 Threats Schema

### threats.incidents Table

Security threat detection and incident tracking.

```sql
CREATE TABLE threats.incidents (
    threat_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents.endpoints(agent_id),
    
    -- Threat classification
    threat_type VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    status VARCHAR(20) DEFAULT 'new' CHECK (status IN ('new', 'investigating', 'resolved', 'false_positive')),
    
    -- Threat details
    title VARCHAR(255) NOT NULL,
    description TEXT,
    file_path TEXT,
    process_name VARCHAR(255),
    command_line TEXT,
    hash VARCHAR(128),
    
    -- Detection metadata
    detection_engine VARCHAR(50),
    confidence_score DECIMAL(3,2),
    false_positive_likelihood DECIMAL(3,2),
    
    -- Resolution tracking
    assigned_to UUID REFERENCES users.users(user_id),
    resolved_by UUID REFERENCES users.users(user_id),
    resolution_notes TEXT,
    
    -- Timestamps
    detected_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

## 🔧 Database Functions

### Password Security Functions

Migration 008 includes specialized database functions for password security:

#### users.validate_password_strength()

Validates password against policy requirements.

```sql
CREATE OR REPLACE FUNCTION users.validate_password_strength(
    password TEXT,
    min_length INTEGER DEFAULT 12,
    require_uppercase BOOLEAN DEFAULT TRUE,
    require_lowercase BOOLEAN DEFAULT TRUE,
    require_numbers BOOLEAN DEFAULT TRUE,
    require_special_chars BOOLEAN DEFAULT TRUE
) RETURNS BOOLEAN AS $$
BEGIN
    -- Check minimum length
    IF length(password) < min_length THEN
        RETURN FALSE;
    END IF;
    
    -- Check character requirements
    IF require_uppercase AND password !~ '[A-Z]' THEN
        RETURN FALSE;
    END IF;
    
    IF require_lowercase AND password !~ '[a-z]' THEN
        RETURN FALSE;
    END IF;
    
    IF require_numbers AND password !~ '[0-9]' THEN
        RETURN FALSE;
    END IF;
    
    IF require_special_chars AND password !~ '[^a-zA-Z0-9]' THEN
        RETURN FALSE;
    END IF;
    
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;
```

#### users.handle_failed_login()

Manages failed login attempts and account lockout.

```sql
CREATE OR REPLACE FUNCTION users.handle_failed_login(username_param TEXT) 
RETURNS VOID AS $$
DECLARE
    max_attempts INTEGER;
    lockout_duration INTEGER;
BEGIN
    -- Get policy settings
    SELECT max_failed_attempts, lockout_duration_minutes 
    INTO max_attempts, lockout_duration
    FROM users.password_policies 
    LIMIT 1;
    
    -- Update failed attempt count
    UPDATE users.users 
    SET 
        failed_login_attempts = failed_login_attempts + 1,
        last_failed_login = now()
    WHERE username = username_param;
    
    -- Lock account if max attempts reached
    UPDATE users.users 
    SET account_locked_until = now() + INTERVAL '1 minute' * lockout_duration
    WHERE username = username_param 
    AND failed_login_attempts >= max_attempts;
END;
$$ LANGUAGE plpgsql;
```

#### users.handle_successful_login()

Resets failed login counters on successful authentication.

```sql
CREATE OR REPLACE FUNCTION users.handle_successful_login(username_param TEXT) 
RETURNS VOID AS $$
BEGIN
    UPDATE users.users 
    SET 
        failed_login_attempts = 0,
        last_failed_login = NULL,
        account_locked_until = NULL,
        last_login = now()
    WHERE username = username_param;
END;
$$ LANGUAGE plpgsql;
```

## 📊 Database Indexes

### Performance Optimization Indexes

```sql
-- User authentication indexes
CREATE INDEX idx_users_username ON users.users(username);
CREATE INDEX idx_users_email ON users.users(email);
CREATE INDEX idx_users_role ON users.users(role);
CREATE INDEX idx_users_account_locked ON users.users(account_locked_until) 
    WHERE account_locked_until IS NOT NULL;

-- Agent monitoring indexes
CREATE INDEX idx_agents_tenant_id ON agents.endpoints(tenant_id);
CREATE INDEX idx_agents_user_id ON agents.endpoints(user_id);
CREATE INDEX idx_agents_status ON agents.endpoints(status);
CREATE INDEX idx_agents_last_seen ON agents.endpoints(last_seen);

-- Threat detection indexes
CREATE INDEX idx_threats_agent_id ON threats.incidents(agent_id);
CREATE INDEX idx_threats_severity ON threats.incidents(severity);
CREATE INDEX idx_threats_status ON threats.incidents(status);
CREATE INDEX idx_threats_detected_at ON threats.incidents(detected_at);

-- API key indexes
CREATE INDEX idx_api_keys_user_id ON users.api_keys(user_id);
CREATE INDEX idx_api_keys_prefix ON users.api_keys(key_prefix);
CREATE INDEX idx_api_keys_active ON users.api_keys(is_active) WHERE is_active = TRUE;
```

## 🔒 Security Considerations

### Data Protection

**Encryption at Rest**:
- Password hashes use Argon2 algorithm
- Sensitive data encrypted in production
- Database backups encrypted

**Access Control**:
- Row-level security for multi-tenant isolation
- Role-based access to sensitive tables
- Audit logging for all data modifications

**Data Retention**:
- Password history limited to configurable count
- Audit logs retained per compliance requirements
- Automatic cleanup of expired data

### Backup and Recovery

**Backup Strategy**:
- Daily full backups
- Continuous WAL archiving
- Point-in-time recovery capability
- Encrypted backup storage

**Disaster Recovery**:
- Hot standby configuration
- Cross-region backup replication
- Recovery time objective: 1 hour
- Recovery point objective: 15 minutes

## 🛠️ Maintenance Procedures

### Regular Maintenance Tasks

**Daily**:
- Monitor failed login attempts
- Check account lockout patterns
- Verify backup completion

**Weekly**:
- Update database statistics
- Review slow query logs
- Analyze threat detection patterns

**Monthly**:
- Password policy compliance review
- Index usage analysis
- Security audit log review

### Database Administration

**User Management**:
```sql
-- Create new user with secure defaults
INSERT INTO users.users (username, email, password_hash, must_change_password)
VALUES ('newuser', 'user@domain.com', '[argon2_hash]', TRUE);

-- Force password change for all users
UPDATE users.users SET must_change_password = TRUE;

-- Reset account lockouts
UPDATE users.users SET 
    failed_login_attempts = 0,
    last_failed_login = NULL,
    account_locked_until = NULL
WHERE account_locked_until IS NOT NULL;
```

**Policy Management**:
```sql
-- Update password policy
UPDATE users.password_policies SET
    min_length = 14,
    max_failed_attempts = 3,
    lockout_duration_minutes = 60;

-- View current policy
SELECT * FROM users.password_policies;
```

## 📈 Monitoring and Metrics

### Key Performance Indicators

**Security Metrics**:
- Failed login attempt rate
- Account lockout frequency
- Password change compliance rate
- Threat detection accuracy

**Performance Metrics**:
- Average query response time
- Database connection count
- Index usage efficiency
- Storage growth rate

### Alerting Thresholds

**Security Alerts**:
- Failed login rate > 10 per minute
- Account lockouts > 5 per hour
- Password policy violations
- Unauthorized database access

**Performance Alerts**:
- Query response time > 1 second
- Database connections > 80% capacity
- Storage usage > 85%
- Backup failure notifications

## 🔮 Future Schema Enhancements

### Planned Improvements

1. **Advanced Audit Logging**:
   - Detailed change tracking
   - Compliance reporting tables
   - Data lineage tracking

2. **Enhanced Threat Detection**:
   - Machine learning model storage
   - Threat correlation tables
   - Behavioral analysis data

3. **Multi-Factor Authentication**:
   - MFA configuration tables
   - TOTP secret storage
   - Backup code management

4. **Integration Support**:
   - LDAP synchronization tables
   - SSO provider configuration
   - API rate limiting data

---

**Next Update**: After advanced threat detection and compliance features implementation

## 📖 Related Documentation

- [Password Security System](Password_Security_System.md)
- [API Documentation](API_Documentation.md)
- [Development Setup Guide](Development_Setup_Guide.md)
- [Admin Interface Guide](Admin_Interface_Guide.md)