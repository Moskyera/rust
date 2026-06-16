$BinDir = "C:\Users\KQHEX\Documents\hacash-rust\target\debug"
$Exe = Join-Path $BinDir "hacash.exe"
$Base = "http://127.0.0.1:8083"
$Prikey = "95f8f5960f8d12471419d76716677cbe1764b628cf11213845b6d917a9f98657"
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
allow_public_rpc = false
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
@'
[default]
connect = 127.0.0.1:8083
supervene = 4
'@ | Set-Content poworker.config.ini
$node = Start-Process -FilePath $Exe -WorkingDirectory $BinDir -PassThru
Start-Sleep 3
$miner = Start-Process -FilePath $Exe -ArgumentList poworker -WorkingDirectory $BinDir -PassThru
for ($i = 0; $i -lt 40; $i++) {
    try { if ((Invoke-RestMethod "$Base/query/latest").height -ge 1) { break } } catch {}
    Start-Sleep 1
}
$txJson = '{"main_address":"1Do17BuqMj5N4EZRuquXtoCCHFZpQoHyc2","timestamp":1718496000,"fee":"0:247","actions":[{"kind":15,"lending_id":"4801000000000000000000000032","diamonds":"WTYUIA","loan":"1:250","borrow_period":5}]}'
Write-Host "1 create..."
$built = Invoke-RestMethod -Uri "$Base/create/transaction?action=true&signature=true" -Method POST -Body $txJson -ContentType "application/json"
Write-Host "2 sign..."
$signJson = (@{ prikey = $Prikey; tx_body = $built.body } | ConvertTo-Json -Compress)
$signed = Invoke-RestMethod -Uri "$Base/util/transaction/sign?signature=true&action=true" -Method POST -Body $signJson -ContentType "application/json"
Write-Host "3 submit..."
$hex = $signed.body
$bytes = [byte[]]::new($hex.Length / 2)
for ($i = 0; $i -lt $bytes.Length; $i++) { $bytes[$i] = [Convert]::ToByte($hex.Substring($i * 2, 2), 16) }
$sub = Invoke-RestMethod -Uri "$Base/submit/transaction" -Method POST -Body $bytes -ContentType "application/octet-stream"
Write-Host "OPEN tx=$($sub.hash)"
Start-Sleep 3
$mg = Invoke-RestMethod "$Base/query/mortgage/global"
Write-Host "active=$($mg.active_contracts) IOU=$($mg.outstanding_ioo_zhu)"
$q = Invoke-RestMethod "$Base/query/mortgage/contract?id=4801000000000000000000000032&redeemer=1Do17BuqMj5N4EZRuquXtoCCHFZpQoHyc2"
Write-Host "quote phase=$($q.redeem_phase) ransom=$($q.min_ransom)"
$redeemJson = "{`"main_address`":`"1Do17BuqMj5N4EZRuquXtoCCHFZpQoHyc2`",`"timestamp`":1718496000,`"fee`":`"0:247`",`"actions`":[{`"kind`":16,`"lending_id`":`"4801000000000000000000000032`",`"ransom`":`"$($q.min_ransom)`"}]}"
$rb = Invoke-RestMethod -Uri "$Base/create/transaction?action=true&signature=true" -Method POST -Body $redeemJson -ContentType "application/json"
$rs = Invoke-RestMethod -Uri "$Base/util/transaction/sign?signature=true&action=true" -Method POST -Body ((@{ prikey = $Prikey; tx_body = $rb.body } | ConvertTo-Json -Compress)) -ContentType "application/json"
$rh = $rs.body
$rbts = [byte[]]::new($rh.Length / 2)
for ($i = 0; $i -lt $rbts.Length; $i++) { $rbts[$i] = [Convert]::ToByte($rh.Substring($i * 2, 2), 16) }
$sub2 = Invoke-RestMethod -Uri "$Base/submit/transaction" -Method POST -Body $rbts -ContentType "application/octet-stream"
Write-Host "REDEEM tx=$($sub2.hash)"
Start-Sleep 3
$mg2 = Invoke-RestMethod "$Base/query/mortgage/global"
Write-Host "DONE active=$($mg2.active_contracts)"
Stop-Process -Id $miner.Id,$node.Id -Force -EA SilentlyContinue