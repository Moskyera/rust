
fn mortgage_principal_mei_from_smelts(
    store: &MintStoreDisk,
    diamonds: &DiamondNameListMax200,
) -> Ret<u64> {
    let mut total_mei = 0u64;
    for dian in diamonds.list() {
        let smelt = must_have!(
            format!("diamond smelt {}", dian.readable()),
            store.diamond_smelt(&dian)
        );
        total_mei = total_mei.saturating_add(smelt.average_bid_burn.uint() as u64);
    }
    if total_mei == 0 {
        return errf!("mortgage loan principal must be positive");
    }
    Ok(total_mei)
}

pub fn mortgage_compute_principal(
    store: &MintStoreDisk,
    diamonds: &DiamondNameListMax200,
) -> Ret<Amount> {
    let mei = mortgage_principal_mei_from_smelts(store, diamonds)?;
    Amount::from_mei(mei as i64)
}

fn mortgage_principal_zhu(principal: &Amount) -> Ret<u64> {
    let zhu = principal.to_zhu_unsafe();
    if zhu <= 0.0 {
        return errf!("mortgage principal zhu must be positive");
    }
    Ok(zhu as u64)
}

fn mortgage_origination_fee_zhu(principal_zhu: u64) -> u64 {
    principal_zhu * MORTGAGE_ORIGINATION_FEE_BPS / 10_000
}

/// Committed interest in basis-points (0.4% × T).
fn mortgage_committed_interest_bps(borrow_period: u64) -> u64 {
    MORTGAGE_COMMITTED_INTEREST_BPS_PER_PERIOD * borrow_period
}

/// Elapsed full periods since contract creation.
fn mortgage_elapsed_periods(create_height: u64, height: u64, period_blocks: u64) -> u64 {
    if height <= create_height || period_blocks == 0 {
        return 0;
    }
    (height - create_height) / period_blocks
}

/// Ransom amount from principal zhu and interest bps (principal × (10000 + bps) / 10000).
fn mortgage_ransom_from_bps(principal_zhu: u64, interest_bps: u64) -> Ret<Amount> {
    let numerator = principal_zhu.saturating_mul(10_000 + interest_bps);
    let zhu = (numerator / 10_000) as i64;
    Amount::from_zhu(zhu)
}

pub fn mortgage_redeem_phase(
    contract: &DiamondSystemLending,
    redeemer: &Address,
    height: u64,
    period_blocks: u64,
) -> Ret<MortgageRedeemPhase> {
    if contract.redeemed() {
        return errf!("mortgage contract already redeemed");
    }
    let t = contract.borrow_period.uint() as u64;
    let create = contract.create_block_height.uint();
    let window = t.saturating_mul(period_blocks);
    let private_end = create.saturating_add(window);
    if height <= private_end {
        if *redeemer != contract.main_address {
            return errf!(
                "mortgage private redeem only by mortgagor {} until height {}",
                contract.main_address.readable(),
                private_end
            );
        }
        return Ok(MortgageRedeemPhase::Private);
    }
    let public_end = private_end.saturating_add(window);
    if height <= public_end {
        return Ok(MortgageRedeemPhase::Public);
    }
    Ok(MortgageRedeemPhase::Auction)
}

