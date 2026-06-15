# Build HIP-25 WASM SDK and copy beside hacash.exe for /pkg/* routes
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$BinDir = Join-Path $Root "target\debug"
$PkgDir = Join-Path $BinDir "pkg"
$WalletPkg = Join-Path $Root "wallet\hip25\pkg"

Write-Host "Building hacash_sdk (wasm32)..." -ForegroundColor Cyan
Set-Location $Root
rustup target add wasm32-unknown-unknown 2>&1 | Out-Null
cargo build --features sdk --target wasm32-unknown-unknown --release --lib

$WasmSrc = Join-Path $Root "target\wasm32-unknown-unknown\release\hacash_sdk.wasm"
if (-not (Test-Path $WasmSrc)) { throw "WASM build failed: $WasmSrc" }

New-Item -ItemType Directory -Force -Path $PkgDir | Out-Null
New-Item -ItemType Directory -Force -Path $WalletPkg | Out-Null

# no-modules: exposes global wasm_bindgen for /hip25/wallet classic script tag
wasm-bindgen $WasmSrc --out-dir $PkgDir --target no-modules --no-typescript

Copy-Item (Join-Path $PkgDir "hacash_sdk.js") (Join-Path $WalletPkg "hacash_sdk.js") -Force
Copy-Item (Join-Path $PkgDir "hacash_sdk_bg.wasm") (Join-Path $WalletPkg "hacash_sdk_bg.wasm") -Force

function Get-Sha256Hex([string]$Path) {
    $hash = Get-FileHash -Path $Path -Algorithm SHA256
    return $hash.Hash.ToLowerInvariant()
}

$jsHash = Get-Sha256Hex (Join-Path $PkgDir "hacash_sdk.js")
$wasmHash = Get-Sha256Hex (Join-Path $PkgDir "hacash_sdk_bg.wasm")
$manifest = @{
    "hacash_sdk.js" = $jsHash
    "hacash_sdk_bg.wasm" = $wasmHash
} | ConvertTo-Json -Compress

$manifest | Set-Content -Encoding UTF8 (Join-Path $PkgDir "integrity.json")
$manifest | Set-Content -Encoding UTF8 (Join-Path $WalletPkg "integrity.json")

Write-Host "OK: pkg copied to" $PkgDir -ForegroundColor Green
Write-Host "     and" $WalletPkg -ForegroundColor Green
Write-Host "integrity.json written (SHA-256 verified on serve)" -ForegroundColor Green
Write-Host "Restart fullnode, then open /hip25/wallet" -ForegroundColor Yellow