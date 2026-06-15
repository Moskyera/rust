# HIP-25 wallet E2E: verify seeded HACD list + stake one diamond + badge labels
param(
    [string]$Base = "http://127.0.0.1:8083",
    [string]$DataDir = "hacash_hip25_wallet_data",
    [string]$SeedPassword = "hip25test",
    [string]$SeedAddress = "1Do17BuqMj5N4EZRuquXtoCCHFZpQoHyc2",
    [string]$SeedPrikey = "95f8f5960f8d12471419d76716677cbe1764b628cf11213845b6d917a9f98657",
    [string]$StakeDiamond = "HXVMEK"
)

$ErrorActionPreference = "Stop"
$ExpectedDiamonds = @("WTYUIA", "HXVMEK", "VMEKBS", "UIASHX", "MEKUIA")
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
    $bytes = if ($Body -is [byte[]]) { $Body } else { [System.Text.Encoding]::UTF8.GetBytes([string]$Body) }
    $req = [System.Net.HttpWebRequest]::Create($uri)
    $req.Method = "POST"
    $req.ContentType = "application/octet-stream"
    $req.ContentLength = $bytes.Length
    $stream = $req.GetRequestStream()
    $stream.Write($bytes, 0, $bytes.Length)
    $stream.Close()
    $resp = $req.GetResponse()
    $reader = New-Object System.IO.StreamReader($resp.GetResponseStream())
    return ($reader.ReadToEnd() | ConvertFrom-Json)
}

function Wait-RpcReady() {
    for ($i = 0; $i -lt 30; $i++) {
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

Write-Host "=== HIP-25 wallet E2E (5 HACD + labels) ===" -ForegroundColor Cyan

if (Test-Path $DataDir) { Remove-Item -Recurse -Force $DataDir }

$cfg = @"
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
hip25_testnet_seed_password = $SeedPassword
[miner]
enable = true
reward = $SeedAddress
message = hip25wallet
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

Get-Process hacash -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1

$node = Start-Process -FilePath $Exe -WorkingDirectory $BinDir -PassThru
Wait-RpcReady
$miner = Start-Process -FilePath $Exe -ArgumentList "poworker" -WorkingDirectory $BinDir -PassThru
Start-Sleep -Seconds 2

try {
    Wait-Height 1 | Out-Null

    $bal = Invoke-HacGet "query/balance" @{ address = $SeedAddress; diamonds = "true" }
    $entry = $bal.list[0]
    $raw = $entry.diamonds -replace "\s", ""
    $found = @()
    for ($i = 0; $i -lt $raw.Length; $i += 6) { $found += $raw.Substring($i, 6) }
    $found = $found | Sort-Object
    $expected = $ExpectedDiamonds | Sort-Object
    if (($found -join ",") -ne ($expected -join ",")) {
        throw "Expected diamonds $($expected -join ',') but got $($found -join ',')"
    }
    Write-Host "[OK] balance lists 5 seeded HACD" -ForegroundColor Green

    foreach ($d in $ExpectedDiamonds) {
        $st = Invoke-HacGet "query/staking/status" @{ diamond = $d }
        if ($st.status -ne "Available") { throw "$d expected Available, got $($st.status)" }
    }
    Write-Host "[OK] all 5 badges = Available before stake" -ForegroundColor Green

    $ts = 1718496000
    $txJson = "{`"main_address`":`"$SeedAddress`",`"timestamp`":$ts,`"fee`":`"0:247`",`"actions`":[{`"kind`":34,`"diamonds`":`"$StakeDiamond`"}]}"
    $built = Invoke-HacPost "create/transaction" $txJson @{ action = "true"; signature = "true" }
    if ($built.ret -ne 0) { throw "create/transaction: $($built.err)" }
    $txBody = [string]$built.body
    $txBytes = [byte[]]::new($txBody.Length / 2)
    for ($i = 0; $i -lt $txBytes.Length; $i++) {
        $txBytes[$i] = [Convert]::ToByte($txBody.Substring($i * 2, 2), 16)
    }
    $signed = Invoke-HacPost "util/transaction/sign" $txBytes @{ prikey = $SeedPrikey; signature = "true"; action = "true" }
    if ($signed.ret -ne 0) { throw "transaction/sign: $($signed.err)" }
    $signedBody = [string]$signed.body
    $signedBytes = [byte[]]::new($signedBody.Length / 2)
    for ($i = 0; $i -lt $signedBytes.Length; $i++) {
        $signedBytes[$i] = [Convert]::ToByte($signedBody.Substring($i * 2, 2), 16)
    }
    $sub = Invoke-HacPost "submit/transaction" $signedBytes @{}
    if ($sub.ret -ne 0) { throw "submit/transaction: $($sub.err)" }
    Write-Host "[OK] staked $StakeDiamond tx=$($sub.hash)" -ForegroundColor Green

    $staked = $false
    for ($i = 0; $i -lt 90; $i++) {
        $st = Invoke-HacGet "query/staking/status" @{ diamond = $StakeDiamond }
        if ($st.status -eq "Staked") { $staked = $true; break }
        Start-Sleep -Seconds 2
    }
    if (-not $staked) { throw "$StakeDiamond not Staked after submit" }

    $summary = Invoke-HacGet "query/staking/summary" @{ address = $SeedAddress }
    if ($summary.staked_count -lt 1) { throw "summary staked_count < 1" }
    Write-Host "[OK] $StakeDiamond Staked; summary staked=$($summary.staked_count)" -ForegroundColor Green

    $other = $ExpectedDiamonds | Where-Object { $_ -ne $StakeDiamond } | Select-Object -First 1
    $st2 = Invoke-HacGet "query/staking/status" @{ diamond = $other }
    if ($st2.status -ne "Available") { throw "$other should stay Available" }
    Write-Host "[OK] $other still Available (label isolation)" -ForegroundColor Green

    $wallet = Invoke-WebRequest -Uri "$Base/hip25/wallet" -UseBasicParsing
    if ($wallet.StatusCode -ne 200) { throw "wallet page failed" }
    Write-Host "[OK] GET /hip25/wallet" -ForegroundColor Green

    Write-Host ""
    Write-Host "Wallet E2E passed. Open $Base/hip25/wallet" -ForegroundColor Green
} finally {
    if ($miner -and -not $miner.HasExited) { Stop-Process -Id $miner.Id -Force -ErrorAction SilentlyContinue }
    if ($node -and -not $node.HasExited) { Stop-Process -Id $node.Id -Force -ErrorAction SilentlyContinue }
}