# SecureGuard API Documentation

**Document Version:** 2.0  
**Last Updated:** August 19, 2025  
**Status:** Production-Ready  
**Author:** SecureGuard Development Team

## Overview

SecureGuard provides a comprehensive REST API built with Rust and Axum for managing cybersecurity operations, agent management, and user authentication. This document covers all available endpoints with their request/response formats and authentication requirements.

## Base URL

- **Development**: `http://localhost:3000/api/v1`
- **Production**: `https://your-domain.com/api/v1`

## Authentication

All protected endpoints require JWT authentication via the `Authorization` header:

```http
Authorization: Bearer <your-jwt-token>
```

### Authentication Flow

1. Register or login to obtain JWT token
2. Include token in subsequent requests
3. Token expires after configured period
4. Refresh token as needed

## 🔐 Authentication Endpoints

### User Registration

Register a new user account.

```http
POST /auth/register
Content-Type: application/json

{
  "username": "john_doe",
  "email": "john@example.com",
  "password": "SecurePassword123!",
  "full_name": "John Doe"
}
```

**Response (201 Created):**
```json
{
  "success": true,
  "user": {
    "user_id": "uuid",
    "username": "john_doe",
    "email": "john@example.com",
    "full_name": "John Doe",
    "role": "user",
    "is_active": true,
    "must_change_password": false
  },
  "token": "jwt-token-string"
}
```

### User Login

Authenticate existing user and obtain JWT token.

```http
POST /auth/login
Content-Type: application/json

{
  "email": "john@example.com",
  "password": "SecurePassword123!"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "user": {
    "user_id": "uuid",
    "username": "john_doe",
    "email": "john@example.com",
    "full_name": "John Doe",
    "role": "user",
    "is_active": true,
    "must_change_password": false
  },
  "token": "jwt-token-string"
}
```

**Login Security Features:**
- Account lockout after 5 failed attempts (30-minute lockout)
- Failed login attempt tracking
- Progressive lockout duration
- Automatic lockout reset on successful login

### Get Current User

Retrieve authenticated user information.

```http
GET /auth/me
Authorization: Bearer <jwt-token>
```

**Response (200 OK):**
```json
{
  "user": {
    "user_id": "uuid",
    "username": "john_doe",
    "email": "john@example.com",
    "full_name": "John Doe",
    "role": "user",
    "is_active": true,
    "must_change_password": false
  },
  "must_change_password": false
}
```

### Change Password (NEW)

Change user password with validation and policy enforcement.

```http
POST /auth/change-password
Authorization: Bearer <jwt-token>
Content-Type: application/json

{
  "old_password": "CurrentPassword123!",
  "new_password": "NewSecurePassword456!"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "message": "Password changed successfully"
}
```

**Password Requirements:**
- Minimum 12 characters
- At least one uppercase letter
- At least one lowercase letter
- At least one number
- At least one special character
- Cannot reuse last 5 passwords
- Must be different from current password

**Error Responses:**
- `400 Bad Request`: Password doesn't meet policy requirements
- `400 Bad Request`: New password matches old password
- `400 Bad Request`: Password found in history
- `401 Unauthorized`: Current password incorrect

### Get Password Policy (NEW)

Retrieve current password policy settings.

```http
GET /auth/password-policy
```

**Response (200 OK):**
```json
{
  "policy": {
    "min_length": 12,
    "require_uppercase": true,
    "require_lowercase": true,
    "require_numbers": true,
    "require_special_chars": true,
    "max_age_days": 90,
    "history_count": 5,
    "max_failed_attempts": 5,
    "lockout_duration_minutes": 30
  }
}
```

### Check Password Change Requirement (NEW)

Check if user must change password before continuing.

```http
GET /auth/must-change-password
Authorization: Bearer <jwt-token>
```

**Response (200 OK):**
```json
{
  "must_change": false,
  "reason": null
}
```

### Request Password Reset (NEW)

Request a password reset token to be sent to the user's email.

```http
POST /auth/password-reset/request
Content-Type: application/json

{
  "email": "john@example.com"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "message": "Password reset instructions sent to your email"
}
```

