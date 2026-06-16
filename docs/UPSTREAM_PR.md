# Pull Request — HIP-25 HACD Staking (upstream `hacash/rust`)

**Use this as the PR title and description** when opening:

https://github.com/hacash/rust/compare/main...Moskyera:rust:hip-25-staking?expand=1

---

## PR Title

```
HIP-25: HACD staking (kinds 34/35), WASM wallet, mainnet security hardening
```

---

## PR Description

### Summary

This PR adds **HIP-25 HACD staking** to the Hacash Rust fullnode:

- On-chain actions **DiamondStake (34)** and **DiamondUnstake (35)**
- Global reward index pool, **10% inscription protocol fee redirect** (v2; no transfer-fee redirect), idle pool burn, min stake / cooldown
- **Local WASM wallet** at `/hip25/wallet` (client-side signing only on mainnet)
- **Security hardening** for mainnet RPC (loopback default, rate limits, origin checks, no server-side secrets on `chain_id=0`)

### Audit status

Five independent security reviews on branch `hip-25-staking` @ `a798094`:

| Audit | Result |
|-------|--------|
| Auth & secrets | PASS |
| Staking economics | PASS |
| API & DoS | PASS |
| Tx pipeline | PASS |
| WASM wallet | PASS |

24 staking unit tests pass. CI: `.github/workflows/hip25-ci.yml`.

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
```

### Documentation

- Community review guide: `docs/HIP25_COMMUNITY_REVIEW.md`
- HIP-25 spec: https://github.com/hacash/hip25 (if applicable)

### Checklist

- [x] Staking consensus + tests
- [x] Mainnet config guards
- [x] WASM wallet (local signing)
- [x] RPC security middleware
- [x] 5 security audits PASS
- [ ] **Maintainer:** set agreed `staking_activation_height` before merge to mainnet release branch

### Request for community

Please review staking economics (`src/mint/operate/staking.rs`), RPC surface (`src/server/security.rs`), and wallet flow (`wallet/hip25/index.html`). Feedback welcome before fork activation height is announced on live mainnet.

---

**Fork:** https://github.com/Moskyera/rust/tree/hip-25-staking  
**Tag:** `v0.1.0-hip25-mainnet`