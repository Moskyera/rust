# Release v0.1.0-hip25-mainnet

**Tag:** `v0.1.0-hip25-mainnet`  
**Branch:** `hip-25-staking`  
**Security baseline:** commit `a798094` (5 audits PASS)

## Summary

Community-review release for **HIP-25 HACD staking** on Hacash Rust fullnode:

- On-chain stake/unstake (kinds 34/35)
- Local WASM wallet at `/hip25/wallet`
- Mainnet-safe defaults (`chain_id=0`, loopback RPC, no server-side signing)

## Build from source

```powershell
git clone https://github.com/Moskyera/rust.git
cd rust
git checkout hip-25-staking
powershell -ExecutionPolicy Bypass -File scripts\BUILD_MAINNET_RELEASE.ps1
scripts\START_MAINNET_WALLET.bat
```

Requires a **synced mainnet** `data_dir` (see `hacash_mainnet_hip25.config.ini.example`).

## Testnet (no mainnet data)

```cmd
scripts\START_WALLET.bat
```

## Verify

```bash
cargo test staking_tests
```

CI: `.github/workflows/hip25-ci.yml`

## Community

- Review checklist: [HIP25_COMMUNITY_REVIEW.md](./HIP25_COMMUNITY_REVIEW.md)
- Upstream PR text: [UPSTREAM_PR.md](./UPSTREAM_PR.md)
- Compare: https://github.com/hacash/rust/compare/main...Moskyera:rust:hip-25-staking?expand=1

## Before live mainnet fork

1. Community consensus on this release
2. Maintainer sets `staking_activation_height` (≥30 day notice per HIP-25)
3. Merge upstream PR to `hacash/rust`