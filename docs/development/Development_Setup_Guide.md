# SecureGuard Development Setup Guide

## Prerequisites

### 1. Windows Development Environment Setup

#### Install Visual Studio C++ Build Tools
SecureGuard requires C++ build tools for Rust compilation on Windows.

**Option A: Visual Studio Build Tools 2022 (Recommended)**
1. Download from [Microsoft](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. Run installer and select **"C++ build tools"** workload
3. Include these components:
   - MSVC v143 - VS 2022 C++ compiler toolset
   - Windows 11 SDK (latest version)
   - CMake tools for Visual Studio

**Option B: Visual Studio Community (Full IDE)**
1. Download Visual Studio Community
2. During installation, select **"Desktop development with C++"**

#### Verify Installation
```bash
# Test C++ compilation
rustc --version
# Should work without linker errors
```

### 2. Rust Development Environment

#### Rust Toolchain (✅ Already Installed)
- Rust 1.89.0 confirmed working
- Cargo 1.89.0 package manager

#### Additional Rust Tools
```bash
# Install sqlx CLI for database migrations
cargo install sqlx-cli --no-default-features --features postgres

# Install cargo-watch for development
cargo install cargo-watch

# Install cargo-audit for security scanning
cargo install cargo-audit
```

### 3. Database Setup

#### PostgreSQL Installation

**Option A: Docker (Recommended for Development)**
```bash
# Using existing docker-compose.yml
docker-compose up -d

# Verify PostgreSQL is running
docker ps
```

**Option B: Local PostgreSQL Installation**
1. Download PostgreSQL 15+ from [postgresql.org](https://www.postgresql.org/download/windows/)
2. Install with default settings
3. Create database:
```sql
CREATE DATABASE secureguard;
CREATE USER postgres WITH PASSWORD 'password';
GRANT ALL PRIVILEGES ON DATABASE secureguard TO postgres;
```

#### Database Migration
```bash
# Set database URL
export DATABASE_URL="postgresql://postgres:password@localhost:5432/secureguard"

# Run migrations (includes password security system)
cd C:\Users\smith\Documents\DEV\secure_guard
sqlx migrate run

# Verify migration 008 (password security) applied
sqlx migrate info
```

#### Secure Admin Account Setup
After running migrations, the system automatically creates a secure admin account:

```bash
# The migration will display the generated admin password
# Look for this in the migration output:
# NOTICE: Default admin password: [random-secure-password]
# NOTICE: IMPORTANT: Change this password immediately after first login!
```

**Important Security Notes**:
- Admin password is randomly generated (32 characters)
- Password must be changed on first login
- Demo credentials only available in development mode
- Production deployments use secure defaults only

### 4. Development Tools Setup

#### VSCode Extensions (Recommended)
Install these extensions for optimal Rust development:

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "serayuzgur.crates",
    "ms-vscode.vscode-json",
    "vadimcn.vscode-lldb",
    "tamasfe.even-better-toml"
  ]
}
```

#### VSCode Settings
Add to `.vscode/settings.json`:
```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "editor.rulers": [100],
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

## Telemetry & Observability Setup

### 1. Start Telemetry Infrastructure
```bash
# Start Jaeger, Prometheus, Grafana, and OTEL Collector
docker-compose -f docker-compose.telemetry.yml up -d

# Verify all services are running
docker-compose -f docker-compose.telemetry.yml ps
```

### 2. Configure Environment Variables
```bash
# Backend (.env)
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
ENVIRONMENT=development

# Frontend (.env.local)
VITE_OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
VITE_ENVIRONMENT=development
```

### 3. Access Telemetry Services
- **Jaeger UI** (Tracing): http://localhost:16686
- **Prometheus** (Metrics): http://localhost:9090
- **Grafana** (Dashboards): http://localhost:3001
  - Login: admin/secureguard
- **API Metrics**: http://localhost:8080/metrics

### 4. Verify Telemetry
```bash
# Check if metrics are being collected
curl http://localhost:8080/metrics

# View traces in Jaeger
# 1. Make some API calls
# 2. Open Jaeger UI
# 3. Select "secureguard-api" service
# 4. Click "Find Traces"
```

## Development Workflow

### 1. Initial Setup Verification
```bash
# Navigate to project
cd C:\Users\smith\Documents\DEV\secure_guard

# Check compilation
cargo check

# Run tests (once implemented)
cargo test

# Start development server
cargo run -p secureguard-api
```

### 2. Database Development
```bash
# Create new migration
sqlx migrate add create_new_table

# Apply migrations
sqlx migrate run

# Revert last migration  
sqlx migrate revert
```

### 3. Code Quality Checks
```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Security audit
cargo audit

# Run all checks
cargo fmt && cargo clippy -- -D warnings && cargo test
```

### 4. Development Server
```bash
# Start API server with auto-reload
cargo watch -x "run -p secureguard-api"

# Server runs on: http://localhost:3000
# Health check: http://localhost:3000/health
```

## Environment Configuration

### Required Environment Variables
Create `.env` file in project root:
```bash
DATABASE_URL=postgresql://postgres:password@localhost:5432/secureguard
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-secret-key-change-in-production-use-strong-random-key
PORT=3000

# Development Mode Settings (optional)
DEV_MODE=true                    # Enables demo credentials
DEMO_PASSWORD=SecurePass123!     # Only used in development
```

### Security Configuration

#### Password Policy Settings
Password policies are configured in the database via the `users.password_policies` table:

```sql
-- View current password policy
SELECT * FROM users.password_policies;

-- Update for development (optional - less strict)
UPDATE users.password_policies SET
    min_length = 8,              -- Shorter for dev testing
    max_failed_attempts = 10,    -- More lenient for dev
    lockout_duration_minutes = 5 -- Shorter lockout for dev
WHERE policy_id = (SELECT policy_id FROM users.password_policies LIMIT 1);
```

