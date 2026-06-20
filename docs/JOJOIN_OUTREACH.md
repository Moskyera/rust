# Maintainer outreach — copy-paste for Discord / GitHub

**Discord dev channel:** https://discord.com/channels/757976908653920299/802807729584209920  
**GitHub PR:** https://github.com/hacash/rust/pull/13

---

## Short version (Discord)

```
@jojoin — HIP-25 update from the Moskyera reference implementation.

We completed a working HACD staking prototype (actions 34/35) on the legacy rust fork, with v3 supply-neutral economics: redirect the existing DiamondMint 10% miner share to stakers — no new HAC minting, inscription fees stay 100% burn. 24 tests, 5 audits PASS.

We understand canonical mainnet work is fullnodedev → fullnode, not hacash/rust. We are NOT asking for a rust repo merge.

Questions:
1. Is HIP-25 staking on the roadmap for fullnodedev?
2. Should activation be a separate fork height or after Istanbul (765432)?
3. Would you prefer we port to fullnodedev ourselves (PR) or wait for core team?

Spec draft: https://github.com/Moskyera/rust/blob/hip-25-staking/docs/HIP25_SPEC.md
Reference branch: https://github.com/Moskyera/rust/tree/hip-25-staking

Happy to adjust economics/parameters from your HIP-11 / HIP-2 guidance. Thanks for the review on PR #13.
```

---

## Long version (GitHub comment on PR #13)

```markdown
## Reference implementation status (not a merge request for legacy rust)

Thanks again for the HIP-11 / HIP-2 economics note. Following that feedback we moved to **v3 supply-neutral economics** (`eeee5da`):

- **Funding:** 10% of DiamondMint `fee_got` (miner share) → staking pool when active
- **Inscription fees:** 100% burn (unchanged from pre-HIP-25)
- **Supply:** zero-sum vs current miner payout — no extra circulating HAC from inscription redirect

We now treat this PR as a **reference implementation + community review**, not a request to merge into legacy `hacash/rust`. Per [hacash.org/development](https://www.hacash.org/development), canonical mainnet path is **fullnodedev → fullnode**.

**Formal spec draft:** [HIP25_SPEC.md](https://github.com/Moskyera/rust/blob/hip-25-staking/docs/HIP25_SPEC.md)

**Questions for maintainers:**

1. Should HIP-25 be ported to `fullnodedev`? We can open a PR there if useful.
2. Preferred activation: separate `staking_activation_height` or post-Istanbul (765432)?
3. Any parameter changes required before port (min stake 25714 blocks, cooldown 864, idle sweep 1008)?

We will not set a mainnet activation height without community consensus and maintainer agreement.

Tag: `v0.1.0-hip25-reference` on the fork for a frozen reference point.
```