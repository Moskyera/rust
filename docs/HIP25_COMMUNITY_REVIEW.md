# HIP-25 — Community Review & Voting Guide

**Proposal:** Merge HIP-25 HACD staking (actions 34/35) + local WASM wallet into Hacash Rust fullnode.  
**Fork:** https://github.com/Moskyera/rust/tree/hip-25-staking  
**Release tag:** `v0.1.0-hip25-mainnet`  
**Audit status:** 5 independent security audits — **PASS** (local mainnet wallet model)

---

## What to review

| Area | Key paths |
|------|-----------|
| Staking consensus | `src/mint/operate/staking.rs`, `src/mint/action/diamond_staking.rs` |
| RPC security | `src/server/security.rs`, `src/server/http/start.rs` |
| WASM wallet | `wallet/hip25/index.html`, `src/sdk/web/transfer.rs` |
| Config guards | `src/config/mint.rs`, `hacash_mainnet_hip25.config.ini.example` |
| Tests | `src/mint/operate/staking.rs` (`staking_tests`, 24 cases) |

---

## Security audit summary (commit `a798094`)

| Audit | Scope | Result |
|-------|-------|--------|
| #1 | Auth, secrets, RPC | **PASS** |
| #2 | Staking economics | **PASS** |
| #3 | API, DoS | **PASS** |
| #4 | Tx pipeline, P2P | **PASS** |
| #5 | WASM wallet | **PASS** |

**Mainnet defaults:** `chain_id=0`, `listen_host=127.0.0.1`, `allow_public_rpc=false`, server-side signing **disabled** on mainnet, client WASM signing only.

---

## How to test locally

### Testnet (quick demo)
```cmd
scripts\START_WALLET.bat
```
Opens http://127.0.0.1:8083/hip25/wallet with HIP-25 dev chain (`chain_id=1`).

### Mainnet wallet (production model)
```powershell
powershell -ExecutionPolicy Bypass -File scripts\BUILD_MAINNET_RELEASE.ps1
scripts\START_MAINNET_WALLET.bat
```
Requires **synced mainnet** `data_dir` — does **not** delete chain data.

---

## Community voting checklist

Reviewers / voters should confirm:

- [ ] **Consensus:** stake/unstake rules match HIP-25 v2 spec (min stake, cooldown, **10% inscription protocol fee only**, idle pool burn, ownership).
- [ ] **Mainnet safety:** dev flags (`hip25_testnet_seed`) panic at startup when `chain_id=0`.
- [ ] **Wallet:** secrets never sent to RPC; only signed `tx_body` submitted.
- [ ] **RPC:** loopback-only by default; public bind requires explicit `allow_public_rpc=true`.
- [ ] **Tests:** `cargo test staking_tests` passes (also run in GitHub Actions).
- [ ] **Fork height:** `staking_activation_height` must be set to **agreed** mainnet activation (≥30 day notice per HIP-25).

---

## Upstream merge

Open PR to official repo using prepared body:

**→ [docs/UPSTREAM_PR.md](./UPSTREAM_PR.md)** (copy-paste PR description)

**One-click compare:**  
https://github.com/hacash/rust/compare/main...Moskyera:rust:hip-25-staking?expand=1

---

## Release artifacts

Tag `v0.1.0-hip25-mainnet` includes:

- Release build scripts (`BUILD_MAINNET_RELEASE.ps1`)
- Mainnet launcher (`START_MAINNET_WALLET.bat`)
- Mainnet config template (`hacash_mainnet_hip25.config.ini.example`)
- CI workflow (`.github/workflows/hip25-ci.yml`)

---

## Contact / feedback

Comment on the fork PR or upstream PR with:

1. Audit finding ID (if any)
2. Severity (Critical / High / Medium / Low)
3. Repro steps

**Target:** community consensus before mainnet fork activation height is set on live network.