/// Minimum valid ransom for the current block height and redeemer.
pub fn mortgage_calc_ransom(
    contract: &DiamondSystemLending,
    redeemer: &Address,
    height: u64,
    period_blocks: u64,
) -> Ret<(MortgageRedeemPhase, Amount)> {
    let phase = mortgage_redeem_phase(contract, redeemer, height, period_blocks)?;
    let principal_zhu = mortgage_principal_zhu(&contract.loan_principal)?;
    let t = contract.borrow_period.uint() as u64;
    let create = contract.create_block_height.uint();
    let window = t.saturating_mul(period_blocks);
    let private_end = create.saturating_add(window);
    let public_end = private_end.saturating_add(window);

    let interest_bps = match phase {
        MortgageRedeemPhase::Private => {
            let elapsed = mortgage_elapsed_periods(create, height, period_blocks);
            let half = t / 2;
            if elapsed <= half {
                MORTGAGE_EARLY_INTEREST_BPS_PER_PERIOD * elapsed
            } else {
                mortgage_committed_interest_bps(t)
            }
        }
        MortgageRedeemPhase::Public => mortgage_committed_interest_bps(t),
        MortgageRedeemPhase::Auction => {
            let committed_bps = mortgage_committed_interest_bps(t);
            let max_zhu = principal_zhu.saturating_mul(10_000 + committed_bps) / 10_000;
            let floor_zhu = principal_zhu.saturating_mul(MORTGAGE_AUCTION_FLOOR_BPS) / 10_000;
            if height <= public_end {
                return mortgage_ransom_from_bps(principal_zhu, committed_bps)
                    .map(|a| (phase, a));
            }
            let auction_elapsed = mortgage_elapsed_periods(public_end, height, period_blocks);
            let auction_duration = t.saturating_mul(2);
            let ransom_zhu = if auction_elapsed >= auction_duration {
                floor_zhu
            } else {
                let span = max_zhu.saturating_sub(floor_zhu);
                let decay = span.saturating_mul(auction_elapsed) / auction_duration.max(1);
                max_zhu.saturating_sub(decay).max(floor_zhu)
            };
            return Amount::from_zhu(ransom_zhu as i64).map(|a| (phase, a));
        }
    };

    mortgage_ransom_from_bps(principal_zhu, interest_bps).map(|a| (phase, a))
}

pub fn mortgage_is_active_at_height(state: &MintState, height: u64) -> bool {
    state.mortgage_global().is_active_at(height)
}

fn mortgage_record_burn(state: &mut MintState, zhu: u64) {
    if zhu == 0 {
        return;
    }
    let mut ttcount = state.total_count();
    ttcount.diamond_insc_burn_zhu =
        Uint8::from(ttcount.diamond_insc_burn_zhu.uint().saturating_add(zhu));
    state.set_total_count(&ttcount);
}

pub fn check_diamond_mortgageable(
    state: &MintState,
    owner: &Address,
    hacd_name: &DiamondName,
) -> Ret<DiamondSto> {
    let diaitem = must_have!(
        format!("diamond {}", hacd_name.readable()),
        state.diamond(hacd_name)
    );
    if !diamond_status_allows_transfer(&diaitem.status) {
        return errf!(
            "diamond {} cannot be mortgaged while status is {}",
            hacd_name.readable(),
            diaitem.status.uint()
        );
    }
    if *owner != diaitem.address {
        return errf!(
            "diamond {} not belong to address {}",
            hacd_name.readable(),
            owner.readable()
        );
    }
    Ok(diaitem)
}

