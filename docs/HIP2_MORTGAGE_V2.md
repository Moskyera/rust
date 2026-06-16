# HIP-2 v2.1 — HACD System Mortgage (HAC IOU)

**Actions:** 15 `MortgageOpen`, 16 `MortgageRedeem`

## Economics (v2.1 — retail-friendly vs v2)

| Parameter | v2 | **v2.1** |
|-----------|-----|----------|
| Origination | 2% | **1%** |
| Early private | 0.25%/period | **0% grace 3 periods**, then **0.1%/period** |
| Committed / public | 0.4% × T (T scales rate) | **3% APR flat** (T = window only) |
| Auction floor | 110% | **103%** |

## Redeem phases

1. **Private** (0 → T periods): mortgagor only; grace + early band, then APR.
2. **Public** (T → 2T periods): anyone; **3% APR** on elapsed blocks.
3. **Auction** (> 2T periods): Dutch decay to **103%** floor over **2T** periods.

**T (borrow_period 1–20)** sets phase *duration* only — not total interest multiplier.

## Config

```ini
mortgage_activation_height = 0
mortgage_max_outstanding_zhu = 0
hip2_testnet_demo_periods = false
```

## RPC

- `GET /query_mortgage_global` — includes `apr_bps`, `early_grace_periods`, `economics_version`
- `GET /query_mortgage/contract?id=<hex>&redeemer=<addr>&height=<n>`

## HIP-25

Unchanged. Mutual exclusion with staking (status 2/3 vs 4/5).