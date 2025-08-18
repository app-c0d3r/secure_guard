#!/bin/bash
# Bash script to set up complete development environment
# Usage: ./scripts/setup_dev.sh

echo "🚀 SecureGuard Development Environment Setup"

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

echo "2️⃣ Installing Rust Tools..."

# Install sqlx-cli
echo "Installing sqlx-cli..."
cargo install sqlx-cli --no-default-features --features postgres

# Install cargo-watch for development
echo "Installing cargo-watch..."
cargo install cargo-watch

# Install cargo-audit for security
echo "Installing cargo-audit..."
cargo install cargo-audit

echo "✅ Rust tools installed"

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
sleep 10

# Run migrations
echo "Running database migrations..."
sqlx migrate run

if [ $? -ne 0 ]; then
    echo "❌ Migration failed"
    exit 1
fi

# Create test database
echo "Setting up test database..."
psql -U secureguard -h localhost -p 5432 -c "CREATE DATABASE secureguard_test;" 2>/dev/null || echo "Test database may already exist"

echo "✅ Database setup complete"

echo "4️⃣ Testing Compilation..."

# Test compilation
echo "Testing project compilation..."
cargo check

if [ $? -ne 0 ]; then
    echo "❌ Compilation failed. Make sure Visual Studio C++ Build Tools are installed:"
    echo "   Download: https://visualstudio.microsoft.com/visual-cpp-build-tools/"
    echo "   Select 'C++ build tools' workload during installation"
    exit 1
fi

echo "✅ Compilation successful"

echo "5️⃣ Running Tests..."

# Set test environment
export DATABASE_URL_TEST="postgresql://secureguard:password@localhost:5432/secureguard_test"
export JWT_SECRET="test-secret-key-for-testing-only"

# Run migrations on test database
sqlx migrate run --database-url "$DATABASE_URL_TEST"

# Run tests
echo "Running test suite..."
cargo test

if [ $? -ne 0 ]; then
    echo "⚠️ Some tests failed, but environment is set up"
else
    echo "✅ All tests passed"
fi

echo "🎉 Development Environment Setup Complete!"
echo ""
echo "🚀 Next Steps:"
echo "   1. Start development server: cargo run -p secureguard-api"
echo "   2. Auto-reload development: cargo watch -x 'run -p secureguard-api'"
echo "   3. Run tests: cargo test"
echo "   4. Code quality: cargo fmt && cargo clippy"
echo ""
echo "📡 Services Running:"
echo "   - API Server: http://localhost:3000"
echo "   - Health Check: http://localhost:3000/health"
echo "   - PostgreSQL: localhost:5432"
echo "   - Redis: localhost:6379"