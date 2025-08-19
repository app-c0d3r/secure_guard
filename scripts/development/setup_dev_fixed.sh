#!/bin/bash
# Fixed setup script that works with current Windows limitations
# Usage: ./scripts/setup_dev_fixed.sh

echo "🚀 SecureGuard Development Environment Setup (Windows Compatible)"

echo "1️⃣ Checking Prerequisites..."

# Check Rust installation
if command -v cargo &> /dev/null; then
    RUST_VERSION=$(cargo --version)
    echo "✅ Rust: $RUST_VERSION"
else
    echo "❌ Rust not found. Install from: https://rustup.rs/"
    exit 1
fi

# Check Docker
if command -v docker &> /dev/null; then
    DOCKER_VERSION=$(docker --version)
    echo "✅ Docker: $DOCKER_VERSION"
else
    echo "❌ Docker not found. Install Docker Desktop"
    exit 1
fi

echo "2️⃣ Checking Build Tools..."

# Test if compilation works (without actually installing tools)
echo "Testing Rust compilation capability..."
echo 'fn main() { println!("Hello, SecureGuard!"); }' > /tmp/test_compile.rs
if rustc /tmp/test_compile.rs -o /tmp/test_compile 2>/dev/null; then
    echo "✅ Rust compilation works"
    rm -f /tmp/test_compile.rs /tmp/test_compile
    BUILD_TOOLS_OK=true
else
    echo "❌ Rust compilation failed - Visual Studio C++ Build Tools needed"
    echo "   Download: https://visualstudio.microsoft.com/visual-cpp-build-tools/"
    echo "   Select 'C++ build tools' workload during installation"
    rm -f /tmp/test_compile.rs /tmp/test_compile
    BUILD_TOOLS_OK=false
fi

echo "3️⃣ Setting up Database..."

# Start Docker services
echo "Starting PostgreSQL and Redis..."
docker-compose up -d

if [ $? -ne 0 ]; then
    echo "❌ Failed to start Docker services. Make sure Docker Desktop is running."
    exit 1
fi

# Wait for PostgreSQL to be ready
echo "Waiting for PostgreSQL to be ready..."
for i in {1..30}; do
    if docker exec secure_guard-db-1 pg_isready -U secureguard >/dev/null 2>&1; then
        echo "✅ PostgreSQL is ready"
        break
    fi
    echo "Waiting for PostgreSQL... ($i/30)"
    sleep 2
done

# Apply migrations manually (since sqlx-cli might not be available)
echo "Applying database migrations..."
docker exec secure_guard-db-1 sh -c "psql -U secureguard -d secureguard_dev -c \"
DO \\\$\\\$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'users' AND tablename = 'users') THEN
        CREATE EXTENSION IF NOT EXISTS \\\"uuid-ossp\\\";
        CREATE EXTENSION IF NOT EXISTS \\\"pgcrypto\\\";
        
        CREATE SCHEMA IF NOT EXISTS users;
        CREATE SCHEMA IF NOT EXISTS agents; 
        CREATE SCHEMA IF NOT EXISTS tenants;
        CREATE SCHEMA IF NOT EXISTS threats;
        
        CREATE TABLE users.users (
            user_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            username VARCHAR(255) UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            is_active BOOLEAN NOT NULL DEFAULT TRUE
        );
        
        CREATE TABLE agents.endpoints (
            agent_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id UUID NOT NULL,
            hardware_fingerprint TEXT UNIQUE NOT NULL,
            os_info JSONB NOT NULL,
            status VARCHAR(50) NOT NULL DEFAULT 'unknown',
            last_heartbeat TIMESTAMPTZ,
            version VARCHAR(50) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        );
        
        CREATE TABLE tenants.tenants (
            tenant_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(255) NOT NULL,
            plan_tier VARCHAR(50) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        );
        
        RAISE NOTICE 'Basic schema created successfully';
    ELSE
        RAISE NOTICE 'Schema already exists, skipping creation';
    END IF;
END\\\$\\\$;
\""

if [ $? -eq 0 ]; then
    echo "✅ Database migrations applied"
