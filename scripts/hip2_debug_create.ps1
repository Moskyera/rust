$BinDir = "C:\Users\KQHEX\Documents\hacash-rust\target\debug"
Set-Location $BinDir
Get-Process hacash -EA SilentlyContinue | Stop-Process -Force
Remove-Item -Recurse -Force hacash_hip2_demo_data -EA SilentlyContinue
@'
[default]
data_dir = hacash_hip2_demo_data
[server]
enable = true
listen = 8083
listen_host = 127.0.0.1
[mint]
chain_id = 1
staking_activation_height = 1
hip25_testnet_seed = true
hip25_testnet_seed_password = hip25test
mortgage_activation_height = 1
hip2_testnet_demo_periods = true
[miner]
enable = true
reward = 1Do17BuqMj5N4EZRuquXtoCCHFZpQoHyc2
[node]
listen = 3339
not_find_nodes = true
'@ | Set-Content hacash.config.ini
$env:RUST_BACKTRACE = "1"
$Exe = Join-Path $BinDir "hacash.exe"
$node = Start-Process -FilePath $Exe -WorkingDirectory $BinDir -PassThru -RedirectStandardError "$BinDir\crash.log"
Start-Sleep 2
Start-Process -FilePath $Exe -ArgumentList poworker -WorkingDirectory $BinDir | Out-Null
for ($i = 0; $i -lt 30; $i++) {
    try {
        $h = (Invoke-RestMethod "http://127.0.0.1:8083/query/latest" -TimeoutSec 2).height
        if ($h -ge 1) { break }
    } catch {}
    Start-Sleep 1
}
$txJson = '{"main_address":"1Do17BuqMj5N4EZRuquXtoCCHFZpQoHyc2","timestamp":1718496000,"fee":"0:247","actions":[{"kind":15,"lending_id":"4801000000000000000000000032","diamonds":"WTYUIA","loan":"1:250","borrow_period":5}]}'
try {
    $r = Invoke-RestMethod -Uri "http://127.0.0.1:8083/create/transaction?action=true&signature=true" -Method POST -Body $txJson -ContentType "application/json"
    Write-Host "create OK ret=$($r.ret)"
} catch {
    Write-Host "create ERR: $($_.Exception.Message)"
}
Start-Sleep 1
if ($node.HasExited) {
    Write-Host "NODE CRASHED"
    Get-Content "$BinDir\crash.log" -Tail 50
} else {
    Write-Host "node alive"
    Stop-Process -Id $node.Id -Force
}