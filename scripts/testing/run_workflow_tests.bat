@echo off
echo ========================================
echo SecureGuard Workflow Tests Runner
echo ========================================
echo.

REM Set test environment variables
set DATABASE_URL_TEST=postgresql://secureguard:password@localhost:5432/secureguard_dev
set RUST_LOG=info

echo Checking database connection...
psql -h localhost -U secureguard -d secureguard_dev -c "SELECT 1;" >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Cannot connect to test database
    echo Please ensure PostgreSQL is running and the database exists
    exit /b 1
)

echo Setting up test database...
cd /d "%~dp0.."

REM Run migrations to ensure schema is up to date
echo Running database migrations...
sqlx migrate run --database-url "%DATABASE_URL_TEST%"

echo.
echo ========================================
echo Running Workflow Tests
echo ========================================
echo.

REM Run specific workflow tests
echo [1/6] Running authentication workflow tests...
cargo test --package secureguard-api --test workflow_tests test_multi_role_authentication_workflow -- --nocapture

echo.
echo [2/6] Running security analyst workflow tests...
cargo test --package secureguard-api --test workflow_tests test_security_analyst_full_workflow -- --nocapture
cargo test --package secureguard-api --test workflow_tests test_analyst_daily_tasks_workflow -- --nocapture

echo.
echo [3/6] Running admin workflow tests...
cargo test --package secureguard-api --test workflow_tests test_admin_user_management_workflow -- --nocapture

echo.
echo [4/6] Running system admin workflow tests...
cargo test --package secureguard-api --test workflow_tests test_system_admin_full_access_workflow -- --nocapture

echo.
echo [5/6] Running user role tests...
cargo test --package secureguard-api --test workflow_tests test_regular_user_limited_workflow -- --nocapture
cargo test --package secureguard-api --test workflow_tests test_readonly_user_view_only_workflow -- --nocapture

echo.
echo [6/6] Running comprehensive end-to-end tests...
cargo test --package secureguard-api --test workflow_tests test_complete_application_workflow -- --nocapture
cargo test --package secureguard-api --test workflow_tests test_role_hierarchy_enforcement -- --nocapture

echo.
echo ========================================
echo Running Test Data Setup Verification
echo ========================================
echo.
cargo test --package secureguard-api --test test_data_setup -- --nocapture

echo.
echo ========================================
echo Running Error Scenario Tests
echo ========================================
echo.
cargo test --package secureguard-api --test workflow_tests test_invalid_credentials_and_permissions -- --nocapture

echo.
echo ========================================
echo Test Summary
echo ========================================
echo.

REM Run all workflow tests together for final summary
echo Running ALL workflow tests...
cargo test --package secureguard-api --test workflow_tests -- --nocapture

if %errorlevel% equ 0 (
    echo.
    echo ✅ ALL WORKFLOW TESTS PASSED!
    echo.
    echo Test Coverage Summary:
    echo - ✅ Authentication and login flows
    echo - ✅ Security Analyst role workflows
    echo - ✅ Admin role workflows  
    echo - ✅ System Admin role workflows
    echo - ✅ Regular User role workflows
    echo - ✅ Read-only User role workflows
    echo - ✅ Role hierarchy enforcement
    echo - ✅ End-to-end application workflows
    echo - ✅ Error scenarios and edge cases
    echo.
) else (
    echo.
    echo ❌ SOME TESTS FAILED!
    echo Please review the test output above for details.
    echo.
    exit /b 1
)

pause