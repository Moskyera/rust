# Pull Request — HIP-25 HACD Staking (upstream `hacash/rust`)

**Use this as the PR title and description** when opening:

https://github.com/hacash/rust/compare/main...Moskyera:rust:hip-25-staking?expand=1

---

## PR Title

```
HIP-25: HACD staking (kinds 34/35) + HIP-2 v2 mortgage (kinds 15/16), WASM wallet, mainnet security hardening
```

---

## PR Description

### Summary

This PR adds **HIP-25 HACD staking** and **HIP-2 v2.1 system mortgage** to the Hacash Rust fullnode:

**HIP-25 (actions 34/35)**
- Global reward index pool, **10% inscription protocol fee redirect** (v2), idle pool burn, min stake / cooldown
- 24 staking unit tests

**HIP-2 v2.1 (actions 15/16)**
- HAC IOU mortgage against bid-burn collateral: **1% origination burn**, **3% flat APR**, **3-period grace**, **103% auction floor**
- Global `GlobalMortgageState`, per-contract `diamond_syslend` map, supply + mortgage RPC
- 15 unit + 9 E2E mortgage tests (full tx pipeline, Go wire compat)
- WASM: `hacd_mortgage_open` / `hacd_mortgage_redeem`; wallet UI mortgage panel

**Shared**
- **Local WASM wallet** at `/hip25/wallet` (client-side signing only on mainnet)
- **Security hardening** for mainnet RPC (loopback default, rate limits, origin checks, no server-side secrets on `chain_id=0`)
- Staking ↔ mortgage mutual exclusion per diamond

### Audit status

**HIP-25** — 5 audits @ `a798094`: all **PASS** (see `docs/HIP25_COMMUNITY_REVIEW.md`)

**HIP-2 v2.1** — 5 audits on branch `hip-25-staking`: all **PASS** (see `docs/HIP2_SECURITY_AUDITS.md`)

| Audit | HIP-25 | HIP-2 |
|-------|--------|-------|
| Auth & secrets | PASS | PASS |
| Economics | PASS | PASS |
| API & DoS | PASS | PASS |
| Tx pipeline | PASS | PASS |
| WASM wallet | PASS | PASS |

**48** consensus tests pass (`staking_tests` + `mortgage_`). CI: `.github/workflows/hip25-ci.yml`.

### Mainnet safety controls

- `chain_id=0` blocks server-side `prikey` signing RPCs
- `hip25_testnet_seed` / `demo_periods` **panic** if enabled with mainnet `chain_id`
- Default `listen_host=127.0.0.1`, `allow_public_rpc=false`
- P2P tx ingress: signature + `try_execute_tx` before relay
- Query-string secret rejection on signing routes

### How to test

**Testnet demo:**
```cmd
scripts\START_WALLET.bat
```

**Mainnet wallet model:**
```powershell
scripts\BUILD_MAINNET_RELEASE.ps1
scripts\START_MAINNET_WALLET.bat
```

### Breaking changes

None for existing mainnet nodes until `staking_activation_height` is reached.

### Configuration

Copy `hacash_mainnet_hip25.config.ini.example` → `hacash.config.ini`:

```ini
[mint]
chain_id = 0
staking_activation_height = <AGREED_FORK_HEIGHT>
hip25_testnet_seed = false
hip25_testnet_demo_periods = false
mortgage_activation_height = 0
mortgage_max_outstanding_zhu = 0
hip2_testnet_demo_periods = false
```

### Documentation

- HIP-25 community review: `docs/HIP25_COMMUNITY_REVIEW.md`
- HIP-2 community review: `docs/HIP2_COMMUNITY_REVIEW.md`
- HIP-2 spec / economics: `docs/HIP2_MORTGAGE_V2.md`
- HIP-2 security audits (5× PASS): `docs/HIP2_SECURITY_AUDITS.md`

### Checklist

- [x] HIP-25 staking consensus + tests (24)
- [x] HIP-2 mortgage consensus + unit/E2E tests (24)
- [x] Mainnet config guards (HIP-25 + HIP-2 dev flags)
- [x] WASM wallet (stake / unstake / mortgage)
- [x] RPC security middleware + mortgage/supply queries
- [x] 10 security audits PASS (5 HIP-25 + 5 HIP-2)
- [ ] **Maintainer:** set agreed `staking_activation_height` before merge to mainnet release branch
- [ ] **Governance:** set `mortgage_activation_height` + IOU cap after separate ≥30 day notice

### Request for community

Please review staking economics (`src/mint/operate/staking.rs`), mortgage economics (`src/mint/operate/diamond_lending.rs`), RPC surface (`src/server/security.rs`, `src/server/rpc/mortgage.rs`), and wallet flow (`wallet/hip25/index.html`). Feedback welcome before fork activation heights are announced on live mainnet.

---

**Fork:** https://github.com/Moskyera/rust/tree/hip-25-staking  
**Tag:** `v0.1.0-hip25-mainnet`