#### Development vs Production Security
- **Development**: 
  - Demo credentials available (admin@company.com / SecurePass123!)
  - Relaxed password policies (configurable)
  - Extended lockout recovery
- **Production**: 
  - Random admin password generation
  - Strict password policies enforced
  - Standard security lockout procedures

### Development vs Production
- **Development**: Use `.env` file with local services, demo credentials enabled
- **Production**: Use environment variables or secrets management, secure defaults only

## 📋 Development Logging

### Log Configuration for Development

Set appropriate logging levels for development work:

```bash
# Verbose logging for development (recommended)
export RUST_LOG="secureguard_api=debug,tower_http=debug,axum=debug"

# Or add to your .env file
echo 'RUST_LOG=secureguard_api=debug,tower_http=debug,axum=debug' >> .env
```

### Understanding Log Output

**Console Output**: During development, you'll see colored log output in your terminal:
```
2024-08-19T10:30:00.123456Z  INFO secureguard_api::services::auth_service: Password verification successful
2024-08-19T10:30:15.456789Z  WARN secureguard_api::services::user_service: Login attempt failed - invalid password username="test_user"
```

**Log Files**: In development, log files are created in `./logs/`:
```
./logs/
├── secureguard-api.log.2024-08-19     # All application logs
├── security-audit.log.2024-08-19      # Security events only
└── error.log.2024-08-19               # Error-level logs
```

### Development Log Analysis

**Watch security events in real-time**:
```bash
# Monitor security audit log
tail -f logs/security-audit.log.$(date +%Y-%m-%d) | jq '.'

# Monitor only login events
tail -f logs/security-audit.log.$(date +%Y-%m-%d) | jq 'select(.fields.event | contains("login"))'
```

**Debug API key operations**:
```bash
# Watch API key events
tail -f logs/security-audit.log.$(date +%Y-%m-%d) | jq 'select(.fields.event | contains("api_key"))'
```

**Monitor errors**:
```bash
# Watch error log
tail -f logs/error.log.$(date +%Y-%m-%d)
```

### Testing Logging Functionality

**Test login event logging**:
```bash
# Start the server
cargo run -p secureguard-api

# In another terminal, trigger login event
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@company.com","password":"wrong"}'

# Check that security audit log captured the event
tail -1 logs/security-audit.log.$(date +%Y-%m-%d) | jq '.fields.event'
# Should output: "login_failed"
```

**Test API key event logging**:
```bash
# Create API key (requires valid auth)
curl -X POST http://localhost:3000/api/v1/api-keys \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"test-key"}'

# Check API key creation was logged
grep "api_key_created" logs/security-audit.log.$(date +%Y-%m-%d)
```

### Debugging with Logs

**Common debugging patterns**:

1. **Authentication Issues**: Check security-audit.log for failed login attempts
2. **Database Problems**: Check main log for database connection errors
3. **Performance Issues**: Look for slow query logs in main application log
4. **Agent Registration**: Monitor agent registration events in security log

**Log Level Guidelines**:
- `TRACE`: Very detailed (rarely needed)
- `DEBUG`: Development debugging (recommended for dev)
- `INFO`: Production default, business events
- `WARN`: Issues that should be addressed
- `ERROR`: Problems requiring immediate attention

### First-Time Development Login

#### Option 1: Demo Credentials (Development Only)
If `DEV_MODE=true` in environment:
```
Email: admin@company.com
Password: SecurePass123!
```

#### Option 2: Generated Admin Account
Use the randomly generated admin credentials from migration output:
```
Email: admin@secureguard.local
Password: [check migration output for generated password]
```

**Note**: Both options require immediate password change on first login due to security policy enforcement.

## Testing Setup

### Test Database
```bash
# Create test database
createdb secureguard_test

# Set test environment
export DATABASE_URL_TEST="postgresql://postgres:password@localhost:5432/secureguard_test"
```

### Running Tests
```bash
# Unit tests
cargo test --lib

# Integration tests  
cargo test --test integration

# All tests with coverage
cargo test -- --nocapture
```

## Troubleshooting

### Common Issues

#### 1. Linker Errors on Windows
```
error: linking with `link.exe` failed: exit code: 1
```
**Solution**: Install Visual Studio C++ Build Tools (see above)

#### 2. Database Connection Errors
```
error: Connection refused (os error 10061)
```
**Solution**: Ensure PostgreSQL is running and DATABASE_URL is correct

#### 3. Port Already in Use
```
error: Address already in use (os error 10048)
```
**Solution**: Change PORT in .env or kill process using port 3000

#### 4. Missing SQLx Prepare Data
```
error: could not find migration files
```
**Solution**: 
```bash
# Generate SQLx metadata
cargo sqlx prepare
```

## Performance Optimization

### Development Performance
- Use `cargo check` for faster compilation during development
- Use `cargo watch` for automatic recompilation
- Enable incremental compilation: `export CARGO_INCREMENTAL=1`

### Database Performance
- Use connection pooling (already configured)
- Index frequently queried columns
- Monitor slow queries in development

## Security Considerations

### Development Security
- Never commit `.env` files to version control
- Use strong JWT secrets (32+ random characters)
- Regularly update dependencies: `cargo update`
- Run security audits: `cargo audit`

### Database Security  
- Use parameterized queries (SQLx provides this)
- Enable SSL in production database connections
- Implement proper backup procedures

---

**Next Steps**: Complete this setup, then proceed with implementing the testing framework and Phase 2 MVP features.