pub fn mortgage_apply_open(
    sta: &mut dyn State,
    sto: &dyn Store,
    owner: &Address,
    lending_id: &DiamondSyslendId,
    diamonds: &DiamondNameListMax200,
    loan_amount: &Amount,
    borrow_period: u8,
    height: u64,
) -> Ret<()> {
    let store = MintStoreDisk::wrap(sto);
    mortgage_validate_lending_id(lending_id)?;
    diamonds.check()?;

    let mut mint = MintState::wrap(sta);
    let global = mint.mortgage_global();
    if !global.is_active_at(height) {
        return errf!("HACD system mortgage is not active at height {}", height);
    }
    if mint.diamond_syslend(lending_id).is_some() {
        return errf!("mortgage lending id already exists");
    }

    let t = borrow_period as u64;
    if t < 1 || t > 20 {
        return errf!("borrow period must be between 1 and 20");
    }

    let computed = mortgage_compute_principal(&store, diamonds)?;
    if computed != *loan_amount {
        return errf!(
            "loan amount must be {} but got {}",
            computed.to_fin_string(),
            loan_amount.to_fin_string()
        );
    }

    let principal_zhu = mortgage_principal_zhu(loan_amount)?;
    let origination_zhu = mortgage_origination_fee_zhu(principal_zhu);
    let new_outstanding = global.outstanding_ioo_zhu.uint().saturating_add(principal_zhu);
    let cap = global.max_outstanding_ioo_zhu.uint();
    if cap > 0 && new_outstanding > cap {
        return errf!(
            "mortgage IOU cap exceeded: outstanding {} + loan {} > max {}",
            global.outstanding_ioo_zhu.uint(),
            principal_zhu,
            cap
        );
    }

    drop(mint);

    if origination_zhu > 0 {
        let fee_amt = Amount::from_zhu(origination_zhu as i64)?;
        let mut core = CoreState::wrap(sta);
        hac_sub(&mut core, owner, &fee_amt)?;
    }

    let mut mint = MintState::wrap(sta);
    for dian in diamonds.list() {
        let mut diaitem = check_diamond_mortgageable(&mint, owner, &dian)?;
        diaitem.status = DIAMOND_STATUS_LENDING_TO_SYSTEM;
        mint.set_diamond(&dian, &diaitem);
    }
    mortgage_drop_owned(&mut mint, owner, diamonds)?;

    let contract = DiamondSystemLending {
        is_ransomed: Uint1::from(0),
        create_block_height: BlockHeight::from(height),
        main_address: owner.clone(),
        mortgage_diamonds: diamonds.clone(),
        loan_principal: loan_amount.clone(),
        borrow_period: Uint1::from(borrow_period),
        ransom_block_height: BlockHeight::from(0),
        ransom_address: Address::default(),
    };
    mint.set_diamond_syslend(lending_id, &contract);

    let mut global = mint.mortgage_global();
    global.outstanding_ioo_zhu = Uint8::from(new_outstanding);
    global.cumulative_loan_zhu =
        Uint8::from(global.cumulative_loan_zhu.uint().saturating_add(principal_zhu));
    global.cumulative_origination_burn_zhu = Uint8::from(
        global
            .cumulative_origination_burn_zhu
            .uint()
            .saturating_add(origination_zhu),
    );
    global.active_contracts = Uint5::from(global.active_contracts.uint() + 1);
    mint.set_mortgage_global(&global);

    mortgage_record_burn(&mut mint, origination_zhu);
    drop(mint);

    let dianum = diamonds.count().uint() as u32;
    let mut core = CoreState::wrap(sta);
    hacd_sub(&mut core, owner, &DiamondNumber::from(dianum))?;
    hac_add(&mut core, owner, loan_amount)?;
    Ok(())
}

pub fn mortgage_apply_redeem(
    sta: &mut dyn State,
    redeemer: &Address,
    lending_id: &DiamondSyslendId,
    ransom_amount: &Amount,
    height: u64,
) -> Ret<()> {
    mortgage_validate_lending_id(lending_id)?;

    let mut mint = MintState::wrap(sta);
    let global = mint.mortgage_global();
    if !global.is_active_at(height) {
        return errf!("HACD system mortgage is not active at height {}", height);
    }
    let period_blocks = global.effective_period_blocks();

    let mut contract = must_have!(
        format!("mortgage contract {:?}", lending_id),
        mint.diamond_syslend(lending_id)
    );
    if contract.redeemed() {
        return errf!("mortgage contract already redeemed");
    }

    let (phase, min_ransom) =
        mortgage_calc_ransom(&contract, redeemer, height, period_blocks)?;
    if ransom_amount.less_than(&min_ransom) {
        return errf!(
            "ransom must be at least {} (phase {}) but got {}",
            min_ransom.to_fin_string(),
            phase.label(),
            ransom_amount.to_fin_string()
        );
    }

    drop(mint);

    let mut core = CoreState::wrap(sta);
    hac_sub(&mut core, redeemer, ransom_amount)?;
    drop(core);

    let mut mint = MintState::wrap(sta);
    let mut contract = must_have!(
        format!("mortgage contract {:?}", lending_id),
        mint.diamond_syslend(lending_id)
    );

    let diamonds = contract.mortgage_diamonds.clone();
    diamonds.check()?;
    for dian in diamonds.list() {
        let mut diaitem = must_have!(
            format!("diamond {}", dian.readable()),
            mint.diamond(&dian)
        );
        if diaitem.status != DIAMOND_STATUS_LENDING_TO_SYSTEM {
            return errf!(
                "diamond {} expected mortgage-to-system status",
                dian.readable()
            );
        }
        diaitem.status = DIAMOND_STATUS_NORMAL;
        diaitem.address = redeemer.clone();
        mint.set_diamond(&dian, &diaitem);
    }

    contract.mark_ransomed(height, redeemer);
    mint.set_diamond_syslend(lending_id, &contract);

    let principal_zhu = mortgage_principal_zhu(&contract.loan_principal)?;
    let ransom_zhu = ransom_amount.to_zhu_unsafe().max(0.0) as u64;

    let mut global = mint.mortgage_global();
    global.outstanding_ioo_zhu = Uint8::from(
        global
            .outstanding_ioo_zhu
            .uint()
            .saturating_sub(principal_zhu),
    );
    global.cumulative_ransom_burn_zhu = Uint8::from(
        global
            .cumulative_ransom_burn_zhu
            .uint()
            .saturating_add(ransom_zhu),
    );
    global.active_contracts = Uint5::from(global.active_contracts.uint().saturating_sub(1));
    mint.set_mortgage_global(&global);

    mortgage_record_burn(&mut mint, ransom_zhu);
    diamond_owned_push_batch(&mut mint, redeemer, &diamonds)?;
    drop(mint);

    let dianum = diamonds.count().uint() as u32;
    let mut core = CoreState::wrap(sta);
    hacd_add(&mut core, redeemer, &DiamondNumber::from(dianum))?;
    Ok(())
}

