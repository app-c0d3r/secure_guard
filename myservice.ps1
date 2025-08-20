#!/usr/bin/env pwsh
# SecureGuard PowerShell Wrapper
# This script forwards commands to the appropriate platform-specific script

param(
    [string]$Command = ""
)

$scriptPath = Join-Path $PSScriptRoot "scripts\deployment\myservice.bat"

if ($Command -eq "") {
    & $scriptPath
} else {
    & $scriptPath $Command
}