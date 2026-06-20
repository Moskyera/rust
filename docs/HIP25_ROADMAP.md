# HIP-25 Roadmap — Reference → Mainline

## Current position

| Item | Status |
|------|--------|
| Reference impl (Moskyera/rust `hip-25-staking`) | ✅ `eeee5da` — v3 economics, 24 tests, 5 audits |
| Discussion PR (hacash/rust #13) | ✅ Open — community feedback |
| Formal HIP doc | ✅ `docs/HIP25_SPEC.md` (submit to hacash/paper) |
| fullnodedev port | ⏳ Awaiting maintainer alignment |
| Mainnet activation height | ⏳ Not set |

## What changed (June 2026)

We no longer treat merge into **legacy** `hacash/rust` as the mainnet path. Official development moved to **fullnodedev → fullnode** ([hacash.org/development](https://www.hacash.org/development)). Our work remains a **reference implementation** and specification baseline.

## Phase 1 — Stabilize reference (done / this repo)

- [x] HIP-25 v3 supply-neutral economics
- [x] WASM wallet + mainnet RPC hardening
- [x] HIP-2 v2.1 mortgage (actions 15/16) on same branch
- [x] Formal spec: `HIP25_SPEC.md`
- [x] Tag: `v0.1.0-hip25-reference`

## Phase 2 — Governance (next)

1. Submit `HIP25_SPEC.md` to [hacash/paper](https://github.com/hacash/paper) (HIP table entry).
2. Maintainer outreach (jojoin) — see `JOJOIN_OUTREACH.md`.
3. Community post — see `HIP25_COMMUNITY_POST.md`.
4. Keep PR #13 open as **discussion + audit record**; description reflects v3 + reference status.

## Phase 3 — fullnodedev port (after maintainer yes)

Port order (estimated):

1. **Protocol** — action kinds 34/35, diamond status 4/5, state types (`mint/`, `protocol/`)
2. **Consensus** — `staking_apply_stake/unstake`, block-close distribution, v3 fee redirect in chain insert
3. **RPC** — `/query/staking/*` in `server/`
4. **Tests** — port `staking_tests` to fullnodedev testkit
5. **Wallet** — integrate with official WASM SDK or hacash/wallet

**Do not start Phase 3 without explicit maintainer direction** — Istanbul upgrade (height 765432) is the current priority.

## Phase 4 — Mainnet activation

- Community ≥30 day notice
- Agreed `staking_activation_height`
- Release via hacash/fullnode
- Optional: separate height from Istanbul or bundled — maintainer decision

## Assets preserved

| Asset | Location |
|-------|----------|
| Spec | `docs/HIP25_SPEC.md` |
| Economics v3 | `docs/HIP25_ECONOMICS_V3.md` |
| Community review | `docs/HIP25_COMMUNITY_REVIEW.md` |
| Code | https://github.com/Moskyera/rust/tree/hip-25-staking |
| Tag | `v0.1.0-hip25-reference` |