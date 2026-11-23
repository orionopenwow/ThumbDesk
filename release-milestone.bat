@echo off
setlocal
set /p VER="Enter milestone version (e.g. v1.0.1): "
set /p MSG="Enter milestone summary: "
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0release-milestone.ps1" -Version "%VER%" -Message "%MSG%"
if errorlevel 1 (
    echo Release failed.
    pause
    exit /b 1
)
pause
