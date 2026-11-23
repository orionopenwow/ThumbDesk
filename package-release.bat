@echo off
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0package-release.ps1"
if errorlevel 1 (
    echo Packaging failed.
    pause
    exit /b 1
)
