# SecureGuard Workflow Testing Guide

## Overview

This guide covers the comprehensive workflow testing system for SecureGuard, including role-based access control testing, user authentication workflows, and end-to-end application functionality verification.

## Test Architecture

### Test Structure
```
crates/secureguard-api/tests/
├── workflow_tests.rs           # Main workflow test suite
├── test_data_setup.rs         # Test data management utilities  
├── auth_tests.rs              # Existing authentication tests
├── integration_tests.rs       # Existing integration tests
└── security_monitoring_test.rs.disabled
```

### Test Runners
```
scripts/
├── run_workflow_tests.bat     # Windows batch script
└── run_workflow_tests.ps1     # PowerShell script (recommended)
```

## User Roles and Permissions

### Role Hierarchy (High to Low)
1. **SystemAdmin** (Level 100) - Complete system control
2. **SecurityAnalyst** (Level 80) - Security monitoring and incident response  
3. **Admin** (Level 70) - User and system management
4. **Manager** (Level 50) - Management oversight
5. **PowerUser** (Level 30) - Advanced user capabilities
6. **User** (Level 10) - Standard access
7. **ReadOnly** (Level 5) - View-only access
8. **Guest** (Level 1) - Minimal access

### Permission Categories
- **System**: `system.admin`, `system.config`, `system.maintenance`
- **Users**: `users.create`, `users.read`, `users.update`, `users.delete`, `users.roles`
- **Secrets**: `secrets.read`, `secrets.create`, `secrets.update`, `secrets.delete`
- **Agents**: `agents.read`, `agents.create`, `agents.update`, `agents.delete`, `agents.control`
- **Security**: `security.incidents`, `security.monitoring`, `security.response`
- **Subscriptions**: `subscriptions.*`
- **Audit**: `audit.read`, `audit.export`
- **API**: `api.read`, `api.write`, `api.admin`

## Test Categories

### 1. Authentication Workflow Tests

**Purpose**: Verify login, token generation, and session management across all roles.

**Key Tests**:
- `test_multi_role_authentication_workflow()` - Tests login for all role types
- Token validation and expiration
- Credential verification

**What's Tested**:
- ✅ User can login with correct credentials
- ✅ Login fails with incorrect credentials  
- ✅ JWT tokens are generated correctly
- ✅ Token validation works properly
- ✅ Role assignment is correct after login

### 2. Security Analyst Workflow Tests

**Purpose**: Comprehensive testing of security analyst daily operations.

**Key Tests**:
- `test_security_analyst_full_workflow()` - Complete analyst permissions check
- `test_analyst_daily_tasks_workflow()` - Daily operational tasks

**Security Analyst Can**:
- ✅ Login with test credentials
- ✅ Monitor security incidents (`security.incidents`)
- ✅ View and respond to security alerts (`security.response`)
- ✅ Monitor agent health (`agents.read`, `agents.update`)
- ✅ Access security monitoring dashboard (`security.monitoring`)
- ✅ View audit logs (`audit.read`)
- ✅ Access necessary secrets for security work (`secrets.read`)

**Security Analyst Cannot**:
- ❌ Create or delete users (`users.create`, `users.delete`)
- ❌ Access system administration (`system.admin`)
- ❌ Manage system configuration (`system.config`)

### 3. Admin Role Workflow Tests

**Purpose**: Verify administrative capabilities for user and resource management.

**Key Tests**:
- `test_admin_user_management_workflow()` - User management operations

**Admin Can**:
- ✅ Create, read, update, delete users
- ✅ Assign roles to users (`users.roles`)
- ✅ Manage agents (create, update, delete)
- ✅ Manage subscriptions
- ✅ View audit logs

**Admin Cannot**:
- ❌ Access system-level administration (reserved for SystemAdmin)
- ❌ Perform system maintenance operations

### 4. System Admin Workflow Tests  

**Purpose**: Verify complete system access and control capabilities.

**Key Tests**:
- `test_system_admin_full_access_workflow()` - Complete system access verification

**System Admin Can**:
- ✅ All user management operations
- ✅ All agent management operations
- ✅ All system administration (`system.admin`)
- ✅ System configuration (`system.config`)
- ✅ System maintenance (`system.maintenance`)
- ✅ All secrets operations
- ✅ All API administration

### 5. Regular User Workflow Tests

**Purpose**: Verify limited access for standard users.

**Key Tests**:
- `test_regular_user_limited_workflow()` - Standard user permissions
- `test_readonly_user_view_only_workflow()` - Read-only user permissions

**Regular User Can**:
- ✅ View agents they own (`agents.read`)
- ✅ Basic API access (`api.read`)

**Regular User Cannot**:
- ❌ Create or manage other users
- ❌ Access system administration
- ❌ Access secrets
- ❌ Delete or control agents

### 6. End-to-End Workflow Tests

**Purpose**: Comprehensive application functionality testing across all roles.

**Key Tests**:
- `test_complete_application_workflow()` - Multi-role scenario testing
- `test_role_hierarchy_enforcement()` - Role hierarchy validation

