// Full transaction pipeline tests: sign → exec_tx_actions → tx.execute (HIP-2).

#[cfg(test)]
mod mortgage_e2e_tests {
    use super::*;
    use crate::chain::execute::exec_tx_actions;
    use crate::core::account::Account;
    use crate::core::state::{BlockStore, ChainState};
    use crate::interface::protocol::{Transaction, TransactionRead, TxExec};
    use crate::mint::action::{self, MortgageOpen, MortgageRedeem};
    use crate::protocol::action::create as parse_action;
    use crate::protocol::transaction::TransactionType2;
    use tempfile::TempDir;

    fn e2e_state() -> (TempDir, ChainState, BlockStore, Account) {
        action::init_reg();
        let dir = TempDir::new().unwrap();
        let path = dir.path();
        let state = ChainState::open(path);
        let store = BlockStore::from_shared(state.copy_ldb());
        let acc = Account::create_by_password("hip2e2etest").unwrap();
        (dir, state, store, acc)
    }

    fn e2e_owner(acc: &Account) -> Address {
        Address::cons(*acc.address())
    }

    fn e2e_lend_id(tag: u8) -> DiamondSyslendId {
        let mut bytes = [0u8; 14];
        bytes[0] = b'H';
        bytes[1] = tag;
        bytes[13] = b'2';
        DiamondSyslendId::cons(bytes)
    }

    fn e2e_activate(state: &mut ChainState) {
        let mut mint = MintState::wrap(state);
        let mut mg = mint.mortgage_global();
        mg.activation_height = BlockHeight::from(1);
        mg.max_outstanding_ioo_zhu = Uint8::from(MORTGAGE_DEFAULT_MAX_OUTSTANDING_ZHU);
        mg.demo_period_blocks = Uint5::from(10);
        mint.set_mortgage_global(&mg);
    }