fn mortgage_drop_owned(
    state: &mut MintState,
    owner: &Address,
    diamonds: &DiamondNameListMax200,
) -> Ret<()> {
    let mut owned = must_have!(
        format!("diamond owned for {}", owner.readable()),
        state.diamond_owned(owner)
    );
    let remaining = owned.drop(diamonds)?;
    if remaining > 0 {
        state.set_diamond_owned(owner, &owned);
    } else {
        state.del_diamond_owned(owner);
    }
    Ok(())
}

fn diamond_owned_push_batch(
    state: &mut MintState,
    owner: &Address,
    diamonds: &DiamondNameListMax200,
) -> Ret<()> {
    let mut owned = state.diamond_owned(owner).unwrap_or_default();
    owned.push(diamonds);
    state.set_diamond_owned(owner, &owned);
    Ok(())
}

#[cfg(test)]
mod mortgage_tests {
    use super::*;
    use crate::core::state::{BlockStore, ChainState};
    use tempfile::TempDir;

    fn test_state() -> (TempDir, ChainState, BlockStore) {
        let dir = TempDir::new().unwrap();
        let path = dir.path();
        let state = ChainState::open(path);
        let store = BlockStore::from_shared(state.copy_ldb());
        (dir, state, store)
    }

    fn test_owner() -> Address {
        Address::from_readable("12vi7DEZjh6KrK5PVmmqSgvuJPCsZMmpfi").unwrap()
    }

    fn test_other() -> Address {
        Address::from_readable("1LsQLqkd8FQDh3R7ZhxC5fndNf92WfhM19").unwrap()
    }

    fn test_lend_id(tag: u8) -> DiamondSyslendId {
        let mut bytes = [0u8; 14];
        bytes[0] = b'M';
        bytes[1] = tag;
        bytes[13] = b'Z';
        DiamondSyslendId::cons(bytes)
    }

    fn lit(name: &[u8; 6]) -> DiamondName {
        DiamondName::cons(*name)
    }

    fn one_diamond_list(name: &str) -> DiamondNameListMax200 {
        let mut list = DiamondNameListMax200::default();
        list.push(DiamondName::cons(name.as_bytes().try_into().unwrap()))
            .unwrap();
        list
    }

    fn activate_mortgage(state: &mut ChainState, demo_periods: bool) {
        let mut mint = MintState::wrap(state);
        let mut global = mint.mortgage_global();
        global.activation_height = BlockHeight::from(1);
        global.max_outstanding_ioo_zhu =
            Uint8::from(MORTGAGE_DEFAULT_MAX_OUTSTANDING_ZHU);
        if demo_periods {
            global.demo_period_blocks = Uint5::from(10);
        }
        mint.set_mortgage_global(&global);
    }