**Scenarios Tested**:
1. **System Setup**: SystemAdmin configures the system
2. **User Management**: Admin creates users and assigns roles  
3. **Security Monitoring**: SecurityAnalyst monitors threats and agents
4. **Daily Operations**: Regular users perform limited operations
5. **Boundary Enforcement**: Cross-role permission boundaries are maintained

### 7. Error Scenario and Edge Case Tests

**Purpose**: Verify proper handling of invalid operations and edge cases.

**Key Tests**:
- `test_invalid_credentials_and_permissions()` - Invalid login attempts
- Non-existent user permission checks
- Invalid token handling

## Running the Tests

### Prerequisites
1. **Database**: PostgreSQL running with `secureguard_dev` database
2. **Environment**: `DATABASE_URL_TEST` environment variable set
3. **Dependencies**: All Rust dependencies installed (`cargo build`)

### Quick Test Run
```bash
# Windows Command Prompt
scripts\run_workflow_tests.bat

# PowerShell (Recommended)
scripts\run_workflow_tests.ps1

# Manual cargo test
cargo test --package secureguard-api --test workflow_tests --nocapture
```

### Individual Test Categories
```bash
# Authentication tests only
cargo test --package secureguard-api --test workflow_tests test_multi_role_authentication_workflow

# Security analyst tests only  
cargo test --package secureguard-api --test workflow_tests test_security_analyst_full_workflow

# Admin tests only
cargo test --package secureguard-api --test workflow_tests test_admin_user_management_workflow

# All workflow tests
cargo test --package secureguard-api --test workflow_tests
```

## Test Data Management

### TestSetup Helper
The `TestSetup` struct in `workflow_tests.rs` provides:
- Database connection management
- Test user creation with roles
- Authentication token generation
- Role assignment automation

### TestDataSetup Utility
The `TestDataSetup` in `test_data_setup.rs` provides:
- Role and permission setup in database
- Test tenant and agent creation
- Complete test environment setup
- Data cleanup utilities

### Test Isolation
- Each test creates unique users (e.g., `sec_analyst`, `admin_user`)
- Test data uses recognizable prefixes (`test_`, email `@test.com`)
- Cleanup functions available for test data removal

## Expected Test Results

### Successful Test Run Output
```
🎉 ALL WORKFLOW TESTS PASSED!

Test Coverage Summary:
- ✅ Authentication and login flows
- ✅ Security Analyst role workflows (login, daily tasks, monitoring)
- ✅ Admin role workflows (user management, agent control)
- ✅ System Admin role workflows (full system access)
- ✅ Regular User role workflows (limited access)
- ✅ Read-only User role workflows (view-only)
- ✅ Role hierarchy enforcement
- ✅ End-to-end application workflows
- ✅ Error scenarios and edge cases
- ✅ Test data setup and teardown

🛡️ Your SecureGuard application is ready for production!
```

## Integration with CI/CD

### GitHub Actions Integration
Add to your `.github/workflows/test.yml`:

```yaml
- name: Run Workflow Tests
  run: |
    $env:DATABASE_URL_TEST = "postgresql://secureguard:password@localhost:5432/secureguard_test"
    scripts/run_workflow_tests.ps1
  shell: pwsh
```

### Environment Variables
Required for testing:
- `DATABASE_URL_TEST`: Test database connection string
- `RUST_LOG`: Set to `info` or `debug` for detailed logging

## Troubleshooting

### Common Issues

1. **Database Connection Failed**
   - Ensure PostgreSQL is running
   - Check database exists: `secureguard_dev`
   - Verify connection string in `DATABASE_URL_TEST`

2. **Permission Errors**
   - Run test data setup: `cargo test --test test_data_setup`
   - Check database schema is current: `sqlx migrate run`

3. **Test User Creation Failed**
   - Ensure unique usernames in tests
   - Check for database constraints
   - Verify role tables exist

4. **Token Validation Failed**  
   - Check JWT secret key consistency
   - Verify token expiration settings
   - Ensure auth service is properly initialized

### Debug Mode
Run tests with detailed logging:
```bash
RUST_LOG=debug cargo test --package secureguard-api --test workflow_tests --nocapture
```

## Extending the Tests

### Adding New Role Tests
1. Create new test function following naming pattern
2. Use `TestSetup::create_test_user()` with new role
3. Verify role-specific permissions with `get_user_permissions()`
4. Test both allowed and forbidden operations

### Adding New Workflow Scenarios
1. Create realistic multi-user scenarios
2. Test cross-role interactions
3. Verify permission boundaries
4. Include cleanup in tests

### Performance Testing
Consider adding:
- Concurrent user login tests
- Large dataset permission checks
- Role switching performance tests
- Database query optimization verification

## Security Considerations

### Test Security
- Test credentials are hardcoded for predictability
- Test users are clearly marked (`@test.com` emails)
- Production secrets should never be used in tests
- Test database should be isolated from production

### Permission Testing
- Tests verify both positive and negative cases
- Role boundaries are strictly enforced
- Escalation attempts are detected and blocked
- Audit trails for test operations are maintained

---

*This testing framework ensures your SecureGuard application maintains proper security boundaries and role-based access control across all user workflows.*