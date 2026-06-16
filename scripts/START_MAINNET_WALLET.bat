@echo off
title HIP-25 Mainnet Wallet Launcher
setlocal EnableDelayedExpansion

set "ROOT=%~dp0.."
set "BINDIR=%ROOT%\target\release"
if not exist "%BINDIR%\hacash.exe" set "BINDIR=%ROOT%\target\debug"

cd /d "%BINDIR%"

if not exist hacash.exe (
    echo.
    echo  hacash.exe not found. Build release first:
    echo    cd %ROOT%
    echo    powershell -ExecutionPolicy Bypass -File scripts\BUILD_MAINNET_RELEASE.ps1
    echo.
    pause
    exit /b 1
)

if not exist pkg\hacash_sdk.js (
    echo.
    echo  WASM SDK missing in %BINDIR%\pkg
    echo  Run: powershell -ExecutionPolicy Bypass -File scripts\BUILD_MAINNET_RELEASE.ps1
    echo.
    pause
    exit /b 1
)

if not exist pkg\integrity.json (
    echo.
    echo  WARNING: pkg\integrity.json missing — run BUILD_MAINNET_RELEASE.ps1 for hash verification.
    echo.
)

taskkill /IM hacash.exe /F >nul 2>&1
timeout /t 2 /nobreak >nul

if not exist hacash.config.ini (
    echo Creating hacash.config.ini from mainnet template...
    copy /Y "%ROOT%\hacash_mainnet_hip25.config.ini.example" hacash.config.ini >nul
) else (
    echo Using existing hacash.config.ini ^(NOT overwritten — edit for your node^).
)

echo.
echo  MAINNET SAFETY CHECK:
echo  - chain_id must be 0
echo  - hip25_testnet_seed must be false
echo  - listen_host must be 127.0.0.1
echo  - allow_public_rpc must be false
echo  - staking_activation_height = agreed fork height ^(NOT 0 for new genesis^)
echo  - data_dir must point to SYNCED mainnet chain ^(never delete^)
echo.
pause

echo [1/2] Starting HIP25-MAINNET-FULLNODE...
start "HIP25-MAINNET-FULLNODE" cmd /k "cd /d %cd% && title HIP25-MAINNET-FULLNODE && hacash.exe"

echo [2/2] Waiting for RPC on port 8081 ^(max 120s^)...
set /a tries=0
:waitrpc
set /a tries+=1
if !tries! gtr 120 goto rpcfail
timeout /t 1 /nobreak >nul
powershell -NoProfile -Command "try { $r = Invoke-RestMethod 'http://127.0.0.1:8081/query/latest' -TimeoutSec 3; if ($r.chain_id -eq 0 -and $r.hip25_dev -eq $false) { exit 0 } else { Write-Host 'WARN: chain_id=' $r.chain_id ' hip25_dev=' $r.hip25_dev; exit 2 } } catch { exit 1 }" >nul 2>&1
if errorlevel 1 goto waitrpc
if errorlevel 2 (
    echo       RPC up but NOT mainnet mode — check hacash.config.ini
    goto openwallet
)
echo       Mainnet RPC ready ^(chain_id=0^).

:openwallet
start http://127.0.0.1:8081/hip25/wallet

echo.
echo  ========================================
echo   MAINNET WALLET: http://127.0.0.1:8081/hip25/wallet
echo  ========================================
echo.
echo  KEEP OPEN: HIP25-MAINNET-FULLNODE window
echo  NO poworker / NO testnet seed on mainnet
echo  Sign stake/unstake locally via WASM only
echo.
pause
exit /b 0

:rpcfail
echo.
echo  RPC did not start on :8081. Check HIP25-MAINNET-FULLNODE window.
echo  Ensure data_dir has synced mainnet chain data.
echo.
pause
exit /b 1