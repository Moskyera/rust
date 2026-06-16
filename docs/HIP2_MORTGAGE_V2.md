# HIP-2 v2 — HACD System Mortgage (HAC IOU)

**Actions:** 15 `MortgageOpen`, 16 `MortgageRedeem` (aligned with Go `diamondlending.go` kinds).

## Economics

| Parameter | Value |
|-----------|-------|
| Loan principal | Σ `average_bid_burn` per mortgaged HACD (from `DiamondSmelt` store) |
| Origination fee | **2%** of principal → burn (paid at open) |
| Committed interest | **0.4% × principal × T** (T = borrow periods, 1–20) |
| Early private discount | **0.25% × principal × elapsed periods** in first half of private window |
| Period length | **10_000** blocks mainnet (~35d); **10** blocks testnet demo |
| Auction floor | **110%** of principal |

## Redeem phases

1. **Private** (0 → T periods): only mortgagor; early interest in first T/2 periods.
2. **Public** (T → 2T periods): anyone; full committed interest.
3. **Auction** (> 2T periods): Dutch decay from committed ransom to 110% floor over **2T** periods.

## Safety

- Global outstanding IOU cap (`max_outstanding_ioo_zhu`)
- Mutual exclusion with HIP-25 staking (status 2/3 vs 4/5)
- No silent collateral burn — redeem only via action 16
- Supply counters in `GlobalMortgageState`

## Config (`mint` section)

```ini
mortgage_activation_height = 0          ; 0 = disabled until governance sets height
mortgage_max_outstanding_zhu = 0        ; 0 = protocol default (8M HAC scale)
hip2_testnet_demo_periods = false       ; dev only, requires hip25_testnet_seed
```

## RPC

- `GET /query_mortgage_global`
- `GET /query_mortgage_contract?id=<hex>&redeemer=<addr>&height=<n>`

## HIP-25 relationship

HIP-25 v2 staking is unchanged. Mortgage and stake cannot apply to the same HACD (status mutex).