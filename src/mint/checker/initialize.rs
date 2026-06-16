use crate::core::account::Account;
use crate::interface::chain::Store;
use crate::mint::component::*;
use crate::mint::operate::diamond_owned_push_one;
use crate::mint::state::{MintState, MintStoreDisk};

/// Demo bid-burn collateral per seeded HACD (100 HAC loan principal each).
const HIP25_SEED_DIAMOND_BURN_MEI: u16 = 100;

fn impl_initialize(this: &BlockMintChecker, db: &mut dyn State, store: &dyn Store) -> RetErr {

    {
        let mut mint_state = MintState::wrap(db);
        let mut global = mint_state.staking_global();
        global.activation_height = BlockHeight::from(this.cnf.staking_activation_height);
        if this.cnf.hip25_testnet_seed && this.cnf.hip25_testnet_demo_periods {
            global.demo_min_stake_blocks = Uint5::from(5);
            global.demo_cooldown_blocks = Uint5::from(3);
            println!(
                "[HIP-25 testnet demo] short periods: min_stake=5 blocks, cooldown=3 blocks"
            );
        }
        mint_state.set_staking_global(&global);
    }

    {
        let mut mint_state = MintState::wrap(db);
        let mut mg = mint_state.mortgage_global();
        mg.activation_height = BlockHeight::from(this.cnf.mortgage_activation_height);
        let cap = this.cnf.mortgage_max_outstanding_zhu;
        mg.max_outstanding_ioo_zhu = Uint8::from(if cap > 0 {
            cap
        } else {
            MORTGAGE_DEFAULT_MAX_OUTSTANDING_ZHU
        });
        if this.cnf.hip25_testnet_seed && this.cnf.hip2_testnet_demo_periods {
            mg.demo_period_blocks = Uint5::from(10);
            println!("[HIP-2 testnet demo] mortgage period = 10 blocks");
        }
        mint_state.set_mortgage_global(&mg);
    }

    if this.cnf.hip25_testnet_seed {
        let acc = Account::create_by_password(&this.cnf.hip25_testnet_seed_password)
            .map_err(|e| e.to_string())?;
        let owner = Address::cons(*acc.address());
        let seed_diamonds: [&[u8; 6]; 5] = [
            b"WTYUIA", b"HXVMEK", b"VMEKBS", b"UIASHX", b"MEKUIA",
        ];
        let fee_hac = Amount::new_small(11, 244);
        let mut mint_state = MintState::wrap(db);
        let mint_store = MintStoreDisk::wrap(store);
        for name in seed_diamonds {
            let dianame = DiamondName::cons(*name);
            let dia = DiamondSto {
                status: DIAMOND_STATUS_NORMAL,
                address: owner.clone(),
                prev_engraved_height: BlockHeight::from(0),
                inscripts: Inscripts::default(),
            };
            mint_state.set_diamond(&dianame, &dia);
            diamond_owned_push_one(&mut mint_state, &owner, &dianame);
            let smelt = DiamondSmelt {
                diamond: dianame.clone(),
                number: DiamondNumber::from(1),
                born_height: BlockHeight::from(1),
                born_hash: Hash::default(),
                prev_hash: Hash::default(),
                miner_address: owner.clone(),
                bid_fee: Amount::default(),
                nonce: Fixed8::default(),
                average_bid_burn: Uint2::from(HIP25_SEED_DIAMOND_BURN_MEI),
                life_gene: Hash::default(),
            };
            mint_store.put_diamond_smelt(&dianame, &smelt);
        }
        let mut core = CoreState::wrap(db);
        let mut bal = Balance::hacash(fee_hac);
        bal.diamond = DiamondNumberAuto::from(seed_diamonds.len() as u64);
        core.set_balance(&owner, &bal);
        println!(
            "[HIP-25 testnet seed] 5 HACD (WTYUIA,HXVMEK,VMEKBS,UIASHX,MEKUIA) + 11 HAC -> {} (see docs for dev password)",
            owner.readable()
        );
        println!(
            "[HIP-2 testnet seed] smelt bid-burn {} mei per HACD (mortgage principal)",
            HIP25_SEED_DIAMOND_BURN_MEI
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