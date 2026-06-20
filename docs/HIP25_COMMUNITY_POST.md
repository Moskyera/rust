# Community post (English) — copy-paste

**Suggested channels:** HacashTalk, Discord, X thread, PR #13

---

## Post

**HIP-25 HACD Staking — reference implementation ready, seeking mainline path**

We've completed a full **HIP-25** prototype: on-chain stake/unstake (actions 34/35), supply-neutral **v3 economics**, WASM wallet, 24 tests, and 5 security audits (all PASS).

### What it does

- Lock HACD to earn a share of HAC from the **existing 10% DiamondMint miner fee** — not new coinbase minting.
- Inscription fees stay **100% burn** (same as today).
- Min stake ~90 days, cooldown ~3 days — reduces mercenary staking around mint events.
- Cannot stake mortgaged diamonds (HIP-2 compatible).

### v3 economics (why it changed)

Early feedback (including @jojoin on PR #13) noted that redirecting burn-bound HAC needs HIP-11-level care. **v3** fixes this: we only redirect the miner share that already goes to the block miner today. Total HAC issuance vs baseline is unchanged — reallocation, not inflation.

### Important: this is a reference impl, not legacy rust merge

Official Hacash development has moved to **fullnodedev → fullnode** ([developer guide](https://www.hacash.org/development)). The legacy `hacash/rust` repo is inactive. Our code is a **tested reference** for community review and a future port — not a claim that PR #13 alone activates mainnet.

### Links

- **Spec:** https://github.com/Moskyera/rust/blob/hip-25-staking/docs/HIP25_SPEC.md
- **Branch:** https://github.com/Moskyera/rust/tree/hip-25-staking
- **Discussion PR:** https://github.com/hacash/rust/pull/13
- **Economics v3:** https://github.com/Moskyera/rust/blob/hip-25-staking/docs/HIP25_ECONOMICS_V3.md
- **Try testnet wallet:** clone fork → `scripts\START_WALLET.bat` → http://127.0.0.1:8083/hip25/wallet

### What we need from the community

1. Review the spec and economics (especially vs HIP-2 mortgage and dynamic-staking proposals).
2. Feedback on min stake / cooldown durations.
3. Support for formal HIP submission to hacash/paper.
4. Maintainer direction on **fullnodedev port** timing (before/after Istanbul @ 765432).

We will **not** announce a mainnet fork height without ≥30 days notice and maintainer agreement.

Thank you to everyone who reviewed PR #13 — the discussion is moving the design in the right direction.

— Moskyera / HIP-25 reference implementation