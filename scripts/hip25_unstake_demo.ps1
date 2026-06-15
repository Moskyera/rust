# HIP-25 unstake demo: short periods (min_stake=5, cooldown=3) on fresh testnet chain
param(
    [string]$Base = "http://127.0.0.1:8083",
    [string]$DataDir = "hacash_hip25_unstake_demo",
    [string]$SeedAddress = "1Do17BuqMj5N4EZRuquXtoCCHFZpQoHyc2",
    [string]$SeedPrikey = "95f8f5960f8d12471419d76716677cbe1764b628cf11213845b6d917a9f98657",
    [string]$Diamond = "MEKUIA"
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

function Submit-StakeTx($kind, $diamonds) {
    $ts = 1718496000
    $txJson = "{`"main_address`":`"$SeedAddress`",`"timestamp`":$ts,`"fee`":`"0:247`",`"actions`":[{`"kind`":$kind,`"diamonds`":`"$diamonds`"}]}"
    $built = Invoke-HacPost "create/transaction" $txJson @{ action = "true"; signature = "true" }
    if ($built.ret -ne 0) { throw "create: $($built.err)" }
    $txBytes = [byte[]]::new($built.body.Length / 2)
    for ($i = 0; $i -lt $txBytes.Length; $i++) { $txBytes[$i] = [Convert]::ToByte($built.body.Substring($i * 2, 2), 16) }
    $signed = Invoke-HacPost "util/transaction/sign" $txBytes @{ prikey = $SeedPrikey; signature = "true"; action = "true" }
    if ($signed.ret -ne 0) { throw "sign: $($signed.err)" }
    $signedBytes = [byte[]]::new($signed.body.Length / 2)
    for ($i = 0; $i -lt $signedBytes.Length; $i++) { $signedBytes[$i] = [Convert]::ToByte($signed.body.Substring($i * 2, 2), 16) }
    $sub = Invoke-HacPost "submit/transaction" $signedBytes @{}
    if ($sub.ret -ne 0) { throw "submit: $($sub.err)" }
    return $sub.hash
}

function Wait-Status($diamond, $want, $timeout = 120) {
    for ($i = 0; $i -lt $timeout; $i++) {
        $st = Invoke-HacGet "query/staking/status" @{ diamond = $diamond }
        if ($st.status -eq $want) { return $st }
        Start-Sleep -Seconds 2
    }
    throw "Timeout: $diamond not $want"
}

Write-Host "=== HIP-25 UNSTAKE DEMO ===" -ForegroundColor Cyan

if (Test-Path $DataDir) { Remove-Item -Recurse -Force $DataDir }

$cfg = @"
[default]
data_dir = $DataDir
[server]
enable = true
listen = 8083
[node]
listen = 3338
not_find_nodes = true
boots =
[mint]
chain_id = 1
staking_activation_height = 1
hip25_testnet_seed = true
hip25_testnet_seed_password = hip25test
hip25_testnet_demo_periods = true
[miner]
enable = true
reward = $SeedAddress
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
Start-Sleep -Seconds 2

$node = Start-Process -FilePath $Exe -WorkingDirectory $BinDir -PassThru
for ($i = 0; $i -lt 30; $i++) {
    try { $null = Invoke-HacGet "query/latest"; break } catch { Start-Sleep -Seconds 1 }
}
$miner = Start-Process -FilePath $Exe -ArgumentList "poworker" -WorkingDirectory $BinDir -PassThru
Start-Sleep -Seconds 2

try {
    for ($i = 0; $i -lt 60; $i++) {
        if ((Invoke-HacGet "query/latest").height -ge 1) { break }
        Start-Sleep -Seconds 2
    }

    Write-Host "[1] Stake $Diamond..." -ForegroundColor Yellow
    $h1 = Submit-StakeTx 34 $Diamond
    Write-Host "    tx $h1"
    $st = Wait-Status $Diamond "Staked"
    Write-Host "    badge: Staked at height $($st.stake_height) min_unstake=$($st.min_unstake_height)" -ForegroundColor Green

    Write-Host "[2] Wait until min unstake height..." -ForegroundColor Yellow
    for ($i = 0; $i -lt 90; $i++) {
        $h = (Invoke-HacGet "query/latest").height
        $st = Invoke-HacGet "query/staking/status" @{ diamond = $Diamond }
        if ($h -ge $st.min_unstake_height) { Write-Host "    height=$h >= min_unstake=$($st.min_unstake_height)" -ForegroundColor Green; break }
        Start-Sleep -Seconds 2
    }

    Write-Host "[3] Unstake $Diamond (kind 35)..." -ForegroundColor Yellow
    $h2 = Submit-StakeTx 35 $Diamond
    Write-Host "    tx $h2"
    $st2 = Wait-Status $Diamond "Cooldown"
    Write-Host "    badge: Cooldown unlock_height=$($st2.unlock_height)" -ForegroundColor Magenta

    Write-Host "[4] Wait cooldown (3 blocks)..." -ForegroundColor Yellow
    for ($i = 0; $i -lt 90; $i++) {
        $h = (Invoke-HacGet "query/latest").height
        if ($h -ge $st2.unlock_height) { break }
        Start-Sleep -Seconds 2
    }
    $st3 = Wait-Status $Diamond "Available"
    Write-Host "    badge: Available (unstake complete)" -ForegroundColor Green

    $events = Invoke-HacGet "query/staking/events" @{ from = 0; limit = 20 }
    Write-Host "[5] Events: $($events.total) total" -ForegroundColor Green
    foreach ($ev in $events.events) {
        if ($ev.literal -eq $Diamond) { Write-Host "    $($ev.kind) h=$($ev.height)" }
    }

    Write-Host ""
    Write-Host "UNSTAKE DEMO PASSED. Wallet: $Base/hip25/wallet" -ForegroundColor Green
} finally {
    if ($miner -and -not $miner.HasExited) { Stop-Process -Id $miner.Id -Force -EA SilentlyContinue }
    if ($node -and -not $node.HasExited) { Stop-Process -Id $node.Id -Force -EA SilentlyContinue }
}