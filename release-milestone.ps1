param(
    [Parameter(Mandatory=$true)]
    [string]$Version,
    [string]$Message = "Milestone release",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $root

if (-not $Version.StartsWith("v")) {
    $Version = "v$Version"
}

Write-Host "==> 1/5 Running frontend & Rust tests..."
npm test
if ($LASTEXITCODE -ne 0) { throw "Frontend tests failed" }

Push-Location (Join-Path $root "src-tauri")
cargo test --lib
$cargoErr = $LASTEXITCODE
Pop-Location
if ($cargoErr -ne 0) { throw "Rust tests failed" }

if (-not $SkipBuild) {
    Write-Host "==> 2/5 Building standalone ThumbDesk.exe & ThumbDesk-Windows-x64.zip..."
    & (Join-Path $root "package-release.ps1")
}

$exePath = Join-Path $root "dist-release\ThumbDesk-Windows-x64\ThumbDesk.exe"
$zipPath = Join-Path $root "dist-release\ThumbDesk-Windows-x64.zip"
$sumsPath = Join-Path $root "dist-release\SHA256SUMS.txt"

if (-not (Test-Path $exePath) -or -not (Test-Path $zipPath)) {
    throw "Release binaries missing in dist-release/. Run without -SkipBuild."
}

$exeHash = (Get-FileHash $exePath -Algorithm SHA256).Hash
$zipHash = (Get-FileHash $zipPath -Algorithm SHA256).Hash
@(
    "$exeHash  ThumbDesk.exe"
    "$zipHash  ThumbDesk-Windows-x64.zip"
) | Set-Content -Path $sumsPath -Encoding UTF8

Write-Host "==> 3/5 Committing & pushing milestone to origin/main..."
git add -A
$status = git status --porcelain
if ($status) {
    git commit -m "$Version - $Message"
}
git push -u origin main

Write-Host "==> 4/5 Tagging $Version..."
git tag -f $Version
git push -f origin $Version

Write-Host "==> 5/5 Publishing GitHub Release $Version with EXE, ZIP, and SHA256SUMS..."
$notes = @"
## ThumbDesk $Version

$Message

### Included Assets
- **ThumbDesk.exe** — Standalone single-file Windows x64 executable (UI & 3D assets embedded, zero runtime dependencies)
- **ThumbDesk-Windows-x64.zip** — Portable Windows package with `ThumbDesk.exe`, `profiles/GAPE.json`, and `README.txt`
- **SHA256SUMS.txt** — SHA-256 verification checksums

### Checksums
``````
$exeHash  ThumbDesk.exe
$zipHash  ThumbDesk-Windows-x64.zip
``````
"@

$existing = gh release view $Version 2>$null
if ($LASTEXITCODE -eq 0) {
    gh release upload $Version $exePath $zipPath $sumsPath --clobber
} else {
    gh release create $Version $exePath $zipPath $sumsPath --title "ThumbDesk $Version" --notes $notes
}

Write-Host ""
Write-Host "Milestone $Version pushed and published successfully!"
