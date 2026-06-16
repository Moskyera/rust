# HIP-25 Public Testnet — Boot Instructions

Local dev/testnet for **Pure HACD Staking** (HIP-25). Not a public internet testnet; run on your machine and open the wallet in a browser.

## Prerequisites

- Windows or Linux
- Rust toolchain (`cargo`, `rustc`)
- Built `hacash` binary: `cargo build` from repo root

Config is loaded from the **directory containing `hacash.exe`**, typically `target/debug/`.

## Quick start

```powershell
cd target\debug
copy ..\..\hacash.config.ini.example hacash.config.ini
# Edit hacash.config.ini: hip25_testnet_seed = true, miner enable = true (see example)
.\hacash.exe
```

In a second terminal (same `target\debug` folder):

```powershell
.\hacash.exe poworker
```

`poworker.config.ini` must contain `connect = 127.0.0.1:8083`.

## Seeded test account (dev only)

When `hip25_testnet_seed = true` in `[mint]`, block **1** seeds:

| Field | Value |
|-------|-------|
| Password | `hip25test` |
| Address | `1Do17BuqMj5N4EZRuquXtoCCHFZpQoHyc2` |
| Private key (hex) | `95f8f5960f8d12471419d76716677cbe1764b628cf11213845b6d917a9f98657` |
| HAC | `11` (11:244) |
| HACD | `WTYUIA`, `HXVMEK`, `VMEKBS`, `UIASHX`, `MEKUIA` |

**Never use this key on mainnet.**

## Wallet UI

Open: **http://127.0.0.1:8083/hip25/wallet**

1. Click **Fill HIP-25 testnet seed**
2. **Load portfolio** — five HACD with badges (`Available` / `Staked` / `Cooldown`)
3. Select diamonds → **Stake selected** or **Unstake selected**
4. **HIP-2 mortgage** — select **Available** HACD → loan auto-fills → **Open mortgage** (action 15); redeem with **Redeem HACD** (action 16)

Tx modes:

- **RPC** — `create/transaction` + `util/transaction/sign` + `submit/transaction` (works out of the box)
- **WASM SDK** — `hacd_stake` / `hacd_unstake` (requires built `pkg/hacash_sdk.js`; see below)

## WASM SDK (optional)

Build (Linux/macOS or Windows with `wasm32-unknown-unknown`):

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --features sdk --target wasm32-unknown-unknown --lib
# Or: wasm-pack build --target web --features sdk
```

Exported functions:

- `hacd_stake(chain_id, password, "WTYUIA,HXVMEK", fee, timestamp)` → signed tx JSON
- `hacd_unstake(chain_id, password, diamonds, fee, timestamp)` → signed tx JSON
- `hacd_mortgage_open(chain_id, password, lending_id_hex, diamonds, loan, borrow_periods, fee, timestamp)` → action 15
- `hacd_mortgage_redeem(chain_id, password, lending_id_hex, ransom, fee, timestamp)` → action 16

Submit `tx_body` hex via `POST /submit/transaction`.

## Automated E2E

From repo root:

```powershell
.\scripts\hip25_smoke.ps1
.\scripts\hip25_live_stake.ps1
.\scripts\hip25_wallet_e2e.ps1
```

## Staking RPC

| Endpoint | Purpose |
|----------|---------|
| `GET /query/staking/status?diamond=WTYUIA` | Per-HACD status label |
| `GET /query/staking/summary?address=…` | Portfolio counts |
| `GET /query/staking/global` | Pool / activation height |
| `GET /query/staking/events?from=0&limit=20` | Stake/unstake events |

Actions: **34** stake, **35** unstake.

## Mortgage RPC (HIP-2 v2.1)

| Endpoint | Purpose |
|----------|---------|
| `GET /query/mortgage/global` | Outstanding IOU, APR, `owner_index_max` (64) |
| `GET /query/mortgage/portfolio?address=…` | Active contracts; `indexed_count`, `owner_index_full` |
| `GET /query/mortgage/principal?diamonds=WTYUIA,HXVMEK` | Loan + origination burn quote |
| `GET /query/mortgage/contract?id=…&redeemer=…` | Min ransom / redemption phase |

Actions: **15** mortgage open, **16** mortgage redeem.

**64-contract limit:** each address may have at most **64** active mortgage contracts indexed on-chain (`MORTGAGE_OWNER_INDEX_MAX`). The wallet and `mortgage/portfolio` RPC surface `owner_index_full` when the limit is reached; opening another contract fails with `mortgage owner contract index full`.

**Testnet economics example:** 1 HACD → loan `1:250`, origination burn ~`1:249` (1%); 2 HACD → loan `2:250`, burn ~`2:248`. Portfolio address and signing password must match.

## Consensus parameters (v1)

- Fee share to staking pool: **10%** (inscription protocol fees only; v2 economics)
- `MIN_STAKE_BLOCKS = 25714`
- `COOLDOWN_BLOCKS = 864`
- `staking_activation_height` in `[mint]` (testnet: `1`)

## One fullnode per data dir

LevelDB locks the data directory. Stop other `hacash.exe` instances before starting a fresh testnet.

## Reference

- HIP: `hacash-hip25/HIP/HIP-25_Pure_HACD_Staking.md`
- Branch: `hip-25-staking` on https://github.com/Moskyera/rust