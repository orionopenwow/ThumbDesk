@echo off
setlocal
cd /d "%~dp0"

if /I "%~1"=="--hot" goto :hotreload
if /I "%~1"=="--watch" goto :hotreload
if /I "%~1"=="--build" goto :build
if /I "%~1"=="--test" goto :test

set "EXE_DIST=%~dp0dist-release\ThumbDesk-Windows-x64\ThumbDesk.exe"
set "EXE_REL=%~dp0src-tauri\target\release\vibe-controller-windows.exe"

:: Ensure single clean instance
taskkill /IM ThumbDesk.exe /F >nul 2>&1
taskkill /IM VibeController.exe /F >nul 2>&1
taskkill /IM vibe-controller-windows.exe /F >nul 2>&1

if exist "%EXE_DIST%" (
    echo Launching ThumbDesk ^(standalone, no compile^)...
    start "" "%EXE_DIST%"
    exit /b 0
)

if exist "%EXE_REL%" (
    echo Launching ThumbDesk ^(release build, no compile^)...
    start "" "%EXE_REL%"
    exit /b 0
)

echo Prebuilt executable not found. Building standalone release now...
goto :build

:hotreload
echo Starting ThumbDesk in live hot-reload mode...
call npx tauri dev
exit /b %ERRORLEVEL%

:build
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0package-release.ps1"
if errorlevel 1 (
    echo Build failed.
    pause
    exit /b 1
)
start "" "%~dp0dist-release\ThumbDesk-Windows-x64\ThumbDesk.exe"
exit /b 0

:test
echo Running frontend and backend tests...
call npm test
if errorlevel 1 exit /b 1
pushd "%~dp0src-tauri"
cargo test --lib
set "TEST_ERR=%ERRORLEVEL%"
popd
exit /b %TEST_ERR%