    fn seed_diamond_with_smelt(
        state: &mut ChainState,
        store: &BlockStore,
        name: &str,
        owner: &Address,
        average_bid_burn_mei: u16,
    ) -> DiamondName {
        let dian = DiamondName::cons(name.as_bytes().try_into().unwrap());
        let dia = DiamondSto {
            status: DIAMOND_STATUS_NORMAL,
            address: owner.clone(),
            prev_engraved_height: BlockHeight::from(0),
            inscripts: Inscripts::default(),
        };
        let smelt = DiamondSmelt {
            diamond: dian.clone(),
            number: DiamondNumber::from(1),
            born_height: BlockHeight::from(1),
            born_hash: Hash::default(),
            prev_hash: Hash::default(),
            miner_address: owner.clone(),
            bid_fee: Amount::default(),
            nonce: Fixed8::default(),
            average_bid_burn: Uint2::from(average_bid_burn_mei),
            life_gene: Hash::default(),
        };
        let mut mint = MintState::wrap(state);
        mint.set_diamond(&dian, &dia);
        diamond_owned_push_one(&mut mint, owner, &dian);
        let mint_store = MintStoreDisk::wrap(store);
        mint_store.put_diamond_smelt(&dian, &smelt);
        dian
    }

    fn fund_owner(state: &mut ChainState, owner: &Address, hac_mei: i64, hacd_count: u32) {
        let mut core = CoreState::wrap(state);
        let mut bal = Balance::hacash(Amount::from_mei(hac_mei).unwrap());
        bal.diamond = DiamondNumberAuto::from(hacd_count as u64);
        core.set_balance(owner, &bal);
    }

    fn hac_balance(state: &ChainState, addr: &Address) -> Amount {
        let core = CoreStateDisk::wrap(state);
        core.balance(addr)
            .map(|b| b.hacash.clone())
            .unwrap_or_default()
    }

    #[test]
    fn lending_id_validation_rejects_bad_format() {
        let mut bytes = [0u8; 14];
        let id = DiamondSyslendId::cons(bytes);
        assert!(mortgage_validate_lending_id(&id).is_err());
        bytes[0] = 1;
        bytes[13] = 0;
        let id2 = DiamondSyslendId::cons(bytes);
        assert!(mortgage_validate_lending_id(&id2).is_err());
    }

    #[test]
    fn origination_fee_is_two_percent() {
        assert_eq!(mortgage_origination_fee_zhu(10_000), 200);
        assert_eq!(mortgage_origination_fee_zhu(1_000_000), 20_000);
    }

    #[test]
    fn private_early_redeem_zero_interest_at_open() {
        let contract = DiamondSystemLending {
            is_ransomed: Uint1::from(0),
            create_block_height: BlockHeight::from(100),
            main_address: test_owner(),
            mortgage_diamonds: DiamondNameListMax200::default(),
            loan_principal: Amount::from_mei(100).unwrap(),
            borrow_period: Uint1::from(10),
            ransom_block_height: BlockHeight::from(0),
            ransom_address: Address::default(),
        };
        let (_, ransom) = mortgage_calc_ransom(&contract, &test_owner(), 100, 10).unwrap();
        assert_eq!(ransom, Amount::from_mei(100).unwrap());
    }

    #[test]
    fn private_early_redeem_scales_quarter_percent_per_period() {
        let contract = DiamondSystemLending {
            is_ransomed: Uint1::from(0),
            create_block_height: BlockHeight::from(100),
            main_address: test_owner(),
            mortgage_diamonds: DiamondNameListMax200::default(),
            loan_principal: Amount::from_mei(1000).unwrap(),
            borrow_period: Uint1::from(10),
            ransom_block_height: BlockHeight::from(0),
            ransom_address: Address::default(),
        };
        let (_, ransom) = mortgage_calc_ransom(&contract, &test_owner(), 130, 10).unwrap();
        // 3 elapsed periods × 0.25% = 0.75%
        let principal_zhu = mortgage_principal_zhu(&contract.loan_principal).unwrap();
        let expected = mortgage_ransom_from_bps(principal_zhu, 75).unwrap();
        assert_eq!(ransom.to_fin_string(), expected.to_fin_string());
    }

