$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $root

Get-Process -Name "ThumbDesk", "VibeController", "vibe-controller-windows" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 300

Write-Host "==> 1/3 Building standalone ThumbDesk executable (embedded UI)..."
npx tauri build --no-bundle
if ($LASTEXITCODE -ne 0) { throw "tauri build failed" }

$builtExe = Join-Path $root "src-tauri\target\release\vibe-controller-windows.exe"
if (-not (Test-Path $builtExe)) { throw "Expected binary not found at $builtExe" }

$releaseRoot = Join-Path $root "dist-release"
$stageDir = Join-Path $releaseRoot "ThumbDesk-Windows-x64"
$zipPath = Join-Path $releaseRoot "ThumbDesk-Windows-x64.zip"

Write-Host "==> 2/3 Staging standalone portable folder..."
Get-Process -Name "ThumbDesk", "VibeController" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 300
if (Test-Path $stageDir) { Remove-Item $stageDir -Recurse -Force }
New-Item -ItemType Directory -Force -Path "$stageDir\profiles" | Out-Null

Copy-Item $builtExe (Join-Path $stageDir "ThumbDesk.exe") -Force
Copy-Item (Join-Path $root "profiles\GAPE.json") (Join-Path $stageDir "profiles\GAPE.json") -Force
Copy-Item (Join-Path $root "README.md") (Join-Path $stageDir "README.txt") -Force

Write-Host "==> 3/3 Creating standalone product ZIP..."
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
Compress-Archive -Path "$stageDir\*" -DestinationPath $zipPath -CompressionLevel Optimal

$exeSizeMB = [math]::Round((Get-Item (Join-Path $stageDir "ThumbDesk.exe")).Length / 1MB, 2)
$zipSizeMB = [math]::Round((Get-Item $zipPath).Length / 1MB, 2)
$hash = (Get-FileHash $zipPath -Algorithm SHA256).Hash

Write-Host ""
Write-Host "Standalone build complete!"
Write-Host "  EXE : $stageDir\ThumbDesk.exe ($exeSizeMB MB)"
Write-Host "  ZIP : $zipPath ($zipSizeMB MB)"
Write-Host "  SHA256: $hash"
