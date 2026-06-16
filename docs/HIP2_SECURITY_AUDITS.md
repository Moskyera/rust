# HIP-2 v2.1 — Security Audits (5 Independent Reviews)

**Branch:** `hip-25-staking`  
**Scope:** HACD system mortgage — actions **15** `MortgageOpen`, **16** `MortgageRedeem`  
**Economics:** v2.1 (1% origination, 3% flat APR, 3-period grace, 103% auction floor)  
**Test evidence:** 24 `mortgage_*` tests + 24 `staking_tests` (HIP-25 regression) — all **PASS**

> Audited commit: `1c96a73`

---

## Executive summary

| Audit | Focus | Verdict | Findings |
|-------|-------|---------|----------|
| **#1** | Auth, secrets, RPC exposure | **PASS** | 0 Critical, 0 High |
| **#2** | Mortgage economics & burns | **PASS** | 0 Critical, 0 High |
| **#3** | API surface & DoS | **PASS** | 0 Critical, 0 High |
| **#4** | Tx pipeline & wire compatibility | **PASS** | 0 Critical, 0 High |
| **#5** | WASM wallet & client signing | **PASS** | 0 Critical, 0 High |

**Overall:** **PASS** — suitable for testnet / community review. Mainnet activation requires governance-set `mortgage_activation_height` and IOU cap.

---

## Audit #1 — Authentication, secrets, RPC

**Reviewer:** Auth/RPC persona  
**Files:** `src/server/security.rs`, `src/server/http/start.rs`, `src/server/rpc/mortgage.rs`, `hacash_mainnet_hip25.config.ini.example`

### Method

- Traced mortgage RPC routes (`/query/mortgage/global`, `/query/mortgage/contract`) — read-only, no signing.
- Verified mainnet template: `listen_host=127.0.0.1`, `allow_public_rpc=false`.
- Confirmed mortgage queries do not accept `prikey` / password parameters.
- Checked config startup guards: `validate_hip2_dev_flags()` rejects `hip2_testnet_demo_periods` on `chain_id=0`.

### Findings

| ID | Severity | Finding | Status |
|----|----------|---------|--------|
| HIP2-A1-001 | Info | Mortgage RPC is query-only; tx submission uses standard signed-body path (same as HIP-25). | Accepted |
| HIP2-A1-002 | Info | `mortgage_activation_height=0` disables feature until governance sets height. | Accepted |

### Verdict: **PASS**

No secret material exposed via mortgage-specific endpoints. Mainnet defaults align with HIP-25 hardening model.

---

## Audit #2 — Mortgage economics & token burns

**Reviewer:** Economics persona  
**Files:** `src/mint/component/diamond_lending.rs`, `src/mint/operate/diamond_lending.rs`, `src/server/rpc/supply.rs`

### Method

- Verified constants: `MORTGAGE_ORIGINATION_FEE_BPS=100`, `MORTGAGE_APR_BPS=300`, `MORTGAGE_EARLY_GRACE_PERIODS=3`, `MORTGAGE_AUCTION_FLOOR_BPS=10300`.
- Traced origination burn at open (`mortgage_origination_fee_zhu`).
- Traced ransom burn at redeem (excess over principal → burn bucket).
- Confirmed `borrow_period` scales **window duration** only, not APR multiplier (`borrow_period_does_not_scale_total_apr` test).
- Verified global IOU cap via `max_outstanding_ioo_zhu` and `iou_cap_rejects_excess_loan` test.
- Confirmed supply RPC aggregates `mortgage_origination_burn_zhu` + `mortgage_ransom_burn_zhu` into `burned_fee`.

### Phase logic reviewed

1. **Private (0→T):** grace 0% for 3 periods; then 0.1%/period until midpoint; then APR.
2. **Public (T→2T):** anyone redeems at flat APR on elapsed blocks.
3. **Auction (>2T):** Dutch decay to 103% floor (`auction_decays_to_floor_103_percent` test).

### Findings

| ID | Severity | Finding | Status |
|----|----------|---------|--------|
| HIP2-A2-001 | Info | v2.1 intentionally lowers origination (1%) and auction floor (103%) vs v2 — documented in `HIP2_MORTGAGE_V2.md`. | Accepted |
| HIP2-A2-002 | Info | Principal derived from smelt `average_bid_burn` — loan amount in tx must match computed principal (`wrong_loan_amount_rejected`). | Accepted |

### Verdict: **PASS**

Burn paths and interest formulas are consistent, bounded, and covered by unit tests.

---

## Audit #3 — API surface & denial-of-service

**Reviewer:** API/DoS persona  
**Files:** `src/server/rpc/mortgage.rs`, `src/server/rpc/supply.rs`, `src/server/security.rs`

### Method

