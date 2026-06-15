# HIP-25 live testnet: mine blocks, stake seeded HACD WTYUIA, verify RPC
param(
    [string]$Base = "http://127.0.0.1:8083",
    [string]$DataDir = "hacash_hip25_live_data",
    [string]$SeedPassword = "hip25test",
    [string]$SeedAddress = "1Do17BuqMj5N4EZRuquXtoCCHFZpQoHyc2",
    [string]$SeedPrikey = "95f8f5960f8d12471419d76716677cbe1764b628cf11213845b6d917a9f98657",
    [int]$MineBlocksBeforeStake = 1,
    [int]$MineBlocksAfterStake = 2
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

function Wait-RpcReady() {
    for ($i = 0; $i -lt 30; $i++) {
        try {
            $null = Invoke-HacGet "query/latest"
            return
        } catch {
            Start-Sleep -Seconds 1
        }
    }
    throw "RPC not ready on $Base"
}

function Wait-Height($Target) {
    for ($i = 0; $i -lt 180; $i++) {
        $latest = Invoke-HacGet "query/latest"
        if ($latest.height -ge $Target) { return $latest.height }
        Start-Sleep -Seconds 2
    }
    throw "Timeout waiting for height >= $Target"
}

Write-Host "=== HIP-25 live stake E2E ===" -ForegroundColor Cyan

# Fresh chain data
if (Test-Path $DataDir) {
    Write-Host "Removing old data dir $DataDir ..."
    Remove-Item -Recurse -Force $DataDir
}

# Sync config beside binary (load_config uses exe directory, not cwd)
$cfg = @"
; HIP-25 live stake run
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
message = hip25testnet

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

Write-Host "Starting fullnode..."
$node = Start-Process -FilePath $Exe -WorkingDirectory $BinDir -PassThru
Wait-RpcReady
Write-Host "[OK] fullnode RPC ready"

Write-Host "Starting poworker..."
$miner = Start-Process -FilePath $Exe -ArgumentList "poworker" -WorkingDirectory $BinDir -PassThru
Start-Sleep -Seconds 2

try {
    Write-Host "Mining $MineBlocksBeforeStake block(s) (genesis init + seed HACD on block 1)..."
    Wait-Height $MineBlocksBeforeStake | Out-Null

    # Submit stake before poworker mines block 2 (avoids mempool / miner_notice stall)
    $ts = 1718496000
    $txJson = "{`"main_address`":`"$SeedAddress`",`"timestamp`":$ts,`"fee`":`"0:247`",`"actions`":[{`"kind`":34,`"diamonds`":`"WTYUIA`"}]}"

    $built = Invoke-HacPost "create/transaction" $txJson @{ action = "true"; signature = "true" }
    if ($built.ret -ne 0) { throw "create/transaction: $($built.err)" }
    Write-Host "[OK] built stake tx hash=$($built.hash)"

    $txBytes = [byte[]]::new($built.body.Length / 2)
    for ($i = 0; $i -lt $txBytes.Length; $i++) {
        $txBytes[$i] = [Convert]::ToByte($built.body.Substring($i * 2, 2), 16)
    }
    $signed = Invoke-HacPost "util/transaction/sign" $txBytes @{ prikey = $SeedPrikey; signature = "true"; action = "true" }
    if ($signed.ret -ne 0) { throw "transaction/sign: $($signed.err)" }
    Write-Host "[OK] signed tx"

    $signedBytes = [byte[]]::new($signed.body.Length / 2)
    for ($i = 0; $i -lt $signedBytes.Length; $i++) {
        $signedBytes[$i] = [Convert]::ToByte($signed.body.Substring($i * 2, 2), 16)
    }
    $submitted = Invoke-HacPost "submit/transaction" $signedBytes @{}
    if ($submitted.ret -ne 0) { throw "submit/transaction: $($submitted.err)" }
    Write-Host "[OK] submitted tx hash=$($submitted.hash)"

    $global = Invoke-HacGet "query/staking/global"
    if ($global.ret -ne 0) { throw "staking/global failed" }
    Write-Host "[OK] activation_height=$($global.activation_height) shares=$($global.total_staked_shares)"

    $bal = Invoke-HacGet "query/balance" @{ address = $SeedAddress; diamonds = "true" }
    $entry = if ($bal.list) { $bal.list[0] } else { $null }
    Write-Host "[OK] seed balance HAC=$($entry.hacash) diamonds=$($entry.diamonds)"
    if ($entry.diamonds.Length -lt 30) {
        throw "Expected 5 seeded HACD (30 chars), got: $($entry.diamonds)"
    }

    Write-Host "Waiting for on-chain stake confirmation..."
    $status = $null
    $lastHeight = -1
    $stuck = 0
    for ($i = 0; $i -lt 120; $i++) {
        try { $null = Invoke-HacGet "query/miner/pending?stuff=true" } catch {}
        $latest = Invoke-HacGet "query/latest"
        $status = Invoke-HacGet "query/staking/status" @{ diamond = "WTYUIA" }
        if ($status.status -eq "Staked") {
            Write-Host "[OK] confirmed at height=$($latest.height)"
            break
        }
        if ($latest.height -eq $lastHeight) {
            $stuck++
            if ($stuck -ge 8) {
                Write-Host "[INFO] mining stalled at height=$lastHeight, restarting poworker..."
                Get-Process hacash -ErrorAction SilentlyContinue |
                    Where-Object { $_.Id -ne $node.Id } |
                    Stop-Process -Force -ErrorAction SilentlyContinue
                Start-Sleep -Seconds 1
                $miner = Start-Process -FilePath $Exe -ArgumentList "poworker" -WorkingDirectory $BinDir -PassThru
                $stuck = 0
            }
        } else {
            $lastHeight = $latest.height
            $stuck = 0
        }
        Start-Sleep -Seconds 2
    }
    if ($status.ret -ne 0) { throw "staking/status: $($status.err)" }
    Write-Host "[OK] WTYUIA status=$($status.status) staker=$($status.staker) stake_height=$($status.stake_height)"

    if ($status.status -ne "Staked") {
        throw "Expected status=Staked, got $($status.status)"
    }

    $events = Invoke-HacGet "query/staking/events" @{ from = 0; limit = 10 }
    Write-Host "[OK] staking events total=$($events.total)"

    Write-Host ""
    Write-Host "Live stake E2E passed." -ForegroundColor Green
} finally {
    if ($miner -and -not $miner.HasExited) { Stop-Process -Id $miner.Id -Force -ErrorAction SilentlyContinue }
    if ($node -and -not $node.HasExited) { Stop-Process -Id $node.Id -Force -ErrorAction SilentlyContinue }
}