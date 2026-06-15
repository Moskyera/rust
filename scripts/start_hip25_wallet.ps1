# Start HIP-25 testnet + open wallet.
# Opens TWO cmd windows (HIP25-FULLNODE + HIP25-POWORKER). Do not close them.
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$BinDir = Join-Path $Root "target\debug"
$Exe = Join-Path $BinDir "hacash.exe"
$SeedAddress = "1Do17BuqMj5N4EZRuquXtoCCHFZpQoHyc2"
$DataDir = "hacash_hip25_demo"
$WalletUrl = "http://127.0.0.1:8083/hip25/wallet"

if (-not (Test-Path $Exe)) {
    Write-Host "Build first: cd $Root ; cargo build" -ForegroundColor Red
    exit 1
}

Set-Location $BinDir
Get-Process hacash -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2

@"
[default]
data_dir = $DataDir
[server]
enable = true
listen = 8083
recent_blocks = false
average_fee_purity = false
[node]
listen = 3338
not_find_nodes = true
boots =
[mint]
chain_id = 1
staking_activation_height = 1
hip25_testnet_seed = true
hip25_testnet_seed_password = hip25test
[miner]
enable = true
reward = $SeedAddress
message = hip25wallet
[diamondminer]
enable = false
"@ | Set-Content "hacash.config.ini"

@"
[default]
connect = 127.0.0.1:8083
supervene = 4
nonce_max = 4294967295
notice_wait = 3
"@ | Set-Content "poworker.config.ini"

Write-Host "Starting fullnode (cmd window: HIP25-FULLNODE)..." -ForegroundColor Cyan
Start-Process cmd.exe -ArgumentList "/k", "cd /d $BinDir && title HIP25-FULLNODE && hacash.exe"

Write-Host "Waiting for RPC..." -ForegroundColor Cyan
$ready = $false
for ($i = 0; $i -lt 40; $i++) {
    try {
        $null = Invoke-RestMethod "http://127.0.0.1:8083/query/latest" -TimeoutSec 2
        $ready = $true
        break
    } catch {
        Start-Sleep -Seconds 1
    }
}

if ($ready) {
    Write-Host "RPC ready." -ForegroundColor Green
} else {
    Write-Host "RPC not ready. Wait 10s then open $WalletUrl" -ForegroundColor Yellow
}

Write-Host "Starting poworker (cmd window: HIP25-POWORKER)..." -ForegroundColor Cyan
Start-Process cmd.exe -ArgumentList "/k", "cd /d $BinDir && title HIP25-POWORKER && hacash.exe poworker"

Start-Sleep -Seconds 2
Start-Process $WalletUrl

Write-Host ""
Write-Host "Wallet: $WalletUrl" -ForegroundColor Yellow
Write-Host "Keep open: HIP25-FULLNODE and HIP25-POWORKER cmd windows." -ForegroundColor White
Write-Host "In wallet: Fill testnet seed -> Load portfolio." -ForegroundColor White