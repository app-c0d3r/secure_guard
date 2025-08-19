# Manual Database Setup (Until Build Tools Fixed)

## Current Status
- ✅ **Docker containers running**: PostgreSQL + Redis
- ❌ **sqlx-cli installation failed**: Needs Visual Studio C++ Build Tools
- ❌ **Rust compilation blocked**: Same issue

## Manual Database Verification

### Test PostgreSQL Connection
```bash
# Test connection to PostgreSQL container
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "SELECT version();"

# Expected output: PostgreSQL version info
```

### Manual Migration (Temporary)
```bash
# Copy all migration files to container
docker cp migrations/001_create_initial_schema.sql secure_guard-db-1:/tmp/
docker cp migrations/002_create_threats_schema.sql secure_guard-db-1:/tmp/
docker cp migrations/003_add_user_agent_linking.sql secure_guard-db-1:/tmp/
docker cp migrations/004_add_subscription_system.sql secure_guard-db-1:/tmp/
docker cp migrations/005_add_remote_commands_system.sql secure_guard-db-1:/tmp/
docker cp migrations/006_add_security_monitoring_system.sql secure_guard-db-1:/tmp/
docker cp migrations/007_fix_schema_mismatches.sql secure_guard-db-1:/tmp/
docker cp migrations/008_add_password_security_system.sql secure_guard-db-1:/tmp/

# Run migrations manually in order
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -f /tmp/001_create_initial_schema.sql
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -f /tmp/002_create_threats_schema.sql
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -f /tmp/003_add_user_agent_linking.sql
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -f /tmp/004_add_subscription_system.sql
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -f /tmp/005_add_remote_commands_system.sql
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -f /tmp/006_add_security_monitoring_system.sql
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -f /tmp/007_fix_schema_mismatches.sql
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -f /tmp/008_add_password_security_system.sql

# Create test database
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "CREATE DATABASE secureguard_test;"
```

### Verify Password Security Migration
```bash
# Check that password security tables were created
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "\dt users.password_policies"

# Verify password security columns added
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "\d users.users"

# Check if admin user was created with secure password
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "SELECT username, email, must_change_password, role FROM users.users WHERE role = 'system_admin';"

# View password policy settings
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "SELECT * FROM users.password_policies;"
```

### Verify Schema Creation
```bash
# List schemas
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "\dn"

# List tables in each schema
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "\dt users.*"
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "\dt agents.*"
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "\dt threats.*"

# Test password security functions
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "SELECT users.validate_password_strength('TestPass123!', 12, TRUE, TRUE, TRUE, TRUE);"
```

### Password Security Configuration
After migration 008, you can configure password policies:

```bash
# View current password policy
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "SELECT * FROM users.password_policies;"

# Update password policy for development (optional - less strict)
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "
UPDATE users.password_policies SET
    min_length = 8,              -- Shorter for dev testing
    max_failed_attempts = 10,    -- More lenient for dev
    lockout_duration_minutes = 5 -- Shorter lockout for dev
WHERE policy_id = (SELECT policy_id FROM users.password_policies LIMIT 1);"

# Reset all account lockouts (useful for development)
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "
UPDATE users.users SET 
    failed_login_attempts = 0,
    last_failed_login = NULL,
    account_locked_until = NULL
WHERE account_locked_until IS NOT NULL;"
```

## After Visual Studio Build Tools Installation

Once you install the Visual Studio C++ Build Tools:

1. **Install sqlx-cli**:
   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   ```

2. **Run proper migrations**:
   ```bash
   sqlx migrate run
   sqlx migrate run --database-url postgresql://secureguard:password@localhost:5432/secureguard_test
   ```

3. **Test compilation**:
   ```bash
   cargo check
   cargo test
   ```

4. **Start development server**:
   ```bash
   cargo run -p secureguard-api
   ```

## Database Connection Details
- **Host**: localhost:5432
- **Database**: secureguard_dev
- **User**: secureguard  
- **Password**: password
- **Test DB**: secureguard_test

## Next Steps Priority
1. **Install Visual Studio C++ Build Tools** (critical blocker)
2. Test manual database setup above
3. Once build tools work, run automated setup
4. Begin Phase 2 development