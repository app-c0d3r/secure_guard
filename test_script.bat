@echo off
echo Testing myservice.bat script...
echo.

echo ===== Test 1: No parameters (should show help) =====
call scripts\myservice.bat
echo.

echo ===== Test 2: Help command =====
call scripts\myservice.bat help
echo.

echo ===== Test 3: Invalid command =====
call scripts\myservice.bat invalid
echo.

echo ===== All tests completed =====
pause