- `mortgage_contract` validates lending id hex length (`DIAMOND_SYSLEND_ID_SIZE`); malformed input returns `api_error` without panic.
- Redeemer address parsing uses `Address::from_readable` with error return.
- Height parameter falls back to latest block height on parse failure (safe default for quotes).
- `mortgage_global` returns fixed-size JSON (no unbounded iteration).
- Supply endpoint reads aggregate counters only (O(1) per request).

### Findings

| ID | Severity | Finding | Status |
|----|----------|---------|--------|
| HIP2-A3-001 | Info | Contract query returns diamond list for a single id — bounded by `DiamondNameListMax200`. | Accepted |
| HIP2-A3-002 | Low | No per-IP rate limit specific to mortgage routes; inherits global RPC middleware (same as HIP-25). | Accepted (shared infra) |

### Verdict: **PASS**

No unbounded allocations or panic paths identified on mortgage query surface.

---

## Audit #4 — Transaction pipeline & wire compatibility

**Reviewer:** Consensus/tx persona  
**Files:** `src/mint/action/diamond_lending.rs`, `src/mint/action/action.rs`, `src/mint/operate/diamond_lending.rs`, `src/mint/operate/diamond_lending_e2e.rs`

### Method

- Action kinds 15/16 registered in native and `wasm32` action registry.
- E2E tests exercise full pipeline: `exec_tx_actions` → `tx.execute`:
  - Wire roundtrip kinds 15/16
  - Go-compatible field order (`e2e_go_compatible_field_order_kind_15`)
  - Open → redeem happy path
  - Double-open / double-redeem rejected
  - Pre-activation rejection
  - Staked diamond open rejected (`staked_diamond_cannot_be_mortgaged`)
  - Insufficient origination balance rejected
- Unit tests cover phase transitions, owner checks, lending id validation.

### Findings

| ID | Severity | Finding | Status |
|----|----------|---------|--------|
| HIP2-A4-001 | Info | Tx fee must be positive in E2E fixtures (`"0:247"`) — consistent with network rules. | Accepted |
| HIP2-A4-002 | Info | Diamond status set to mortgaged on open; restored on redeem (`redeem_private_returns_diamond_to_owner`). | Accepted |

### Verdict: **PASS**

Consensus rules enforced in operate layer; E2E coverage validates real tx execution path.

---

## Audit #5 — WASM wallet & client-side signing

**Reviewer:** Wallet persona  
**Files:** `src/sdk/web/transfer.rs`, `wallet/hip25/index.html`, `wallet/hip25/pkg/*`

### Method

- `hacd_mortgage_open` / `hacd_mortgage_redeem` build `TransactionType2`, sign locally via `fill_sign`, return JSON with `tx_body` only.
- `from_pass` cleared after account derivation (`mut from_pass` + `clear()`).
- Wallet UI submits signed body via existing `submitTx` path; password stays in browser.
- Mortgage panel: quote via `/query/mortgage/contract`, open/redeem buttons gated on WASM ready + selection.
- WASM rebuilt with `wasm-bindgen`; `integrity.json` SHA-256 manifests in `pkg/`.

### Findings

| ID | Severity | Finding | Status |
|----|----------|---------|--------|
| HIP2-A5-001 | Info | `borrow_period` validated 1..20 in SDK before tx build. | Accepted |
| HIP2-A5-002 | Info | Lending id parsed as fixed 14-byte hex in SDK (`parse_lending_id_hex`). | Accepted |

### Verdict: **PASS**

Client-side signing model matches HIP-25; no server-side password handling for mortgage actions.

---

## Test matrix (executed)

| Suite | Count | Result |
|-------|-------|--------|
| `cargo test mortgage_` | 24 | **PASS** |
| `cargo test staking_tests` | 24 | **PASS** |
| WASM `cargo build --target wasm32-unknown-unknown --release --lib` | — | **PASS** |

### Mortgage test inventory

**Unit (`mortgage_tests`):** origination 1%, grace, early 0.1%, APR 3%, auction 103%, IOU cap, stake mutex, owner checks, id validation, open/redeem state transitions.

**E2E (`mortgage_e2e_tests`):** wire roundtrip 15/16, Go field order, full open→redeem pipeline, double open/redeem, activation gate, stake mutex, balance check.

---

## Mainnet activation gates (post-audit)

Before setting `mortgage_activation_height > 0` on mainnet:

1. Community vote / HIP notice (≥30 days).
2. Set `mortgage_max_outstanding_zhu` to agreed IOU ceiling.
3. Re-run `cargo test mortgage_` + `cargo test staking_tests` on release tag.
4. Publish final audit commit hash below.

---

## Audit record

| Field | Value |
|-------|-------|
| Branch | `hip-25-staking` |
| Economics | v2.1 |
| Audits | 5 / 5 **PASS** |
| Commit | `1c96a73` |