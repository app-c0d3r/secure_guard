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
# Copy migration files to container and run manually
docker cp migrations/V001_create_initial_schema.sql secure_guard-db-1:/tmp/
docker cp migrations/V002_create_threats_schema.sql secure_guard-db-1:/tmp/

# Run migrations manually
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -f /tmp/V001_create_initial_schema.sql
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -f /tmp/V002_create_threats_schema.sql

# Create test database
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "CREATE DATABASE secureguard_test;"
```

### Verify Schema Creation
```bash
# List schemas
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "\dn"

# List tables in each schema
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "\dt users.*"
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "\dt agents.*"
docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev -c "\dt threats.*"
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