**Notes:**
- Always returns success for security (prevents email enumeration)
- Token expires after 1 hour
- Email contains reset link with token
- Previous tokens are invalidated

### Confirm Password Reset (NEW)

Reset password using the token received via email.

```http
POST /auth/password-reset/confirm
Content-Type: application/json

{
  "token": "reset-token-from-email",
  "new_password": "NewSecurePassword789!"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "message": "Password reset successfully"
}
```

**Error Responses:**
- `400 Bad Request`: Invalid or expired token
- `400 Bad Request`: Password doesn't meet policy requirements
- `400 Bad Request`: Token already used

**Response (200 OK):**
```json
{
  "must_change_password": true,
  "reason": "Password change required on first login"
}
```

## 🖥️ Agent Management Endpoints

### Register Agent

Register a new agent endpoint.

```http
POST /agents/register
Content-Type: application/json

{
  "tenant_id": "uuid",
  "hostname": "workstation-01",
  "ip_address": "192.168.1.100",
  "mac_address": "00:11:22:33:44:55",
  "os_info": "Windows 11 Pro",
  "version": "1.0.0"
}
```

**Response (201 Created):**
```json
{
  "success": true,
  "agent": {
    "agent_id": "uuid",
    "tenant_id": "uuid",
    "hostname": "workstation-01",
    "ip_address": "192.168.1.100",
    "status": "active",
    "last_seen": "2025-08-19T10:30:00Z",
    "version": "1.0.0"
  }
}
```

### Agent Heartbeat

Update agent status and health information.

```http
POST /agents/heartbeat
Content-Type: application/json

{
  "agent_id": "uuid",
  "status": "active",
  "system_info": {
    "cpu_usage": 45.2,
    "memory_usage": 68.5,
    "disk_usage": 82.1,
    "uptime": 86400
  },
  "threats_detected": 0
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "message": "Heartbeat received"
}
```

### List Agents

Get list of all registered agents for the authenticated user.

```http
GET /agents
Authorization: Bearer <jwt-token>
```

**Response (200 OK):**
```json
{
  "agents": [
    {
      "agent_id": "uuid",
      "hostname": "workstation-01",
      "ip_address": "192.168.1.100",
      "status": "active",
      "last_seen": "2025-08-19T10:30:00Z",
      "version": "1.0.0",
      "system_info": {
        "cpu_usage": 45.2,
        "memory_usage": 68.5,
        "disk_usage": 82.1
      }
    }
  ]
}
```

### Get Agent Details

Get detailed information about a specific agent.

```http
GET /agents/{agent_id}
Authorization: Bearer <jwt-token>
```

**Response (200 OK):**
```json
{
  "agent": {
    "agent_id": "uuid",
    "hostname": "workstation-01",
    "ip_address": "192.168.1.100",
    "mac_address": "00:11:22:33:44:55",
    "os_info": "Windows 11 Pro",
    "status": "active",
    "last_seen": "2025-08-19T10:30:00Z",
    "version": "1.0.0",
    "created_at": "2025-08-01T09:00:00Z",
    "system_info": {
      "cpu_usage": 45.2,
      "memory_usage": 68.5,
      "disk_usage": 82.1,
      "uptime": 86400
    }
  }
}
```

## 🚨 Threat Management Endpoints

### List Threats

Get list of detected threats.

```http
GET /threats
Authorization: Bearer <jwt-token>
```

**Query Parameters:**
- `severity` (optional): Filter by severity level (low, medium, high, critical)
- `status` (optional): Filter by status (new, investigating, resolved, false_positive)
- `limit` (optional): Number of results to return (default: 50)
- `offset` (optional): Pagination offset (default: 0)

**Response (200 OK):**
```json
{
  "threats": [
    {
      "threat_id": "uuid",
      "agent_id": "uuid",
      "threat_type": "malware_detection",
      "severity": "high",
      "status": "new",
      "description": "Suspicious executable detected",
      "file_path": "C:\\Users\\Downloads\\suspicious.exe",
      "detected_at": "2025-08-19T10:30:00Z",
      "hash": "sha256-hash"
    }
  ],
  "total": 1,
  "limit": 50,
  "offset": 0
}
```

