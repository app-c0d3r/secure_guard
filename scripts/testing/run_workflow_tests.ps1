#!/usr/bin/env pwsh

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "SecureGuard Workflow Tests Runner" -ForegroundColor Cyan  
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Set test environment variables
$env:DATABASE_URL_TEST = "postgresql://secureguard:password@localhost:5432/secureguard_dev"
$env:RUST_LOG = "info"

Write-Host "Checking database connection..." -ForegroundColor Yellow
try {
    $null = psql -h localhost -U secureguard -d secureguard_dev -c "SELECT 1;" 2>$null
    if ($LASTEXITCODE -ne 0) { throw "Connection failed" }
    Write-Host "✅ Database connection successful" -ForegroundColor Green
} catch {
    Write-Host "❌ ERROR: Cannot connect to test database" -ForegroundColor Red
    Write-Host "Please ensure PostgreSQL is running and the database exists" -ForegroundColor Red
    exit 1
}

Write-Host "Setting up test database..." -ForegroundColor Yellow
Set-Location (Split-Path $PSScriptRoot)

# Run migrations to ensure schema is up to date
Write-Host "Running database migrations..." -ForegroundColor Yellow
sqlx migrate run --database-url "$env:DATABASE_URL_TEST"

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Running Workflow Tests" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Define test functions for better organization
function Run-TestGroup {
    param(
        [string]$GroupName,
        [string[]]$Tests
    )
    
    Write-Host "[$GroupName] Running tests..." -ForegroundColor Magenta
    foreach ($test in $Tests) {
        Write-Host "  → $test" -ForegroundColor Gray
        $result = cargo test --package secureguard-api --test workflow_tests $test --nocapture
        if ($LASTEXITCODE -ne 0) {
            Write-Host "    ❌ FAILED" -ForegroundColor Red
            return $false
        } else {
            Write-Host "    ✅ PASSED" -ForegroundColor Green
        }
    }
    return $true
}

# Track test results
$testResults = @{}
$allPassed = $true

# Authentication Tests
$authTests = @("test_multi_role_authentication_workflow")
$testResults["Authentication"] = Run-TestGroup "1/6 Authentication" $authTests
$allPassed = $allPassed -and $testResults["Authentication"]

Write-Host ""

# Security Analyst Tests
$analystTests = @(
    "test_security_analyst_full_workflow",
    "test_analyst_daily_tasks_workflow"
)
$testResults["SecurityAnalyst"] = Run-TestGroup "2/6 Security Analyst" $analystTests
$allPassed = $allPassed -and $testResults["SecurityAnalyst"]

Write-Host ""

# Admin Tests
$adminTests = @("test_admin_user_management_workflow")
$testResults["Admin"] = Run-TestGroup "3/6 Admin" $adminTests
$allPassed = $allPassed -and $testResults["Admin"]

Write-Host ""

# System Admin Tests
$sysAdminTests = @("test_system_admin_full_access_workflow")
$testResults["SystemAdmin"] = Run-TestGroup "4/6 System Admin" $sysAdminTests
$allPassed = $allPassed -and $testResults["SystemAdmin"]

Write-Host ""

# User Role Tests
$userTests = @(
    "test_regular_user_limited_workflow",
    "test_readonly_user_view_only_workflow"
)
$testResults["UserRoles"] = Run-TestGroup "5/6 User Roles" $userTests
$allPassed = $allPassed -and $testResults["UserRoles"]

Write-Host ""

# End-to-End Tests
$e2eTests = @(
    "test_complete_application_workflow",
    "test_role_hierarchy_enforcement"
)
$testResults["EndToEnd"] = Run-TestGroup "6/6 End-to-End" $e2eTests
$allPassed = $allPassed -and $testResults["EndToEnd"]

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Running Test Data Setup Verification" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$setupResult = cargo test --package secureguard-api --test test_data_setup --nocapture
$testResults["TestDataSetup"] = ($LASTEXITCODE -eq 0)
$allPassed = $allPassed -and $testResults["TestDataSetup"]

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Running Error Scenario Tests" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$errorTests = @("test_invalid_credentials_and_permissions")
$testResults["ErrorScenarios"] = Run-TestGroup "Error Scenarios" $errorTests
$allPassed = $allPassed -and $testResults["ErrorScenarios"]

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Test Summary" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Display detailed results
foreach ($category in $testResults.Keys) {
    $status = if ($testResults[$category]) { "✅ PASSED" } else { "❌ FAILED" }
    $color = if ($testResults[$category]) { "Green" } else { "Red" }
    Write-Host "$category`: $status" -ForegroundColor $color
}

Write-Host ""

if ($allPassed) {
    Write-Host "🎉 ALL WORKFLOW TESTS PASSED!" -ForegroundColor Green -BackgroundColor Black
    Write-Host ""
    Write-Host "Test Coverage Summary:" -ForegroundColor White
    Write-Host "- ✅ Authentication and login flows" -ForegroundColor Green
    Write-Host "- ✅ Security Analyst role workflows (login, daily tasks, monitoring)" -ForegroundColor Green
    Write-Host "- ✅ Admin role workflows (user management, agent control)" -ForegroundColor Green
    Write-Host "- ✅ System Admin role workflows (full system access)" -ForegroundColor Green
    Write-Host "- ✅ Regular User role workflows (limited access)" -ForegroundColor Green
    Write-Host "- ✅ Read-only User role workflows (view-only)" -ForegroundColor Green
    Write-Host "- ✅ Role hierarchy enforcement" -ForegroundColor Green
    Write-Host "- ✅ End-to-end application workflows" -ForegroundColor Green
    Write-Host "- ✅ Error scenarios and edge cases" -ForegroundColor Green
    Write-Host "- ✅ Test data setup and teardown" -ForegroundColor Green
    Write-Host ""
    Write-Host "🛡️  Your SecureGuard application is ready for production!" -ForegroundColor Cyan
} else {
    Write-Host "❌ SOME TESTS FAILED!" -ForegroundColor Red -BackgroundColor Black
    Write-Host "Please review the test output above for details." -ForegroundColor Yellow
    Write-Host ""
    
    Write-Host "Failed test categories:" -ForegroundColor Red
    foreach ($category in $testResults.Keys) {
        if (-not $testResults[$category]) {
            Write-Host "- ❌ $category" -ForegroundColor Red
        }
    }
    
    exit 1
}

Write-Host ""
Write-Host "Press any key to continue..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")