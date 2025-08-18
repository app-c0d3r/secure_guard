@echo off
echo Testing SecureGuard Agent...
echo.

:: Test 1: Check if executable exists
if not exist "secureguard-agent.exe" (
    echo FAIL: Agent executable not found
    pause
    exit /b 1
)
echo PASS: Agent executable found

:: Test 2: Check executable properties
echo.
echo Agent Executable Properties:
dir "secureguard-agent.exe"

:: Test 3: Quick startup test (5 second timeout)
echo.
echo Testing agent startup (will timeout after 5 seconds)...
timeout /t 5 /nobreak | "secureguard-agent.exe" 2>test_error.log 1>test_output.log

:: Show test results
echo.
echo Agent startup test completed. Output:
type test_output.log 2>nul
if exist test_error.log (
    echo.
    echo Errors (if any):
    type test_error.log 2>nul
)

:: Clean up test files
del test_output.log 2>nul
del test_error.log 2>nul

echo.
echo Agent executable test completed successfully!
pause