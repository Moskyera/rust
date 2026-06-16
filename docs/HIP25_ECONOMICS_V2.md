# HIP-25 v2 Economics (response to HIP-11 / jojoin review)

**Context:** [hacash/rust#13](https://github.com/hacash/rust/pull/13#issuecomment-4714088187) — any redistribution of HAC that would otherwise burn requires long-term community consideration (HIP-11). HIP-2 (mortgage + repay principal+interest) is the reference model for HAC liquidity from HACD.

## What changed (v2)

| Item | v1 | **v2** |
|------|-----|--------|
| Fee sources | 13% inscription protocol + 13% HACD transfer fees | **10% inscription protocol fees only** |
| Transfer fees | Partial redirect | **Unchanged** (full burn/miner split) |
| Idle pool | Accumulates forever if no stakers | **Burned** after `1008` consecutive blocks with zero stakers |
| Supply stats | Pool balance only | `cumulative_deposit_zhu`, `cumulative_paid_zhu`, `cumulative_pool_burned_zhu` |

## What did NOT change

- Stake / unstake mechanics, cooldown, min stake age
- No coinbase minting (not HIP-2-style IOU issuance)
- Mutual exclusion with HIP-2 mortgage diamond status
- Mainnet activation still requires agreed `staking_activation_height` + community notice

## HIP-2 comparison

| | HIP-2 mortgage | HIP-25 v2 staking |
|--|----------------|-------------------|
| HAC source | Coinbase IOU (= bid burn restored) | Redirected inscription protocol burn share |
| Repayment | Principal + interest | None (fee-funded yield) |
| Mainnet | Never activated | Pending HIP-11-style vote |

HIP-25 v2 is a **simpler, lower-impact** complement — not a replacement for HIP-2.

## RPC

`GET /query_global_staking` returns v2 fields: `fee_share_percent`, `fee_sources`, cumulative counters, `pool_sweep_blocks`.