### Get Threat Details

Get detailed information about a specific threat.

```http
GET /threats/{threat_id}
Authorization: Bearer <jwt-token>
```

**Response (200 OK):**
```json
{
  "threat": {
    "threat_id": "uuid",
    "agent_id": "uuid",
    "threat_type": "malware_detection",
    "severity": "high",
    "status": "new",
    "description": "Suspicious executable detected",
    "file_path": "C:\\Users\\Downloads\\suspicious.exe",
    "detected_at": "2025-08-19T10:30:00Z",
    "hash": "sha256-hash",
    "metadata": {
      "file_size": 1024000,
      "file_type": "executable",
      "detection_engine": "signature_scan"
    }
  }
}
```

## 🏥 System Health Endpoints

## 📊 Telemetry & Observability Endpoints

### Prometheus Metrics

Export application metrics in Prometheus format.

```http
GET /metrics
```

**Response (200 OK):**
```text
# HELP api_requests Total number of API requests
# TYPE api_requests counter
api_requests{method="GET",endpoint="/api/v1/agents"} 1234
api_requests{method="POST",endpoint="/api/v1/auth/login"} 567

# HELP api_duration API request duration in seconds
# TYPE api_duration histogram
api_duration_bucket{method="GET",endpoint="/api/v1/agents",le="0.1"} 1200
api_duration_bucket{method="GET",endpoint="/api/v1/agents",le="0.5"} 1230

# HELP agents_active Number of active agents
# TYPE agents_active gauge
agents_active 142

# Additional metrics...
```

**Available Metrics:**
- `api_requests`: Total API requests by method and endpoint
- `api_errors`: Total API errors by method, endpoint, and status
- `api_duration`: Request duration histogram
- `agents_active`: Current number of active agents
- `security_events`: Total security events processed
- `websocket_connections`: Active WebSocket connections
- `database_queries`: Total database queries
- `database_query_duration`: Database query duration

## 📋 Logging & Audit Trail

SecureGuard implements comprehensive logging with daily file rotation and structured JSON format for production monitoring and compliance.

### Log File Structure

```
./logs/
├── secureguard-api.log.YYYY-MM-DD     # General application logs (JSON format)
├── security-audit.log.YYYY-MM-DD      # Security events & audit trail
└── error.log.YYYY-MM-DD               # Error-only logs for incidents
```

### Security Events Logged

**Authentication Events:**
- User login attempts (success/failure)
- Password verification events
- JWT token generation and validation
- Account lockout events
- Password change attempts

**API Key Operations:**
- API key creation and deletion
- API key validation attempts (success/failure)
- API key usage and expiration events
- Invalid API key usage attempts

**Agent Management:**
- Agent registration events
- Agent authentication attempts
- Device limit violations
- Agent communication errors

**Example Security Audit Entry:**
```json
{
  "timestamp": "2024-08-19T10:30:00.123456Z",
  "level": "WARN",
  "target": "secureguard_api",
  "security": "login_attempt",
  "audit": "failed_login",
  "fields": {
    "event": "login_failed",
    "username": "user123",
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "reason": "invalid_password",
    "failed_attempts": 3,
    "status": "failed"
  },
  "message": "Login attempt failed - invalid password"
}
```

### Log Access (Admin Only)

While log files are not exposed via API endpoints for security reasons, administrators can:

1. **Direct File Access**: Access log files directly on the server filesystem
2. **Log Aggregation**: Integrate with ELK stack, Grafana Loki, or similar tools
3. **Monitoring Tools**: Use structured JSON format for automated analysis
4. **Alerting**: Set up alerts based on log patterns and security events

### Operational Benefits

- **Security Compliance**: Complete audit trail for SOC 2, ISO 27001
- **Incident Response**: Focused error logs for rapid troubleshooting  
- **Performance Monitoring**: Request timing and resource usage tracking
- **Threat Detection**: Pattern analysis for security events
- **Compliance Reporting**: Structured data for automated compliance reports

📖 **See [Logging & Monitoring Guide](Logging_Monitoring_Guide.md) for detailed log analysis and configuration.**

### Health Check

Basic system health check endpoint.

