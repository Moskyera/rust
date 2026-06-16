# Opens GitHub compare page to create upstream PR (hacash/rust <- Moskyera/rust hip-25-staking)
$Url = "https://github.com/hacash/rust/compare/main...Moskyera:rust:hip-25-staking?expand=1"
Write-Host "Opening upstream PR compare:" $Url -ForegroundColor Cyan
Write-Host "Copy PR body from: docs\UPSTREAM_PR.md" -ForegroundColor Yellow
Start-Process $Url