# Pull Request — HIP-25 Reference Implementation

**Discussion PR (legacy repo):** https://github.com/hacash/rust/pull/13

**Canonical mainnet path:** [hacash/fullnodedev](https://github.com/hacash/fullnodedev) → [hacash/fullnode](https://github.com/hacash/fullnode/releases) per [HIP-12](https://github.com/hacash/doc/blob/main/HIP/development/HIP-12_Hacash_development_workflow_and_code_permission.pdf).

This document is the PR body for **community review**. It is **not** a request to merge into inactive `hacash/rust` as the mainnet delivery vehicle.

---

## PR Title

```
HIP-25 reference: HACD staking (34/35), v3 supply-neutral economics, WASM wallet
```

---

## PR Description

### Summary

**Reference implementation** of HIP-25 HACD staking on legacy Hacash Rust architecture ([Moskyera/rust](https://github.com/Moskyera/rust) fork). Intended for spec validation, audits, and future port to **fullnodedev**.

**HIP-25 (actions 34/35)**
- Global reward index pool, **v3 supply-neutral economics**
- **10% DiamondMint miner share** (`fee_got`) → staking pool when active
- Inscription fees: **100% burn** (unchanged from pre-HIP-25)
- Idle pool burn after 1008 blocks with zero stakers
- Min stake ~90d (`25714` blocks), cooldown ~3d (`864` blocks)
- 24 staking unit tests

**HIP-2 v2.1 (actions 15/16)** — on same branch, complementary
- HAC IOU mortgage: 1% origination burn, 3% flat APR, mutual exclusion with staking
- 15 unit + 9 E2E mortgage tests

**Shared**
- Local WASM wallet at `/hip25/wallet` (client-side signing on mainnet)
- Mainnet RPC hardening (loopback default, rate limits, no server `prikey` on `chain_id=0`)

### Economics v3 (current)

| Source | v2 (superseded) | **v3** |
|--------|-----------------|--------|
| Staking pool | 10% inscription protocol fees | **10% DiamondMint miner share** |
| Inscription | Partial redirect to pool | **100% burn** |
| Supply vs baseline | Higher circulating HAC | **Unchanged** (reallocation only) |

Details: `docs/HIP25_ECONOMICS_V3.md` · Formal spec: `docs/HIP25_SPEC.md`

### Audit status

| Audit | HIP-25 @ `a798094` | HIP-2 |
|-------|-------------------|-------|
| Auth & secrets | PASS | PASS |
| Economics | PASS | PASS |
| API & DoS | PASS | PASS |
| Tx pipeline | PASS | PASS |
| WASM wallet | PASS | PASS |

**48** consensus tests pass. CI: `.github/workflows/hip25-ci.yml`.

### Mainnet safety

- `chain_id=0` blocks server-side `prikey` signing RPCs
- `hip25_testnet_seed` / demo flags **panic** on mainnet `chain_id`
- Default `listen_host=127.0.0.1`, `allow_public_rpc=false`
- P2P tx ingress: signature + `try_execute_tx` before relay

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

None until `staking_activation_height` is reached on a network running this consensus.

### Configuration

`hacash_mainnet_hip25.config.ini.example`:

```ini
[mint]
chain_id = 0
staking_activation_height = 0
hip25_testnet_seed = false
hip25_testnet_demo_periods = false
```

Activation height set only after community + maintainer consensus (≥30 day notice).

### Documentation

- **HIP-25 spec:** `docs/HIP25_SPEC.md`
- **Roadmap:** `docs/HIP25_ROADMAP.md`
- Community review: `docs/HIP25_COMMUNITY_REVIEW.md`
- Economics v3: `docs/HIP25_ECONOMICS_V3.md`

### Request for maintainers

1. Should HIP-25 be ported to **fullnodedev**?
2. Separate fork height or post-Istanbul (765432)?
3. Parameter changes before port?

We treat this PR as **discussion + reference**, not legacy-rust mainnet merge.

---

**Fork:** https://github.com/Moskyera/rust/tree/hip-25-staking  
**Reference tag:** `v0.1.0-hip25-reference`  
**Spec:** https://github.com/Moskyera/rust/blob/hip-25-staking/docs/HIP25_SPEC.md