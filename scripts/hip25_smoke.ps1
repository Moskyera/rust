# HIP-25 testnet smoke test — run while fullnode listens on 8083
# Note: hacash loads hacash.config.ini from target\debug\ (exe dir). Sync before run:
#   Copy-Item ..\hacash.config.ini .\hacash.config.ini
$Base = "http://127.0.0.1:8083"
$Rqid = "hip25smoke$(Get-Random)"

function Get-HacQuery($path) {
    $uri = "$Base/query/$path" + $(if ($path -match '\?') { "&" } else { "?" }) + "rqid=$Rqid"
    Invoke-RestMethod -Uri $uri -TimeoutSec 10
}

Write-Host "=== HIP-25 smoke test ===" -ForegroundColor Cyan

try {
    $global = Get-HacQuery "staking/global"
    if ($global.ret -ne 0) { throw "staking/global ret=$($global.ret)" }
    Write-Host "[OK] staking/global"
    Write-Host "     activation_height=$($global.activation_height) shares=$($global.total_staked_shares) pool=$($global.reward_pool_pending_zhu) events=$($global.event_count)"

    $latest = Get-HacQuery "latest"
    Write-Host "[OK] latest height=$($latest.height)"

    $events = Get-HacQuery "staking/events?from=0&limit=5"
    Write-Host "[OK] staking/events total=$($events.total)"

    $addr = "12vi7DEZjh6KrK5PVmmqSgvuJPCsZMmpfi"
    $bal = Get-HacQuery "balance?address=$addr"
    Write-Host "[OK] balance $addr = $($bal.balance)"

    Write-Host ""
    Write-Host "Smoke RPC checks passed." -ForegroundColor Green
} catch {
    Write-Host "[FAIL] $_" -ForegroundColor Red
    exit 1
}