    fn e2e_seed(
        state: &mut ChainState,
        store: &BlockStore,
        owner: &Address,
        name: &str,
        burn_mei: u16,
        hac_mei: i64,
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
            average_bid_burn: Uint2::from(burn_mei),
            life_gene: Hash::default(),
        };
        let mut mint = MintState::wrap(state);
        mint.set_diamond(&dian, &dia);
        diamond_owned_push_one(&mut mint, owner, &dian);
        drop(mint);
        let mut core = CoreState::wrap(state);
        let mut bal = Balance::hacash(Amount::from_mei(hac_mei).unwrap());
        bal.diamond = DiamondNumberAuto::from(1);
        core.set_balance(owner, &bal);
        MintStoreDisk::wrap(store).put_diamond_smelt(&dian, &smelt);
        dian
    }

    fn e2e_fee() -> Amount {
        Amount::from_string_unsafe("0:247").unwrap()
    }

    fn e2e_run_tx(
        state: &mut ChainState,
        store: &BlockStore,
        tx: &TransactionType2,
        height: u64,
    ) {
        let blkhash = Hash::default();
        exec_tx_actions(false, 0, height, blkhash, state, store, tx.as_read()).unwrap();
        tx.execute(height, state).unwrap();
    }

    fn e2e_build_open_tx(
        acc: &Account,
        lend_id: &DiamondSyslendId,
        diamonds: &DiamondNameListMax200,
        principal: &Amount,
        borrow_period: u8,
    ) -> TransactionType2 {
        let mut tx = TransactionType2::build(e2e_owner(acc), e2e_fee());
        tx.timestamp = Timestamp::from(1_700_000_000);
        let mut act = MortgageOpen::new();
        act.lending_id = lend_id.clone();
        act.mortgage_diamonds = diamonds.clone();
        act.loan_total_amount = principal.clone();
        act.borrow_period = Uint1::from(borrow_period);
        tx.push_action(Box::new(act)).unwrap();
        tx.fill_sign(acc).unwrap();
        tx
    }

    fn e2e_build_redeem_tx(
        acc: &Account,
        lend_id: &DiamondSyslendId,
        ransom: &Amount,
    ) -> TransactionType2 {
        let mut tx = TransactionType2::build(e2e_owner(acc), e2e_fee());
        tx.timestamp = Timestamp::from(1_700_000_000);
        let mut act = MortgageRedeem::new();
        act.lending_id = lend_id.clone();
        act.ransom_amount = ransom.clone();
        tx.push_action(Box::new(act)).unwrap();
        tx.fill_sign(acc).unwrap();
        tx
    }

    #[test]
    fn e2e_open_action_wire_roundtrip_kind_15() {
        action::init_reg();
        let mut act = MortgageOpen::new();
        act.lending_id = e2e_lend_id(9);
        let mut list = DiamondNameListMax200::default();
        list.push(DiamondName::cons(*b"WTYUIA")).unwrap();
        act.mortgage_diamonds = list;
        act.loan_total_amount = Amount::from_mei(50).unwrap();
        act.borrow_period = Uint1::from(5);
        let wire = act.serialize();
        assert_eq!(wire.len() >= 2, true);
        assert_eq!(u16::from_be_bytes([wire[0], wire[1]]), 15);
        let (parsed, sk) = parse_action(&wire).unwrap();
        assert_eq!(parsed.kind(), 15);
        assert_eq!(sk, wire.len());
    }

    #[test]
    fn e2e_redeem_action_wire_roundtrip_kind_16() {
        action::init_reg();
        let mut act = MortgageRedeem::new();
        act.lending_id = e2e_lend_id(8);
        act.ransom_amount = Amount::from_mei(51).unwrap();
        let wire = act.serialize();
        assert_eq!(u16::from_be_bytes([wire[0], wire[1]]), 16);
        let (parsed, _) = parse_action(&wire).unwrap();
        assert_eq!(parsed.kind(), 16);
    }

    #[test]
    fn e2e_go_compatible_field_order_kind_15() {
        // Matches hacash/core Action_15: kind | lending_id(14) | diamond_list | amount | borrow_period(1)
        action::init_reg();
        let mut act = MortgageOpen::new();
        act.lending_id = e2e_lend_id(1);
        let mut list = DiamondNameListMax200::default();
        list.push(DiamondName::cons(*b"ABCDEF")).unwrap();
        act.mortgage_diamonds = list;
        act.loan_total_amount = Amount::from_mei(10).unwrap();
        act.borrow_period = Uint1::from(3);
        let wire = act.serialize();
        let mut seek = 2usize;
        let mut lid = DiamondSyslendId::default();
        seek = lid.parse(&wire, seek).unwrap();
        let mut dlist = DiamondNameListMax200::default();
        seek = dlist.parse(&wire, seek).unwrap();
        let mut amt = Amount::default();
        seek = amt.parse(&wire, seek).unwrap();
        let mut bp = Uint1::default();
        seek = bp.parse(&wire, seek).unwrap();
        assert_eq!(seek, wire.len());
        assert_eq!(amt, act.loan_total_amount);
        assert_eq!(bp.uint(), 3);
    }

    #[test]
    fn e2e_full_tx_open_then_redeem_via_pipeline() {
        let (_dir, mut state, store, acc) = e2e_state();
        let owner = e2e_owner(&acc);
        e2e_activate(&mut state);
        e2e_seed(&mut state, &store, &owner, "WTYUIA", 100, 20);

        let list = {
            let mut l = DiamondNameListMax200::default();
            l.push(DiamondName::cons(*b"WTYUIA")).unwrap();
            l
        };
        let lend_id = e2e_lend_id(1);
        let principal = Amount::from_mei(100).unwrap();
        let open_tx = e2e_build_open_tx(&acc, &lend_id, &list, &principal, 5);
        e2e_run_tx(&mut state, &store, &open_tx, 10);

        let mint = MintStateDisk::wrap(&state);
        assert!(mint.diamond_syslend(&lend_id).is_some());
        assert_eq!(
            mint.diamond(&DiamondName::cons(*b"WTYUIA"))
                .unwrap()
                .status,
            DIAMOND_STATUS_LENDING_TO_SYSTEM
        );

        let contract = mint.diamond_syslend(&lend_id).unwrap();
        let (_, min_ransom) = mortgage_calc_ransom(&contract, &owner, 10, 10).unwrap();
        let redeem_tx = e2e_build_redeem_tx(&acc, &lend_id, &min_ransom);
        e2e_run_tx(&mut state, &store, &redeem_tx, 10);

        let mint = MintStateDisk::wrap(&state);
        assert!(mint.diamond_syslend(&lend_id).unwrap().redeemed());
        assert_eq!(
            mint.diamond(&DiamondName::cons(*b"WTYUIA"))
                .unwrap()
                .status,
            DIAMOND_STATUS_NORMAL
        );
        assert_eq!(mint.mortgage_global().outstanding_ioo_zhu.uint(), 0);
    }

    #[test]
    fn e2e_open_tx_rejected_before_activation() {
        let (_dir, mut state, store, acc) = e2e_state();
        let owner = e2e_owner(&acc);
        e2e_seed(&mut state, &store, &owner, "WTYUIA", 100, 20);
        let mut list = DiamondNameListMax200::default();
        list.push(DiamondName::cons(*b"WTYUIA")).unwrap();
        let tx = e2e_build_open_tx(
            &acc,
            &e2e_lend_id(2),
            &list,
            &Amount::from_mei(100).unwrap(),
            3,
        );
        let err = exec_tx_actions(false, 0, 5, Hash::default(), &mut state, &store, tx.as_read())
            .unwrap_err();
        assert!(format!("{}", err).contains("not active"));
    }

    #[test]
    fn e2e_double_open_same_id_rejected() {
        let (_dir, mut state, store, acc) = e2e_state();
        let owner = e2e_owner(&acc);
        e2e_activate(&mut state);
        e2e_seed(&mut state, &store, &owner, "WTYUIA", 100, 20);
        let mut list = DiamondNameListMax200::default();
        list.push(DiamondName::cons(*b"WTYUIA")).unwrap();
        let lend_id = e2e_lend_id(3);
        let principal = Amount::from_mei(100).unwrap();
        let tx1 = e2e_build_open_tx(&acc, &lend_id, &list, &principal, 3);
        e2e_run_tx(&mut state, &store, &tx1, 10);
        let tx2 = e2e_build_open_tx(&acc, &lend_id, &list, &principal, 3);
        let err = exec_tx_actions(false, 0, 11, Hash::default(), &mut state, &store, tx2.as_read())
            .unwrap_err();
        assert!(format!("{}", err).contains("already exists"));
    }

    #[test]
    fn e2e_redeem_twice_rejected() {
        let (_dir, mut state, store, acc) = e2e_state();
        let owner = e2e_owner(&acc);
        e2e_activate(&mut state);
        e2e_seed(&mut state, &store, &owner, "HXVMEK", 50, 10);
        let mut list = DiamondNameListMax200::default();
        list.push(DiamondName::cons(*b"HXVMEK")).unwrap();
        let lend_id = e2e_lend_id(4);
        let principal = Amount::from_mei(50).unwrap();
        e2e_run_tx(
            &mut state,
            &store,
            &e2e_build_open_tx(&acc, &lend_id, &list, &principal, 2),
            10,
        );
        let contract = MintStateDisk::wrap(&state).diamond_syslend(&lend_id).unwrap();
        let (_, ransom) = mortgage_calc_ransom(&contract, &owner, 10, 10).unwrap();
        e2e_run_tx(
            &mut state,
            &store,
            &e2e_build_redeem_tx(&acc, &lend_id, &ransom),
            10,
        );
        let err = exec_tx_actions(
            false,
            0,
            10,
            Hash::default(),
            &mut state,
            &store,
            e2e_build_redeem_tx(&acc, &lend_id, &ransom).as_read(),
        )
        .unwrap_err();
        assert!(format!("{}", err).contains("already redeemed"));
    }

    #[test]
    fn e2e_staked_diamond_open_tx_rejected() {
        let (_dir, mut state, store, acc) = e2e_state();
        let owner = e2e_owner(&acc);
        e2e_activate(&mut state);
        e2e_seed(&mut state, &store, &owner, "WTYUIA", 100, 20);
        let mut mint = MintState::wrap(&mut state);
        let mut sg = mint.staking_global();
        sg.activation_height = BlockHeight::from(1);
        mint.set_staking_global(&sg);
        let mut list = DiamondNameListMax200::default();
        list.push(DiamondName::cons(*b"WTYUIA")).unwrap();
        staking_apply_stake(&mut mint, &owner, &list, 10).unwrap();
        drop(mint);
        let tx = e2e_build_open_tx(
            &acc,
            &e2e_lend_id(5),
            &list,
            &Amount::from_mei(100).unwrap(),
            2,
        );
        let err = exec_tx_actions(false, 0, 10, Hash::default(), &mut state, &store, tx.as_read())
            .unwrap_err();
        assert!(format!("{}", err).contains("cannot be mortgaged"));
    }

    #[test]
    fn e2e_insufficient_origination_balance_rejected() {
        let (_dir, mut state, store, acc) = e2e_state();
        let owner = e2e_owner(&acc);
        e2e_activate(&mut state);
        e2e_seed(&mut state, &store, &owner, "WTYUIA", 100, 0);
        let mut list = DiamondNameListMax200::default();
        list.push(DiamondName::cons(*b"WTYUIA")).unwrap();
        let tx = e2e_build_open_tx(
            &acc,
            &e2e_lend_id(6),
            &list,
            &Amount::from_mei(100).unwrap(),
            2,
        );
        let err = exec_tx_actions(false, 0, 10, Hash::default(), &mut state, &store, tx.as_read())
            .unwrap_err();
        assert!(format!("{}", err).contains("not enough"));
    }
}