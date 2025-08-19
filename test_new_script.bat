@echo off
echo Testing enhanced myservice.bat script...
echo.

echo ===== Test 1: Help display =====
call scripts\myservice.bat
echo.

echo ===== Test 2: Invalid command =====  
call scripts\myservice.bat invalid
echo.

echo ===== All tests completed =====
echo Press any key to continue...
pause >nul