    #[test]
    fn private_second_half_uses_committed_interest() {
        let contract = DiamondSystemLending {
            is_ransomed: Uint1::from(0),
            create_block_height: BlockHeight::from(0),
            main_address: test_owner(),
            mortgage_diamonds: DiamondNameListMax200::default(),
            loan_principal: Amount::from_mei(1000).unwrap(),
            borrow_period: Uint1::from(4),
            ransom_block_height: BlockHeight::from(0),
            ransom_address: Address::default(),
        };
        // T=4, half=2, elapsed=3 periods → committed 1.6%
        let (_, ransom) = mortgage_calc_ransom(&contract, &test_owner(), 30, 10).unwrap();
        let expected = Amount::from_mei(1016).unwrap();
        assert_eq!(ransom.to_fin_string(), expected.to_fin_string());
    }

    #[test]
    fn public_redeem_allows_anyone_at_committed_rate() {
        let contract = DiamondSystemLending {
            is_ransomed: Uint1::from(0),
            create_block_height: BlockHeight::from(0),
            main_address: test_owner(),
            mortgage_diamonds: DiamondNameListMax200::default(),
            loan_principal: Amount::from_mei(500).unwrap(),
            borrow_period: Uint1::from(5),
            ransom_block_height: BlockHeight::from(0),
            ransom_address: Address::default(),
        };
        let period = 10u64;
        let public_h = 5 * period + 1;
        let (_, ransom) =
            mortgage_calc_ransom(&contract, &test_other(), public_h, period).unwrap();
        // 5 × 0.4% = 2%
        let expected = Amount::from_mei(510).unwrap();
        assert_eq!(ransom.to_fin_string(), expected.to_fin_string());
    }

    #[test]
    fn auction_decays_to_floor_110_percent() {
        let contract = DiamondSystemLending {
            is_ransomed: Uint1::from(0),
            create_block_height: BlockHeight::from(0),
            main_address: test_owner(),
            mortgage_diamonds: DiamondNameListMax200::default(),
            loan_principal: Amount::from_mei(1000).unwrap(),
            borrow_period: Uint1::from(2),
            ransom_block_height: BlockHeight::from(0),
            ransom_address: Address::default(),
        };
        let period = 10u64;
        let public_end = 2 * 2 * period;
        let floor_h = public_end + 2 * 2 * period;
        let (_, ransom) =
            mortgage_calc_ransom(&contract, &test_other(), floor_h, period).unwrap();
        let expected = Amount::from_mei(1100).unwrap();
        assert_eq!(ransom.to_fin_string(), expected.to_fin_string());
    }

    #[test]
    fn private_redeem_rejects_non_owner() {
        let contract = DiamondSystemLending {
            is_ransomed: Uint1::from(0),
            create_block_height: BlockHeight::from(0),
            main_address: test_owner(),
            mortgage_diamonds: DiamondNameListMax200::default(),
            loan_principal: Amount::from_mei(100).unwrap(),
            borrow_period: Uint1::from(2),
            ransom_block_height: BlockHeight::from(0),
            ransom_address: Address::default(),
        };
        let err = mortgage_calc_ransom(&contract, &test_other(), 5, 10).unwrap_err();
        assert!(format!("{}", err).contains("private redeem"));
    }

    #[test]
    fn open_mortgage_credits_loan_and_burns_origination() {
        let (_dir, mut state, store) = test_state();
        activate_mortgage(&mut state, true);
        let owner = test_owner();
        seed_diamond_with_smelt(&mut state, &store, "WTYUIA", &owner, 100);
        fund_owner(&mut state, &owner, 50, 1);

        let list = one_diamond_list("WTYUIA");
        let principal = Amount::from_mei(100).unwrap();
        let lend_id = test_lend_id(1);

        mortgage_apply_open(
            &mut state,
            &store,
            &owner,
            &lend_id,
            &list,
            &principal,
            5,
            10,
        )
        .unwrap();

        let mint = MintStateDisk::wrap(&state);
        let dia = mint.diamond(&lit(b"WTYUIA")).unwrap();
        assert_eq!(dia.status, DIAMOND_STATUS_LENDING_TO_SYSTEM);
        assert_eq!(mint.mortgage_global().outstanding_ioo_zhu.uint(), 100_0000_0000);
        assert_eq!(mint.mortgage_global().cumulative_origination_burn_zhu.uint(), 2_0000_0000);
        // 50 HAC start - 2 HAC origination + 100 HAC loan = 148 HAC
        let bal = hac_balance(&state, &owner);
        assert_eq!(bal, Amount::from_mei(148).unwrap());
    }

