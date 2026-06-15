use crate::core::account::Account;
use crate::mint::operate::diamond_owned_push_one;
use crate::mint::state::MintState;

fn impl_initialize(this: &BlockMintChecker, db: &mut dyn State) -> RetErr {

    {
        let mut mint_state = MintState::wrap(db);
        let mut global = mint_state.staking_global();
        global.activation_height = BlockHeight::from(this.cnf.staking_activation_height);
        mint_state.set_staking_global(&global);
    }

    if this.cnf.hip25_testnet_seed {
        let acc = Account::create_by_password(&this.cnf.hip25_testnet_seed_password)
            .map_err(|e| e.to_string())?;
        let owner = Address::cons(*acc.address());
        let dianame = DiamondName::cons(*b"WTYUIA");
        let dia = DiamondSto {
            status: DIAMOND_STATUS_NORMAL,
            address: owner.clone(),
            prev_engraved_height: BlockHeight::from(0),
            inscripts: Inscripts::default(),
        };
        let fee_hac = Amount::new_small(11, 244);
        let mut mint_state = MintState::wrap(db);
        mint_state.set_diamond(&dianame, &dia);
        diamond_owned_push_one(&mut mint_state, &owner, &dianame);
        let mut core = CoreState::wrap(db);
        core.set_balance(&owner, &Balance::hacash(fee_hac));
        println!(
            "[HIP-25 testnet seed] HACD WTYUIA + 11 HAC -> {} (password: {})",
            owner.readable(),
            &this.cnf.hip25_testnet_seed_password
        );
    }

	let addr1 = Address::from_readable("12vi7DEZjh6KrK5PVmmqSgvuJPCsZMmpfi").unwrap();
	let addr2 = Address::from_readable("1LsQLqkd8FQDh3R7ZhxC5fndNf92WfhM19").unwrap();
	let addr3 = Address::from_readable("1NUgKsTgM6vQ5nxFHGz1C4METaYTPgiihh").unwrap();
	let amt1 = Amount::new_small(1, 244);
	let amt2 = Amount::new_small(12, 244);
    let bls1 = Balance::hacash(amt1);
    let bls2 = Balance::hacash(amt2);
    let mut state = CoreState::wrap(db);
    state.set_balance(&addr1, &bls2);
    state.set_balance(&addr2, &bls1);
    state.set_balance(&addr3, &bls1);

    // let stateread = CoreStateDisk::wrap(db);

    // ok
    Ok(())
} 
