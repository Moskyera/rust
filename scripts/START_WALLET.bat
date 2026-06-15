@echo off
title HIP-25 Wallet Launcher
cd /d "%~dp0..\target\debug"

if not exist hacash.exe (
    echo.
    echo  hacash.exe not found. Build first:
    echo    cd C:\Users\KQHEX\Documents\hacash-rust
    echo    cargo build
    echo.
    pause
    exit /b 1
)

taskkill /IM hacash.exe /F >nul 2>&1
timeout /t 2 /nobreak >nul

if exist hacash_hip25_demo (
    echo Removing old chain data ^(fresh testnet seed^)...
    rmdir /s /q hacash_hip25_demo
)

(
echo [default]
echo data_dir = hacash_hip25_demo
echo [server]
echo enable = true
echo listen = 8083
echo listen_host = 127.0.0.1
echo allow_public_rpc = false
echo recent_blocks = false
echo average_fee_purity = false
echo [node]
echo listen = 3338
echo not_find_nodes = true
echo boots =
echo [mint]
echo chain_id = 1
echo staking_activation_height = 1
echo hip25_testnet_seed = true
echo hip25_testnet_seed_password = hip25test
echo hip25_testnet_demo_periods = true
echo [miner]
echo enable = true
echo reward = 1Do17BuqMj5N4EZRuquXtoCCHFZpQoHyc2
echo message = hip25wallet
echo [diamondminer]
echo enable = false
) > hacash.config.ini

(
echo [default]
echo connect = 127.0.0.1:8083
echo supervene = 4
echo nonce_max = 4294967295
echo notice_wait = 3
) > poworker.config.ini

echo.
echo [1/3] Starting HIP25-FULLNODE...
start "HIP25-FULLNODE" cmd /k "cd /d %cd% && title HIP25-FULLNODE && hacash.exe"

echo [2/3] Waiting for RPC (max 60s)...
set /a tries=0
:waitrpc
set /a tries+=1
if %tries% gtr 60 goto rpcfail
timeout /t 1 /nobreak >nul
powershell -NoProfile -Command "try { Invoke-RestMethod 'http://127.0.0.1:8083/query/latest' -TimeoutSec 2 | Out-Null; exit 0 } catch { exit 1 }" >nul 2>&1
if errorlevel 1 goto waitrpc
echo       RPC ready.

echo [3/4] Starting HIP25-POWORKER...
start "HIP25-POWORKER" cmd /k "cd /d %cd% && title HIP25-POWORKER && hacash.exe poworker"

echo [4/4] Waiting for block 1 ^(HACD seed, max 90s^)...
set /a tries=0
:waitblock
set /a tries+=1
if %tries% gtr 90 goto blockwarn
timeout /t 1 /nobreak >nul
powershell -NoProfile -Command "try { $r = Invoke-RestMethod 'http://127.0.0.1:8083/query/latest' -TimeoutSec 2; if ([int]$r.height -ge 1) { exit 0 } else { exit 1 } } catch { exit 1 }" >nul 2>&1
if errorlevel 1 goto waitblock
echo       Block 1 ready — 5 HACD seeded.
goto openwallet
:blockwarn
echo       Block 1 not mined yet — wallet will auto-retry.
:openwallet
start http://127.0.0.1:8083/hip25/wallet

echo.
echo  ========================================
echo   WALLET: http://127.0.0.1:8083/hip25/wallet
echo  ========================================
echo.
echo  KEEP OPEN: HIP25-FULLNODE + HIP25-POWORKER
echo  In wallet: Fill testnet seed -^> Load portfolio
echo.
pause
exit /b 0

:rpcfail
echo.
echo  RPC did not start. Check HIP25-FULLNODE window for errors.
echo.
pause
exit /b 1