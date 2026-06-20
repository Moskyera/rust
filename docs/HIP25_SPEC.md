# HIP-25: HACD Staking (Pure Lock-and-Earn)

**Status:** Reference implementation complete · Formal HIP submission draft  
**Authors:** Moskyera community implementation  
**Reference code:** https://github.com/Moskyera/rust/tree/hip-25-staking @ `eeee5da`  
**Economics version:** v3 (supply-neutral)  
**Intended upstream:** [hacash/fullnodedev](https://github.com/hacash/fullnodedev) per [HIP-12](https://github.com/hacash/doc/blob/main/HIP/development/HIP-12_Hacash_development_workflow_and_code_permission.pdf)

---

## Abstract

HIP-25 introduces on-chain **HACD staking**: holders lock diamonds to earn a pro-rata share of HAC redirected from the existing **10% DiamondMint miner fee** (`fee_got`). No new HAC is minted. Inscription protocol fees remain **100% burn** (unchanged from pre-HIP-25). Staking complements — does not replace — [HIP-2](https://hacashtalk.com/t/diamond-mortgage-loan-proposal/117) mortgage lending.

---

## Motivation

1. **Store-of-value utility** for HACD without coinbase IOU issuance (contrast HIP-2).
2. **Supply-neutral yield** — redirect existing miner-share, not burn-bound inscription fees (v3 addresses HIP-11 concerns raised in [PR #13](https://github.com/hacash/rust/pull/13)).
3. **Commitment** — minimum stake period reduces mercenary stake/unstake around mint events.
4. **Mutual exclusion** with mortgaged diamonds (status 2/3) and inscription on locked diamonds.

---

## On-chain actions

| Kind | Name | Payload |
|------|------|---------|
| 34 | `DiamondStake` | `DiamondNameListMax200` |
| 35 | `DiamondUnstake` | `DiamondNameListMax200` |

Both require main-address signature. Up to 200 diamonds per action.

---

## Diamond status

| Status | Value | Meaning |
|--------|-------|---------|
| Normal | 1 | Transfer / inscribe allowed |
| Staked | 4 | Locked, earning rewards |
| Staking cooldown | 5 | Unstake requested; rewards fixed; unlock at `unlock_height` |

Staked and cooldown diamonds cannot transfer or receive inscriptions.

---

## Timing parameters

| Parameter | Blocks | ~Duration |
|-----------|--------|-----------|
| `MIN_STAKE_BLOCKS` | 25,714 | ~90 days |
| `COOLDOWN_BLOCKS` | 864 | ~3 days |
| `STAKING_POOL_SWEEP_BLOCKS` | 1,008 | idle pool burn delay |

Unstake allowed only after `stake_height + MIN_STAKE_BLOCKS`.  
HACD returns to liquid balance at `unstake_height + COOLDOWN_BLOCKS`.

Testnet may use compressed periods via `hip25_testnet_demo_periods` (mainnet: **panic** if enabled with `chain_id=0`).

---

## Economics v3 (supply-neutral)

### Fee redirect

When staking is active at block height `H ≥ activation_height`:

1. `DiamondMint` (kind 4) transactions use `burn_90` fee split.
2. The **10% miner share** (`fee_got`) deposits into `reward_pool_zhu` instead of the block miner.
3. Inscription actions (32/33) do **not** fund the pool — **100% burn** as today.
4. HAC transfer fees follow existing burn/miner rules (no staking redirect).

### Reward distribution

- Global `reward_index` increases per staked share each block close.
- Stakers claim accrued HAC on unstake (fixed `pending_reward` during cooldown).
- If `total_staked_shares == 0` for `STAKING_POOL_SWEEP_BLOCKS` consecutive blocks, undistributed pool burns (`hacd_bid_burn_zhu` counter).

### Comparison

| Model | HAC source | Supply vs baseline |
|-------|------------|-------------------|
| HIP-2 mortgage | Coinbase IOU | Issuance on loan; burn on repay |
| HIP-25 v2 (superseded) | Inscription protocol fees | More circulating HAC |
| **HIP-25 v3** | DiamondMint miner 10% | **Unchanged total issuance** |

See `HIP25_ECONOMICS_V3.md` for implementation notes.

---

## Global state

`StakingGlobal` (consensus):

- `activation_height`
- `reward_pool_zhu`, `reward_index`, `total_staked_shares`
- `cumulative_deposit_zhu`, `cumulative_paid_zhu`, `cumulative_pool_burned_zhu`
- `idle_pool_blocks`, `event_log_tail`

Per-diamond `StakingRecord` and per-address stake index as in reference impl.

---

## Activation

- Config: `staking_activation_height` in `[mint]` (0 = disabled).
- Requires **≥30 days** community notice before mainnet height is set.
- Nodes must upgrade before activation height; no effect until then.

---

## RPC (reference)

`GET /query/staking/global`:

```json
{
  "economics_version": "v3",
  "fee_sources": "hacd_mint_miner_share",
  "fee_share_percent": 10
}
```

---

## Security (reference impl)

- 24 staking unit tests (`cargo test staking_tests`)
- 5 independent audits PASS @ `a798094`
- Mainnet: loopback RPC default, no server-side `prikey` on `chain_id=0`

---

## Relationship to other HIPs

| HIP | Relationship |
|-----|--------------|
| HIP-1 | Bid fee destruction unchanged (90% burn on mint) |
| HIP-2 | Complementary; mutual exclusion per diamond |
| HIP-11 | v3 avoids redirecting burn-bound HAC |
| HIP-15 | Inscription burn unchanged under v3 |
| HIP-12 | Port target: `fullnodedev`, not legacy `hacash/rust` |

---

## Reference implementation disclaimer

The working prototype lives on **legacy** [hacash/rust](https://github.com/hacash/rust) architecture (fork: [Moskyera/rust](https://github.com/Moskyera/rust)). Canonical mainnet integration requires porting to [hacash/fullnodedev](https://github.com/hacash/fullnodedev) and release via [hacash/fullnode](https://github.com/hacash/fullnode/releases).

**Open PR (discussion):** https://github.com/hacash/rust/pull/13

---

## Changelog

| Version | Change |
|---------|--------|
| v1 | Initial stake/unstake + global index |
| v2 | Inscription fee redirect only; idle pool burn |
| v3 | Supply-neutral: DiamondMint miner-share redirect; inscription 100% burn |