    #[test]
    fn redeem_private_returns_diamond_to_owner() {
        let (_dir, mut state, store) = test_state();
        activate_mortgage(&mut state, true);
        let owner = test_owner();
        seed_diamond_with_smelt(&mut state, &store, "WTYUIA", &owner, 200);
        fund_owner(&mut state, &owner, 10, 1);
        let list = one_diamond_list("WTYUIA");
        let principal = Amount::from_mei(200).unwrap();
        let lend_id = test_lend_id(2);

        mortgage_apply_open(
            &mut state,
            &store,
            &owner,
            &lend_id,
            &list,
            &principal,
            4,
            10,
        )
        .unwrap();

        let (_, min_ransom) = {
            let mint = MintStateDisk::wrap(&state);
            let c = mint.diamond_syslend(&lend_id).unwrap();
            mortgage_calc_ransom(&c, &owner, 10, 10).unwrap()
        };

        mortgage_apply_redeem(&mut state, &owner, &lend_id, &min_ransom, 10).unwrap();

        let mint = MintStateDisk::wrap(&state);
        assert!(mint.diamond_syslend(&lend_id).unwrap().redeemed());
        assert_eq!(mint.diamond(&lit(b"WTYUIA")).unwrap().status, DIAMOND_STATUS_NORMAL);
        assert_eq!(mint.mortgage_global().outstanding_ioo_zhu.uint(), 0);
    }

    #[test]
    fn iou_cap_rejects_excess_loan() {
        let (_dir, mut state, store) = test_state();
        activate_mortgage(&mut state, true);
        let owner = test_owner();
        seed_diamond_with_smelt(&mut state, &store, "WTYUIA", &owner, 1000);
        fund_owner(&mut state, &owner, 500, 1);

        let mut mint = MintState::wrap(&mut state);
        let mut global = mint.mortgage_global();
        global.max_outstanding_ioo_zhu = Uint8::from(50_0000_0000); // 50 HAC
        mint.set_mortgage_global(&global);

        let list = one_diamond_list("WTYUIA");
        let principal = Amount::from_mei(1000).unwrap();
        let err = mortgage_apply_open(
            &mut state,
            &store,
            &owner,
            &test_lend_id(3),
            &list,
            &principal,
            3,
            10,
        )
        .unwrap_err();
        assert!(format!("{}", err).contains("IOU cap"));
    }

    #[test]
    fn staked_diamond_cannot_be_mortgaged() {
        let (_dir, mut state, store) = test_state();
        activate_mortgage(&mut state, true);
        let owner = test_owner();
        seed_diamond_with_smelt(&mut state, &store, "WTYUIA", &owner, 100);
        fund_owner(&mut state, &owner, 20, 1);

        let list = one_diamond_list("WTYUIA");
        let mut mint = MintState::wrap(&mut state);
        let mut global = mint.staking_global();
        global.activation_height = BlockHeight::from(1);
        mint.set_staking_global(&global);
        staking_apply_stake(&mut mint, &owner, &list, 10).unwrap();
        drop(mint);

        let err = mortgage_apply_open(
            &mut state,
            &store,
            &owner,
            &test_lend_id(4),
            &list,
            &Amount::from_mei(100).unwrap(),
            2,
            10,
        )
        .unwrap_err();
        assert!(format!("{}", err).contains("cannot be mortgaged"));
    }

    #[test]
    fn wrong_loan_amount_rejected() {
        let (_dir, mut state, store) = test_state();
        activate_mortgage(&mut state, true);
        let owner = test_owner();
        seed_diamond_with_smelt(&mut state, &store, "WTYUIA", &owner, 100);
        fund_owner(&mut state, &owner, 20, 1);

        let err = mortgage_apply_open(
            &mut state,
            &store,
            &owner,
            &test_lend_id(5),
            &one_diamond_list("WTYUIA"),
            &Amount::from_mei(99).unwrap(),
            2,
            10,
        )
        .unwrap_err();
        assert!(format!("{}", err).contains("loan amount must"));
    }
}