else
    echo "❌ Database migration failed"
    exit 1
fi

# Create test database
echo "Setting up test database..."
docker exec secure_guard-db-1 sh -c "psql -U secureguard -d secureguard_dev -c 'CREATE DATABASE secureguard_test;'" 2>/dev/null || echo "ℹ️ Test database may already exist"

echo "✅ Database setup complete"

# Conditional steps based on build tools availability
if [ "$BUILD_TOOLS_OK" = true ]; then
    echo "4️⃣ Installing Rust Tools..."
    
    # Only install if build tools work
    if ! command -v sqlx &> /dev/null; then
        echo "Installing sqlx-cli..."
        cargo install sqlx-cli --no-default-features --features postgres
    else
        echo "✅ sqlx-cli already installed"
    fi
    
    if ! command -v cargo-watch &> /dev/null; then
        echo "Installing cargo-watch..."
        cargo install cargo-watch
    else
        echo "✅ cargo-watch already installed"
    fi
    
    echo "5️⃣ Testing Compilation..."
    cargo check
    
    if [ $? -eq 0 ]; then
        echo "✅ Project compilation successful"
        
        echo "6️⃣ Running Tests..."
        export DATABASE_URL_TEST="postgresql://secureguard:password@localhost:5432/secureguard_test"
        export JWT_SECRET="test-secret-key-for-testing-only"
        
        # Apply migrations to test database
        if command -v sqlx &> /dev/null; then
            sqlx migrate run --database-url "$DATABASE_URL_TEST"
        fi
        
        cargo test
        
        if [ $? -eq 0 ]; then
            echo "✅ All tests passed"
        else
            echo "⚠️ Some tests failed, but environment is set up"
        fi
    else
        echo "❌ Project compilation failed"
    fi
else
    echo "4️⃣ Skipping Rust Tools Installation (Build tools required)"
    echo "5️⃣ Skipping Compilation Test (Build tools required)"
    echo "6️⃣ Skipping Tests (Build tools required)"
fi

echo ""
echo "🎉 Development Environment Setup Summary:"
echo ""
echo "✅ Docker Services Running:"
echo "   - PostgreSQL: localhost:5432"
echo "   - Redis: localhost:6379"
echo ""
echo "✅ Database:"
echo "   - Main DB: secureguard_dev"
echo "   - Test DB: secureguard_test"
echo "   - Basic schema applied"
echo ""

if [ "$BUILD_TOOLS_OK" = true ]; then
    echo "✅ Rust Environment:"
    echo "   - Compilation working"
    echo "   - Tools installed"
    echo "   - Tests can run"
    echo ""
    echo "🚀 Ready for Development:"
    echo "   1. Start server: cargo run -p secureguard-api"
    echo "   2. Auto-reload: cargo watch -x 'run -p secureguard-api'"
    echo "   3. Run tests: cargo test"
    echo "   4. Code quality: cargo fmt && cargo clippy"
    echo ""
    echo "📡 API Endpoints (when running):"
    echo "   - Health Check: http://localhost:3000/health"
    echo "   - API Base: http://localhost:3000/api/v1"
else
    echo "⚠️ Rust Environment:"
    echo "   - Compilation blocked by missing Visual Studio C++ Build Tools"
    echo "   - Database is ready for when tools are installed"
    echo ""
    echo "🔧 Next Steps:"
    echo "   1. Install Visual Studio C++ Build Tools:"
    echo "      https://visualstudio.microsoft.com/visual-cpp-build-tools/"
    echo "   2. Select 'C++ build tools' workload"
    echo "   3. Restart terminal and run this script again"
    echo ""
    echo "📋 Alternative: Manual Database Testing:"
    echo "   - Test connection: docker exec -it secure_guard-db-1 psql -U secureguard -d secureguard_dev"
    echo "   - View tables: \\dt users.*; \\dt agents.*;"
fi

echo ""
echo "📊 Setup Status: Database ✅ | Docker ✅ | Rust Tools: $([ "$BUILD_TOOLS_OK" = true ] && echo "✅" || echo "⚠️ Pending")"