```http
GET /health
```

**Response (200 OK):**
```json
{
  "status": "healthy",
  "timestamp": "2025-08-19T10:30:00Z",
  "version": "1.0.0",
  "database": "connected",
  "redis": "connected"
}
```

### Detailed Health Status

Comprehensive system health information (admin only).

```http
GET /health/detailed
Authorization: Bearer <admin-jwt-token>
```

**Response (200 OK):**
```json
{
  "status": "healthy",
  "timestamp": "2025-08-19T10:30:00Z",
  "version": "1.0.0",
  "services": {
    "database": {
      "status": "connected",
      "connections": 10,
      "response_time_ms": 2.5
    },
    "redis": {
      "status": "connected",
      "memory_usage": "45MB",
      "response_time_ms": 1.2
    }
  },
  "metrics": {
    "total_agents": 150,
    "active_agents": 142,
    "total_threats": 25,
    "new_threats": 3
  }
}
```

## ⚠️ Error Responses

### Standard Error Format

All endpoints return errors in the following format:

```json
{
  "error": "Error message description",
  "error_code": "SPECIFIC_ERROR_CODE",
  "details": {
    "field": "Additional error details"
  }
}
```

### Common HTTP Status Codes

- `200 OK`: Request successful
- `201 Created`: Resource created successfully
- `400 Bad Request`: Invalid request data
- `401 Unauthorized`: Authentication required or invalid
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Resource not found
- `409 Conflict`: Resource conflict (e.g., duplicate)
- `422 Unprocessable Entity`: Validation errors
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error

### Password Security Error Codes

- `PASSWORD_TOO_SHORT`: Password below minimum length
- `PASSWORD_MISSING_UPPERCASE`: No uppercase characters
- `PASSWORD_MISSING_LOWERCASE`: No lowercase characters
- `PASSWORD_MISSING_NUMBERS`: No numeric characters
- `PASSWORD_MISSING_SPECIAL`: No special characters
- `PASSWORD_IN_HISTORY`: Password found in history
- `PASSWORD_SAME_AS_CURRENT`: New password same as current
- `ACCOUNT_LOCKED`: Account locked due to failed attempts
- `PASSWORD_CHANGE_REQUIRED`: Must change password to continue

### Authentication Error Codes

- `INVALID_CREDENTIALS`: Invalid email/password combination
- `ACCOUNT_DISABLED`: User account is disabled
- `ACCOUNT_LOCKED`: Account locked due to security policy
- `TOKEN_EXPIRED`: JWT token has expired
- `TOKEN_INVALID`: JWT token is invalid or malformed
- `INSUFFICIENT_PRIVILEGES`: User lacks required permissions

## 🔒 Security Considerations

### Rate Limiting

- **Login attempts**: 5 attempts per 15 minutes per IP
- **API requests**: 1000 requests per hour per user
- **Agent heartbeats**: No limit (essential for monitoring)

### Request Validation

- All input data is validated and sanitized
- SQL injection protection via parameterized queries
- XSS protection in all user inputs
- CSRF protection for state-changing operations

### Password Security

- Passwords hashed with Argon2 algorithm
- Password history tracked (last 5 passwords)
- Account lockout after failed attempts
- Secure password policy enforcement

### Data Privacy

- Personal data encrypted at rest
- GDPR compliance for data handling
- Audit logs for all data access
- Right to data deletion supported

## 📊 API Versioning

The API uses URL-based versioning:

- Current version: `v1` (stable)
- Future versions will be: `v2`, `v3`, etc.
- Backward compatibility maintained for 1 year minimum

## 📞 Support

For API support and integration assistance:

- **Documentation**: [docs/README.md](README.md)
- **GitHub Issues**: Submit bug reports and feature requests
- **Enterprise Support**: Available for production deployments

---

**Next Update**: After WebSocket API implementation and advanced threat detection features

## 📖 Related Documentation

- [API Key Agent Registration](API_Key_Agent_Registration.md)
- [Frontend Security Guide](Frontend_Security_Guide.md)
- [Development Setup Guide](Development_Setup_Guide.md)
- [Admin Interface Guide](Admin_Interface_Guide.md)