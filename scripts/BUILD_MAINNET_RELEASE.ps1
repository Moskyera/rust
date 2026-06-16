# HIP-25 mainnet release build: hacash.exe (release) + WASM SDK + integrity manifest
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$ReleaseDir = Join-Path $Root "target\release"
$PkgDir = Join-Path $ReleaseDir "pkg"
$WalletPkg = Join-Path $Root "wallet\hip25\pkg"

Write-Host "=== HIP-25 Mainnet Release Build ===" -ForegroundColor Cyan
Set-Location $Root

Write-Host "[1/3] cargo build --release ..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) { throw "cargo build --release failed" }

Write-Host "[2/3] WASM SDK (wasm32) ..." -ForegroundColor Yellow
rustup target add wasm32-unknown-unknown 2>&1 | Out-Null
cargo build --features sdk --target wasm32-unknown-unknown --release --lib
if ($LASTEXITCODE -ne 0) { throw "WASM lib build failed" }

$WasmSrc = Join-Path $Root "target\wasm32-unknown-unknown\release\hacash_sdk.wasm"
if (-not (Test-Path $WasmSrc)) { throw "WASM artifact missing: $WasmSrc" }

New-Item -ItemType Directory -Force -Path $PkgDir, $WalletPkg | Out-Null
wasm-bindgen $WasmSrc --out-dir $PkgDir --target no-modules --no-typescript

Copy-Item (Join-Path $PkgDir "hacash_sdk.js") (Join-Path $WalletPkg "hacash_sdk.js") -Force
Copy-Item (Join-Path $PkgDir "hacash_sdk_bg.wasm") (Join-Path $WalletPkg "hacash_sdk_bg.wasm") -Force

function Get-Sha256Hex([string]$Path) {
    (Get-FileHash -Path $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

$jsHash = Get-Sha256Hex (Join-Path $PkgDir "hacash_sdk.js")
$wasmHash = Get-Sha256Hex (Join-Path $PkgDir "hacash_sdk_bg.wasm")
$manifest = @{ "hacash_sdk.js" = $jsHash; "hacash_sdk_bg.wasm" = $wasmHash } | ConvertTo-Json -Compress
$manifest | Set-Content -Encoding UTF8 (Join-Path $PkgDir "integrity.json")
$manifest | Set-Content -Encoding UTF8 (Join-Path $WalletPkg "integrity.json")

Write-Host "[3/3] Copy mainnet config template beside binary ..." -ForegroundColor Yellow
Copy-Item (Join-Path $Root "hacash_mainnet_hip25.config.ini.example") (Join-Path $ReleaseDir "hacash_mainnet_hip25.config.ini.example") -Force

Write-Host ""
Write-Host "OK Release ready:" -ForegroundColor Green
Write-Host "  Binary: $ReleaseDir\hacash.exe"
Write-Host "  WASM:   $PkgDir\"
Write-Host "  Config: $ReleaseDir\hacash_mainnet_hip25.config.ini.example"
Write-Host ""
Write-Host "Next: scripts\START_MAINNET_WALLET.bat" -ForegroundColor Yellow