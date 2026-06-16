# HIP-2 v2.1 — Community Review & Voting Guide

**Proposal:** Merge HIP-2 HACD system mortgage (actions 15/16) into Hacash Rust fullnode alongside HIP-25 staking.  
**Fork:** https://github.com/Moskyera/rust/tree/hip-25-staking  
**Economics version:** `v2.1` (1% origination, 3% flat APR, 3-period grace, 103% auction floor)  
**Audit status:** 5 independent security audits — **PASS** @ `1c96a73` (see [HIP2_SECURITY_AUDITS.md](./HIP2_SECURITY_AUDITS.md))

---

## What to review

| Area | Key paths |
|------|-----------|
| Mortgage consensus | `src/mint/operate/diamond_lending.rs`, `src/mint/action/diamond_lending.rs` |
| Constants / state | `src/mint/component/diamond_lending.rs` (`GlobalMortgageState`, `diamond_syslend`) |
| RPC | `src/server/rpc/mortgage.rs`, `src/server/rpc/supply.rs` |
| WASM wallet | `wallet/hip25/index.html`, `src/sdk/web/transfer.rs` (`hacd_mortgage_open/redeem`) |
| Config guards | `src/config/mint.rs`, `hacash_mainnet_hip25.config.ini.example` |
| Unit tests | `mortgage_tests` in `diamond_lending.rs` (15 cases) |
| E2E / wire tests | `src/mint/operate/diamond_lending_e2e.rs` (9 cases) |

---

## Economics summary (v2.1)

| Parameter | Value |
|-----------|-------|
| Origination | **1%** of principal → burn |
| Grace | **0%** ransom for first **3** periods (~3.5 months) |
| Early private | **0.1%/period** after grace until midpoint |
| Main rate | **3% APR flat** (`apr_bps=300`); `borrow_period` sets window length only |
| Auction floor | **103%** of principal |
| Period | 10_000 blocks mainnet; 10 blocks with `hip2_testnet_demo_periods` |

**Redeem phases:** Private (0→T) → Public (T→2T) → Dutch auction (>2T).

---

## Security audit summary

Full reports: **[HIP2_SECURITY_AUDITS.md](./HIP2_SECURITY_AUDITS.md)**

| Audit | Scope | Result |
|-------|-------|--------|
| #1 | Auth, secrets, RPC | **PASS** |
| #2 | Mortgage economics | **PASS** |
| #3 | API, DoS | **PASS** |
| #4 | Tx pipeline, wire compat | **PASS** |
| #5 | WASM wallet | **PASS** |

**Mainnet defaults:** `mortgage_activation_height=0` (disabled), `chain_id=0`, `listen_host=127.0.0.1`, `allow_public_rpc=false`, client WASM signing only.

---

## How to test locally

### Run all mortgage + staking tests
```powershell
cargo test mortgage_
cargo test staking_tests
```
Expected: **24** mortgage + **24** staking = **48** passing tests.

### Rebuild WASM (after SDK changes)
```powershell
powershell -ExecutionPolicy Bypass -File scripts\build_wallet_sdk.ps1
```

### Testnet wallet demo
```cmd
scripts\START_WALLET.bat
```
Opens http://127.0.0.1:8083/hip25/wallet — mortgage panel (actions 15/16) + staking (34/35).

### Query RPC (node running)
```
GET /query/mortgage/global
GET /query/mortgage/contract?id=<hex>&redeemer=<addr>&height=<n>
GET /query/supply   # includes mortgage IOU / burn fields
```

---

## Community voting checklist

Reviewers / voters should confirm:

- [ ] **Consensus:** open/redeem rules match HIP-2 v2.1 (origination burn, grace, flat APR, auction floor 103%).
- [ ] **IOU cap:** `mortgage_max_outstanding_zhu` enforced at open; governance sets cap before activation.
- [ ] **HIP-25 mutex:** staked HACD cannot be mortgaged; mortgaged HACD cannot be staked (status 2/3 vs 4/5).
- [ ] **Mainnet safety:** `hip2_testnet_demo_periods` panics at startup when `chain_id=0`.
- [ ] **Wallet:** password never sent to RPC; only signed `tx_body` submitted.
- [ ] **Tests:** `cargo test mortgage_` and `cargo test staking_tests` pass (CI).
- [ ] **Fork height:** `mortgage_activation_height` set only after ≥30 day community notice (separate from HIP-25 staking activation).

---

## HIP-25 interaction

HIP-25 staking is unchanged. Mortgage and staking are mutually exclusive per diamond. Global supply RPC includes mortgage outstanding IOU and cumulative burns.

---

## Upstream merge

**→ [docs/UPSTREAM_PR.md](./UPSTREAM_PR.md)** (copy-paste PR description)

**One-click compare:**  
https://github.com/hacash/rust/compare/main...Moskyera:rust:hip-25-staking?expand=1

---

## Contact / feedback

Comment on the fork PR with:

1. Audit finding ID (e.g. `HIP2-A3-001`)
2. Severity (Critical / High / Medium / Low / Info)
3. Repro steps

**Target:** community consensus before `mortgage_activation_height` is set on live mainnet.