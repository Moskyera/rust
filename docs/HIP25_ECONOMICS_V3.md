# HIP-25 v3 Economics (supply-neutral staking)

**Context:** Community feedback — staking yield should not reduce HAC burn vs baseline. Redirect the existing HACD mint miner-share instead of inscription protocol fees.

## v3 rule

| Item | v2 | **v3** |
|------|-----|--------|
| Staking pool funding | 10% of inscription protocol fees (less burn) | **10% of HACD mint bid fee** (`fee_got` on action 4) |
| Inscription fees | 10% → pool, 90% burn when staking active | **100% burn** (unchanged from pre-HIP-25) |
| Miner on DiamondMint blocks | Receives bid 10% via `fee_got` | **0%** when staking active — share goes to pool |
| Total issuance vs pre-HIP-25 | Higher circulating HAC (less inscription burn) | **Unchanged** — reallocation only |
| Idle pool (no stakers) | Burn after 1008 blocks | Same |

## Mechanism

1. `DiamondMint` txs use `burn_90`: ~90% of bid fee burns, ~10% is `fee_got`.
2. On block close (`insert.rs`), when staking is active, `fee_got` from **DiamondMint-only** txs deposits into `reward_pool_zhu` instead of the block miner.
3. `staking_on_block_close` distributes pool to stakers by shares; idle pool sweeps to burn (`hacd_bid_burn_zhu` counter).
4. Inscription actions (32/33) no longer call `staking_deposit_fee`.

## RPC

`GET /query/staking/global` returns:

- `economics_version`: `"v3"`
- `fee_sources`: `"hacd_mint_miner_share"`
- `fee_share_percent`: `10`

## Comparison

| | HIP-2 mortgage | HIP-25 v3 staking |
|--|----------------|-------------------|
| HAC source | Coinbase IOU (bid collateral) | HACD mint miner-share redirect |
| Supply impact | IOU issuance model | Zero-sum vs current miner payout |

See `HIP25_ECONOMICS_V2.md` for superseded v2 inscription-fee model.