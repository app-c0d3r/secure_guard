# PowerShell script to run tests with proper setup
# Usage: .\scripts\run_tests.ps1

Write-Host "🧪 Setting up SecureGuard Test Environment" -ForegroundColor Green

# Check if PostgreSQL is running
$pgProcess = Get-Process postgres -ErrorAction SilentlyContinue
if (-not $pgProcess) {
    Write-Host "❌ PostgreSQL not running. Please start PostgreSQL first:" -ForegroundColor Red
    Write-Host "   Option A: docker-compose up -d" -ForegroundColor Yellow
    Write-Host "   Option B: Start local PostgreSQL service" -ForegroundColor Yellow
    exit 1
}

Write-Host "✅ PostgreSQL is running" -ForegroundColor Green

# Set test environment variables
$env:DATABASE_URL_TEST = "postgresql://secureguard:password@localhost:5432/secureguard_test"
$env:JWT_SECRET = "test-secret-key-for-testing-only"

Write-Host "🗄️ Setting up test database..." -ForegroundColor Blue

# Create test database if it doesn't exist
try {
    & psql -U secureguard -h localhost -p 5432 -c "CREATE DATABASE secureguard_test;" 2>$null
    Write-Host "✅ Test database created" -ForegroundColor Green
} catch {
    Write-Host "ℹ️ Test database already exists or connection failed" -ForegroundColor Yellow
}

Write-Host "🔧 Running migrations on test database..." -ForegroundColor Blue
& sqlx migrate run --database-url $env:DATABASE_URL_TEST

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Migration failed. Make sure sqlx-cli is installed:" -ForegroundColor Red
    Write-Host "   cargo install sqlx-cli --no-default-features --features postgres" -ForegroundColor Yellow
    exit 1
}

Write-Host "✅ Migrations completed" -ForegroundColor Green

Write-Host "🧪 Running unit tests..." -ForegroundColor Blue
& cargo test --lib --verbose

Write-Host "🌐 Running integration tests..." -ForegroundColor Blue  
& cargo test --test integration_tests --verbose

Write-Host "🎯 Running all tests..." -ForegroundColor Blue
& cargo test --verbose

Write-Host "✅ All tests completed!" -ForegroundColor Green
Write-Host "📊 Test Results Summary:" -ForegroundColor Cyan
Write-Host "   - Unit Tests: Service layer, authentication, validation" -ForegroundColor White
Write-Host "   - Integration Tests: Full API endpoint testing" -ForegroundColor White
Write-Host "   - Database Tests: PostgreSQL integration" -ForegroundColor White