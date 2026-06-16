# HIP-2 v2.1 live demo: open mortgage (15) -> quote -> redeem (16) on testnet
param(
    [string]$Base = "http://127.0.0.1:8083",
    [string]$DataDir = "hacash_hip2_demo_data",
    [string]$SeedPassword = "hip25test",
    [string]$SeedAddress = "1Do17BuqMj5N4EZRuquXtoCCHFZpQoHyc2",
    [string]$SeedPrikey = "95f8f5960f8d12471419d76716677cbe1764b628cf11213845b6d917a9f98657",
    [string]$Diamond = "WTYUIA",
    [string]$LendIdHex = "4801000000000000000000000032",
    [string]$Loan = "1:250",
    [int]$BorrowPeriod = 5
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$BinDir = Join-Path $Root "target\debug"
$Exe = Join-Path $BinDir "hacash.exe"
Set-Location $BinDir

function Invoke-HacGet($Path, [hashtable]$Query = @{}) {
    $qs = ($Query.GetEnumerator() | ForEach-Object { "$($_.Key)=$($_.Value)" }) -join "&"
    $uri = "$Base/$Path" + $(if ($qs) { "?$qs" } else { "" })
    return Invoke-RestMethod -Uri $uri -TimeoutSec 30
}

function Invoke-HacPost($Path, $Body, [hashtable]$Query = @{}) {
    $qs = ($Query.GetEnumerator() | ForEach-Object { "$($_.Key)=$($_.Value)" }) -join "&"
    $uri = "$Base/$Path" + $(if ($qs) { "?$qs" } else { "" })
    if ($Body -is [byte[]]) {
        return Invoke-RestMethod -Uri $uri -Method POST -Body $Body -ContentType "application/octet-stream" -TimeoutSec 60
    }
    if ($Body -is [string] -and $Body.TrimStart().StartsWith("{")) {
        return Invoke-RestMethod -Uri $uri -Method POST -Body $Body -ContentType "application/json" -TimeoutSec 60
    }
    return Invoke-RestMethod -Uri $uri -Method POST -Body $Body -ContentType "application/octet-stream" -TimeoutSec 60
}

function Wait-RpcReady() {
    for ($i = 0; $i -lt 40; $i++) {
        try { $null = Invoke-HacGet "query/latest"; return } catch { Start-Sleep -Seconds 1 }
    }
    throw "RPC not ready"
}

function Wait-Height($Target) {
    for ($i = 0; $i -lt 120; $i++) {
        $latest = Invoke-HacGet "query/latest"
        if ($latest.height -ge $Target) { return $latest.height }
        Start-Sleep -Seconds 2
    }
    throw "Timeout height >= $Target"
}

function Submit-SignedTx($TxJson) {
    try {
        $built = Invoke-HacPost "create/transaction" $TxJson @{ action = "true"; signature = "true" }
    } catch {
        throw "create/transaction HTTP: $($_.Exception.Message)"
    }
    if ($built.ret -ne 0) { throw "create/transaction: $($built.err)" }
    $txBody = [string]$built.body
    $signBody = (@{ prikey = $SeedPrikey; tx_body = $txBody } | ConvertTo-Json -Compress)
    try {
        $signed = Invoke-HacPost "util/transaction/sign" $signBody @{ signature = "true"; action = "true" }
    } catch {
        throw "transaction/sign HTTP: $($_.Exception.Message)"
    }
    if ($signed.ret -ne 0) { throw "transaction/sign: $($signed.err)" }
    $signedBody = [string]$signed.body
    $signedBytes = [byte[]]::new($signedBody.Length / 2)
    for ($i = 0; $i -lt $signedBytes.Length; $i++) {
        $signedBytes[$i] = [Convert]::ToByte($signedBody.Substring($i * 2, 2), 16)
    }
    try {
        $sub = Invoke-HacPost "submit/transaction" $signedBytes @{}
    } catch {
        throw "submit/transaction HTTP: $($_.Exception.Message)"
    }
    if ($sub.ret -ne 0) { throw "submit/transaction: $($sub.err)" }
    return $sub.hash
}

Write-Host "=== HIP-2 mortgage live demo ===" -ForegroundColor Cyan

if (-not (Test-Path $Exe)) { throw "Build first: cargo build" }

Get-Process hacash -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1
if (Test-Path $DataDir) { Remove-Item -Recurse -Force $DataDir }

$cfg = @"
[default]
data_dir = $DataDir
[server]
enable = true
listen = 8083
listen_host = 127.0.0.1
allow_public_rpc = false
recent_blocks = false
average_fee_purity = false
[node]
listen = 3339
not_find_nodes = true
boots =
[mint]
chain_id = 1
staking_activation_height = 1
hip25_testnet_seed = true
hip25_testnet_seed_password = $SeedPassword
hip25_testnet_demo_periods = true
mortgage_activation_height = 1
mortgage_max_outstanding_zhu = 0
hip2_testnet_demo_periods = true
[miner]
enable = true
reward = $SeedAddress
message = hip2demo
[diamondminer]
enable = false
"@
Set-Content "hacash.config.ini" $cfg
@"
[default]
connect = 127.0.0.1:8083
supervene = 4
nonce_max = 4294967295
notice_wait = 3
"@ | Set-Content "poworker.config.ini"

$node = Start-Process -FilePath $Exe -WorkingDirectory $BinDir -PassThru
Wait-RpcReady
$miner = Start-Process -FilePath $Exe -ArgumentList "poworker" -WorkingDirectory $BinDir -PassThru
Start-Sleep -Seconds 2

try {
    $h = Wait-Height 1
    Write-Host "[OK] block height $h (seeded 5 HACD + smelt 100 mei each)" -ForegroundColor Green

    $mg0 = Invoke-HacGet "query/mortgage/global"
    Write-Host "[OK] mortgage global: apr=$($mg0.apr_bps)bps period=$($mg0.period_blocks)blocks v=$($mg0.economics_version)" -ForegroundColor Green

    $bal0 = Invoke-HacGet "query/balance" @{ address = $SeedAddress }
    Write-Host "[..] balance before: $($bal0.list[0].hacash)" -ForegroundColor DarkGray

    $ts = 1718496000
    $openJson = "{`"main_address`":`"$SeedAddress`",`"timestamp`":$ts,`"fee`":`"0:247`",`"actions`":[{`"kind`":15,`"lending_id`":`"$LendIdHex`",`"diamonds`":`"$Diamond`",`"loan`":`"$Loan`",`"borrow_period`":$BorrowPeriod}]}"
    Write-Host "[..] submit mortgage OPEN (kind 15)..." -ForegroundColor Yellow
    $openHash = Submit-SignedTx $openJson
    Write-Host "[OK] mortgage OPEN tx: $openHash" -ForegroundColor Green

    $confirmed = $false
    for ($i = 0; $i -lt 60; $i++) {
        $mg = Invoke-HacGet "query/mortgage/global"
        if ($mg.active_contracts -ge 1) { $confirmed = $true; break }
        Start-Sleep -Seconds 2
    }
    if (-not $confirmed) { throw "mortgage contract not confirmed on chain" }

    $mg1 = Invoke-HacGet "query/mortgage/global"
    Write-Host "[OK] active=$($mg1.active_contracts) IOU=$($mg1.outstanding_ioo_zhu) orig_burn=$($mg1.cumulative_origination_burn_zhu)" -ForegroundColor Green

    $height = (Invoke-HacGet "query/latest").height
    $quote = Invoke-HacGet "query/mortgage/contract" @{
        id = $LendIdHex
        redeemer = $SeedAddress
        height = $height
    }
    Write-Host "[OK] quote: phase=$($quote.redeem_phase) min_ransom=$($quote.min_ransom) principal=$($quote.loan_principal)" -ForegroundColor Green

    $bal1 = Invoke-HacGet "query/balance" @{ address = $SeedAddress }
    Write-Host "[..] balance after open: $($bal1.list[0].hacash) (loan +$Loan, origination 1%)" -ForegroundColor DarkGray

    $ransom = $quote.min_ransom
    $redeemJson = "{`"main_address`":`"$SeedAddress`",`"timestamp`":$ts,`"fee`":`"0:247`",`"actions`":[{`"kind`":16,`"lending_id`":`"$LendIdHex`",`"ransom`":`"$ransom`"}]}"
    Write-Host "[..] submit mortgage REDEEM (kind 16) ransom=$ransom..." -ForegroundColor Yellow
    $redeemHash = Submit-SignedTx $redeemJson
    Write-Host "[OK] mortgage REDEEM tx: $redeemHash" -ForegroundColor Green

    $redeemed = $false
    for ($i = 0; $i -lt 60; $i++) {
        $mg = Invoke-HacGet "query/mortgage/global"
        if ($mg.active_contracts -eq 0) { $redeemed = $true; break }
        Start-Sleep -Seconds 2
    }
    if (-not $redeemed) { throw "mortgage not redeemed on chain" }

    $mg2 = Invoke-HacGet "query/mortgage/global"
    Write-Host "[OK] redeemed: active=$($mg2.active_contracts) IOU=$($mg2.outstanding_ioo_zhu)" -ForegroundColor Green

    $bal2 = Invoke-HacGet "query/balance" @{ address = $SeedAddress }
    Write-Host "[..] balance after redeem: $($bal2.list[0].hacash)" -ForegroundColor DarkGray

    try {
        $supply = Invoke-HacGet "query/supply"
        Write-Host "[OK] supply mortgage_burn=$($supply.mortgage_origination_burn_zhu) v=$($supply.mortgage_economics_version)" -ForegroundColor Green
    } catch {
        Write-Host "[WARN] supply query skipped: $($_.Exception.Message)" -ForegroundColor Yellow
    }

    Write-Host ""
    Write-Host "HIP-2 DEMO PASSED - open, quote, redeem OK" -ForegroundColor Green
    Write-Host "Wallet UI: $Base/hip25/wallet (password: $SeedPassword)" -ForegroundColor Cyan
} finally {
    if ($miner -and -not $miner.HasExited) { Stop-Process -Id $miner.Id -Force -ErrorAction SilentlyContinue }
    if ($node -and -not $node.HasExited) { Stop-Process -Id $node.Id -Force -ErrorAction SilentlyContinue }
}