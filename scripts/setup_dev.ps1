# PowerShell script to set up complete development environment
# Usage: .\scripts\setup_dev.ps1

Write-Host "🚀 SecureGuard Development Environment Setup" -ForegroundColor Green

Write-Host "1️⃣ Checking Prerequisites..." -ForegroundColor Blue

# Check Rust installation
try {
    $rustVersion = & cargo --version
    Write-Host "✅ Rust: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Rust not found. Install from: https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# Check Docker
try {
    $dockerVersion = & docker --version
    Write-Host "✅ Docker: $dockerVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Docker not found. Install Docker Desktop" -ForegroundColor Red
    exit 1
}

Write-Host "2️⃣ Installing Rust Tools..." -ForegroundColor Blue

# Install sqlx-cli
Write-Host "Installing sqlx-cli..." -ForegroundColor Yellow
& cargo install sqlx-cli --no-default-features --features postgres

# Install cargo-watch for development
Write-Host "Installing cargo-watch..." -ForegroundColor Yellow
& cargo install cargo-watch

# Install cargo-audit for security
Write-Host "Installing cargo-audit..." -ForegroundColor Yellow
& cargo install cargo-audit

Write-Host "✅ Rust tools installed" -ForegroundColor Green

Write-Host "3️⃣ Setting up Database..." -ForegroundColor Blue

# Start Docker services
Write-Host "Starting PostgreSQL and Redis..." -ForegroundColor Yellow
& docker-compose up -d

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Failed to start Docker services. Make sure Docker Desktop is running." -ForegroundColor Red
    exit 1
}

# Wait for PostgreSQL to be ready
Write-Host "Waiting for PostgreSQL to be ready..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

# Run migrations
Write-Host "Running database migrations..." -ForegroundColor Yellow
& sqlx migrate run

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Migration failed" -ForegroundColor Red
    exit 1
}

# Create test database
Write-Host "Setting up test database..." -ForegroundColor Yellow
& psql -U secureguard -h localhost -p 5432 -c "CREATE DATABASE secureguard_test;" 2>$null

Write-Host "✅ Database setup complete" -ForegroundColor Green

Write-Host "4️⃣ Testing Compilation..." -ForegroundColor Blue

# Test compilation
Write-Host "Testing project compilation..." -ForegroundColor Yellow
& cargo check

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Compilation failed. Make sure Visual Studio C++ Build Tools are installed:" -ForegroundColor Red
    Write-Host "   Download: https://visualstudio.microsoft.com/visual-cpp-build-tools/" -ForegroundColor Yellow
    Write-Host "   Select 'C++ build tools' workload during installation" -ForegroundColor Yellow
    exit 1
}

Write-Host "✅ Compilation successful" -ForegroundColor Green

Write-Host "5️⃣ Running Tests..." -ForegroundColor Blue

# Set test environment
$env:DATABASE_URL_TEST = "postgresql://secureguard:password@localhost:5432/secureguard_test"
$env:JWT_SECRET = "test-secret-key-for-testing-only"

# Run migrations on test database
& sqlx migrate run --database-url $env:DATABASE_URL_TEST

# Run tests
Write-Host "Running test suite..." -ForegroundColor Yellow
& cargo test

if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️ Some tests failed, but environment is set up" -ForegroundColor Yellow
} else {
    Write-Host "✅ All tests passed" -ForegroundColor Green
}

Write-Host "🎉 Development Environment Setup Complete!" -ForegroundColor Green
Write-Host ""
Write-Host "🚀 Next Steps:" -ForegroundColor Cyan
Write-Host "   1. Start development server: cargo run -p secureguard-api" -ForegroundColor White
Write-Host "   2. Auto-reload development: cargo watch -x 'run -p secureguard-api'" -ForegroundColor White
Write-Host "   3. Run tests: cargo test" -ForegroundColor White
Write-Host "   4. Code quality: cargo fmt && cargo clippy" -ForegroundColor White
Write-Host ""
Write-Host "📡 Services Running:" -ForegroundColor Cyan
Write-Host "   - API Server: http://localhost:3000" -ForegroundColor White
Write-Host "   - Health Check: http://localhost:3000/health" -ForegroundColor White
Write-Host "   - PostgreSQL: localhost:5432" -ForegroundColor White
Write-Host "   - Redis: localhost:6379" -ForegroundColor White