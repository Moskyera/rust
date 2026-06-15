
fn staking_accrued_zhu(global_index: &Uint8, snapshot: &Uint8) -> u64 {
    global_index.uint().saturating_sub(snapshot.uint())
}

pub fn staking_accrued_amount(global_index: &Uint8, snapshot: &Uint8) -> Ret<Amount> {
    let zhu = staking_accrued_zhu(global_index, snapshot) as i64;
    if zhu <= 0 {
        return Ok(Amount::default());
    }
    Amount::from_zhu(zhu)
}

/// Live accrual while staked; fixed `pending_reward` during cooldown (HIP-25 v1).
pub fn staking_display_accrued_reward(
    global_index: &Uint8,
    record: &StakingRecord,
) -> Ret<Amount> {
    if record.is_active_stake() {
        staking_accrued_amount(global_index, &record.reward_index)
    } else {
        Ok(record.pending_reward.clone())
    }
}

pub fn staking_is_active_at_height(state: &MintState, height: u64) -> bool {
    state.staking_global().is_active_at(height)
}

/// Returns true if tx contains a HACD transfer action (kinds 5–8).
pub fn tx_contains_diamond_transfer(tx: &dyn TransactionRead) -> bool {
    for act in tx.actions() {
        let k = act.kind();
        if (5..=8).contains(&k) {
            return true;
        }
    }
    false
}

pub fn staking_redirect_fee_zhu(fee_zhu: u64) -> (u64, u64) {
    let to_pool = fee_zhu * STAKING_FEE_SHARE_PERCENT / 100;
    let to_burn = fee_zhu - to_pool;
    (to_pool, to_burn)
}

/// HIP-25: 13% of total transfer fee → pool; remainder follows burn/miner split.
pub fn staking_split_transfer_tx_fee(total: &Amount, burn_90: bool) -> Ret<(u64, Amount)> {
    let total_zhu = total.to_zhu_unsafe() as u64;
    let (to_pool, remainder_zhu) = staking_redirect_fee_zhu(total_zhu);
    let mut miner = if remainder_zhu > 0 {
        Amount::from_zhu(remainder_zhu as i64)?
    } else {
        Amount::default()
    };
    if burn_90 && miner.unit() > 1 {
        miner.unit_sub(1);
    }
    Ok((to_pool, miner))
}

fn staking_push_event(state: &mut MintState, event: &StakingEvent) {
    let mut global = state.staking_global();
    let id = global.event_log_tail.uint();
    state.set_staking_event(&Uint5::from(id), event);
    global.event_log_tail = Uint5::from(id + 1);
    state.set_staking_global(&global);
}

pub fn staking_deposit_fee(state: &mut MintState, fee_zhu: u64) {
    if fee_zhu == 0 {
        return;
    }
    let mut global = state.staking_global();
    global.reward_pool_zhu = Uint8::from(global.reward_pool_zhu.uint() + fee_zhu);
    state.set_staking_global(&global);
}

pub fn staking_distribute_rewards(state: &mut MintState, height: u64) -> Ret<()> {
    let mut global = state.staking_global();
    let shares = global.total_staked_shares.uint();
    let pool = global.reward_pool_zhu.uint();
    if shares == 0 || pool == 0 {
        return Ok(());
    }
    let increment = pool / shares;
    if increment > 0 {
        global.global_reward_index =
            Uint8::from(global.global_reward_index.uint() + increment);
    }
    // dust stays in pool when increment rounds to zero
    let distributed = increment * shares;
    if distributed >= pool {
        global.reward_pool_zhu = Uint8::from(0);
    } else {
        global.reward_pool_zhu = Uint8::from(pool - distributed);
    }
    state.set_staking_global(&global);
    if distributed > 0 {
        let reward = Amount::from_zhu(distributed as i64).unwrap_or_default();
        staking_push_event(
            state,
            &StakingEvent {
                kind: STAKING_EVENT_REWARD_DISTRIBUTED,
                height: BlockHeight::from(height),
                diamond: DiamondName::default(),
                staker: Address::default(),
                unlock_height: BlockHeight::from(0),
                reward,
                shares: Uint5::from(shares),
            },
        );
    }
    Ok(())
}

fn staking_enqueue_unlock(state: &mut MintState, entry: &StakingUnlockEntry) -> Ret<()> {
    let mut global = state.staking_global();
    let id = global.unlock_queue_tail.uint();
    let key = Uint5::from(id);
    state.set_staking_unlock_entry(&key, entry);
    global.unlock_queue_tail = Uint5::from(id + 1);
    state.set_staking_global(&global);
    Ok(())
}

fn staking_finalize_unlock(mint_state: &mut MintState, entry: &StakingUnlockEntry) -> Ret<()> {
    let dianame = &entry.diamond;
    let mut diaitem = must_have!(
        format!("diamond {}", dianame.readable()),
        mint_state.diamond(dianame)
    );
    if diaitem.status != DIAMOND_STATUS_STAKING_COOLDOWN {
        return errf!(
            "diamond {} unlock failed: expected cooldown status",
            dianame.readable()
        );
    }
    diaitem.status = DIAMOND_STATUS_NORMAL;
    mint_state.set_diamond(dianame, &diaitem);
    mint_state.del_staking_record(dianame);

    staking_push_event(
        mint_state,
        &StakingEvent {
            kind: STAKING_EVENT_UNSTAKED,
            height: entry.unlock_height.clone(),
            diamond: entry.diamond.clone(),
            staker: entry.staker.clone(),
            unlock_height: entry.unlock_height.clone(),
            reward: entry.reward.clone(),
            shares: Uint5::from(0),
        },
    );

    Ok(())
}

pub fn staking_process_unlock_queue(base_state: &mut dyn State, height: u64) -> Ret<()> {
    let mut pending: Vec<(Uint5, StakingUnlockEntry)> = Vec::new();

    {
        let mut mint_state = MintState::wrap(base_state);
        let mut global = mint_state.staking_global();
        let mut head = global.unlock_queue_head.uint();
        let tail = global.unlock_queue_tail.uint();

        while head < tail {
            let key = Uint5::from(head);
            let entry = match mint_state.staking_unlock_entry(&key) {
                Some(e) => e,
                None => {
                    return errf!(
                        "staking unlock queue corrupted: missing entry {} (head {} tail {})",
                        head,
                        head,
                        tail
                    );
                }
            };
            if entry.unlock_height.uint() > height {
                break;
            }
            pending.push((key, entry));
            head += 1;
        }

        global.unlock_queue_head = Uint5::from(head);
        mint_state.set_staking_global(&global);
    }

    for (key, entry) in pending {
        let reward = entry.reward.clone();
        let staker = entry.staker.clone();
        {
            let mut mint_state = MintState::wrap(base_state);
            staking_finalize_unlock(&mut mint_state, &entry)?;
            mint_state.del_staking_unlock_entry(&key);
        }
        if reward.is_positive() {
            let mut core_state = CoreState::wrap(base_state);
            hac_add(&mut core_state, &staker, &reward)?;
        }
    }

    Ok(())
}

pub fn staking_on_block_close(base_state: &mut dyn State, height: u64) -> Ret<()> {
    let mint_state = MintState::wrap(base_state);
    if !staking_is_active_at_height(&mint_state, height) {
        return Ok(());
    }
    drop(mint_state);
    {
        let mut mint_state = MintState::wrap(base_state);
        staking_distribute_rewards(&mut mint_state, height)?;
    }
    staking_process_unlock_queue(base_state, height)?;
    Ok(())
}

pub fn check_diamond_stakeable(
    state: &MintState,
    staker: &Address,
    hacd_name: &DiamondName,
) -> Ret<DiamondSto> {
    let diaitem = must_have!(
        format!("diamond {}", hacd_name.readable()),
        state.diamond(hacd_name)
    );
    if !diamond_status_allows_transfer(&diaitem.status) {
        return errf!(
            "diamond {} cannot be staked while status is {}",
            hacd_name.readable(),
            diaitem.status.uint()
        );
    }
    if *staker != diaitem.address {
        return errf!(
            "diamond {} not belong to address {}",
            hacd_name.readable(),
            staker.readable()
        );
    }
    Ok(diaitem)
}

/// Parse HVM / HIP-25 diamond list wire format: `Uint1 count` + `count × 6` literal bytes.
pub fn staking_parse_hvm_diamonds(raw: &[u8]) -> Ret<DiamondNameListMax200> {
    if raw.is_empty() {
        return errf!("diamond list empty");
    }
    let mut list = DiamondNameListMax200::default();
    list.parse(raw, 0)?;
    list.check()?;
    Ok(list)
}

/// Execute HIP-25 HVM external opcode (`0x01` stake, `0x02` unstake) against Mint state.
pub fn staking_exec_hvm_external(
    opcode: u8,
    payload: &[u8],
    staker: &Address,
    height: u64,
    base_state: &mut dyn State,
) -> Ret<()> {
    let diamonds = staking_parse_hvm_diamonds(payload)?;
    let mut mint_state = MintState::wrap(base_state);
    match opcode {
        STAKE_HACD_VMKIND => staking_apply_stake(&mut mint_state, staker, &diamonds, height),
        UNSTAKE_HACD_VMKIND => staking_apply_unstake(&mut mint_state, staker, &diamonds, height),
        _ => errf!("unknown HIP-25 HVM opcode {}", opcode),
    }
}

pub fn staking_set_paused(state: &mut MintState, paused: bool) {
    let mut global = state.staking_global();
    global.paused = Uint1::from(if paused { 1 } else { 0 });
    state.set_staking_global(&global);
}

pub fn staking_apply_stake(
    state: &mut MintState,
    staker: &Address,
    diamonds: &DiamondNameListMax200,
    height: u64,
) -> Ret<()> {
    diamonds.check()?;
    let mut global = state.staking_global();
    if !global.is_active_at(height) {
        return errf!("HACD staking is not active at height {}", height);
    }
    if global.is_paused() {
        return errf!("HACD staking is paused");
    }
    let reward_index = global.global_reward_index.clone();

    for dianame in diamonds.list() {
        let mut diaitem = check_diamond_stakeable(state, staker, &dianame)?;
        diaitem.status = DIAMOND_STATUS_STAKED;
        state.set_diamond(&dianame, &diaitem);

        let record = StakingRecord {
            stake_height: BlockHeight::from(height),
            unlock_height: BlockHeight::from(0),
            reward_index: reward_index.clone(),
            pending_reward: Amount::default(),
        };
        state.set_staking_record(&dianame, &record);
        global.total_staked_shares =
            Uint5::from(global.total_staked_shares.uint() + 1);

        staking_push_event(
            state,
            &StakingEvent {
                kind: STAKING_EVENT_STAKED,
                height: BlockHeight::from(height),
                diamond: dianame.clone(),
                staker: staker.clone(),
                unlock_height: BlockHeight::from(0),
                reward: Amount::default(),
                shares: global.total_staked_shares.clone(),
            },
        );
    }

    let mut final_global = state.staking_global();
    final_global.total_staked_shares = global.total_staked_shares;
    state.set_staking_global(&final_global);
    Ok(())
}

pub fn staking_apply_unstake(
    state: &mut MintState,
    staker: &Address,
    diamonds: &DiamondNameListMax200,
    height: u64,
) -> Ret<()> {
    diamonds.check()?;
    let global = state.staking_global();
    let reward_index = global.global_reward_index.clone();

    for dianame in diamonds.list() {
        let mut diaitem = must_have!(
            format!("diamond {}", dianame.readable()),
            state.diamond(&dianame)
        );
        if diaitem.status != DIAMOND_STATUS_STAKED {
            return errf!("diamond {} is not staked", dianame.readable());
        }
        if *staker != diaitem.address {
            return errf!(
                "diamond {} not belong to staker {}",
                dianame.readable(),
                staker.readable()
            );
        }

        let record = must_have!(
            format!("staking record for {}", dianame.readable()),
            state.staking_record(&dianame)
        );
        let stake_height = record.stake_height.uint();
        let global_snap = state.staking_global();
        let min_stake = global_snap.effective_min_stake_blocks();
        let cooldown = global_snap.effective_cooldown_blocks();
        if height < stake_height + min_stake {
            return errf!(
                "diamond {} must remain staked for at least {} blocks",
                dianame.readable(),
                min_stake
            );
        }

        let reward = staking_accrued_amount(&reward_index, &record.reward_index)?;

        diaitem.status = DIAMOND_STATUS_STAKING_COOLDOWN;
        state.set_diamond(&dianame, &diaitem);

        let unlock_height = height + cooldown;
        let cooldown_record = StakingRecord {
            stake_height: record.stake_height.clone(),
            unlock_height: BlockHeight::from(unlock_height),
            reward_index: reward_index.clone(),
            pending_reward: reward.clone(),
        };
        state.set_staking_record(&dianame, &cooldown_record);

        let mut global = state.staking_global();
        global.total_staked_shares =
            Uint5::from(global.total_staked_shares.uint().saturating_sub(1));
        state.set_staking_global(&global);

        let entry = StakingUnlockEntry {
            unlock_height: BlockHeight::from(unlock_height),
            diamond: dianame.clone(),
            staker: staker.clone(),
            reward,
        };
        staking_enqueue_unlock(state, &entry)?;

        staking_push_event(
            state,
            &StakingEvent {
                kind: STAKING_EVENT_UNSTAKE_REQUESTED,
                height: BlockHeight::from(height),
                diamond: dianame.clone(),
                staker: staker.clone(),
                unlock_height: BlockHeight::from(unlock_height),
                reward: entry.reward.clone(),
                shares: Uint5::from(0),
            },
        );
    }

    Ok(())
}

#[cfg(test)]
mod staking_tests {
    use super::*;
    use crate::core::state::ChainState;
    use crate::mint::operate::hacd_move_one_diamond;
    use tempfile::TempDir;

    fn test_state() -> (TempDir, ChainState) {
        let dir = TempDir::new().unwrap();
        let state = ChainState::open(dir.path());
        (dir, state)
    }

    fn test_staker() -> Address {
        Address::from_readable("12vi7DEZjh6KrK5PVmmqSgvuJPCsZMmpfi").unwrap()
    }

    fn test_other() -> Address {
        Address::from_readable("1LsQLqkd8FQDh3R7ZhxC5fndNf92WfhM19").unwrap()
    }

    fn seed_diamond(state: &mut ChainState, name: &str, owner: &Address) -> DiamondName {
        let dian = DiamondName::cons(name.as_bytes().try_into().unwrap());
        let dia = DiamondSto {
            status: DIAMOND_STATUS_NORMAL,
            address: owner.clone(),
            prev_engraved_height: BlockHeight::from(0),
            inscripts: Inscripts::default(),
        };
        let mut mint = MintState::wrap(state);
        mint.set_diamond(&dian, &dia);
        dian
    }

    fn one_diamond_list(name: &str) -> DiamondNameListMax200 {
        let mut list = DiamondNameListMax200::default();
        list.push(DiamondName::cons(name.as_bytes().try_into().unwrap()))
            .unwrap();
        list
    }

    fn hac_balance(state: &ChainState, addr: &Address) -> Amount {
        let core = CoreStateDisk::wrap(state);
        core.balance(addr)
            .map(|b| b.hacash.clone())
            .unwrap_or_default()
    }

    fn lit(name: &[u8; 6]) -> DiamondName {
        DiamondName::cons(*name)
    }

    #[test]
    fn hip25_testnet_seed_password_address() {
        use crate::core::account::Account;
        let acc = Account::create_by_password("hip25test").unwrap();
        eprintln!("HIP25_TESTNET_ADDRESS={}", acc.readable());
        let prikey = hex::encode(acc.secret_key().serialize());
        eprintln!("HIP25_TESTNET_PRIKEY={}", prikey);
    }

    #[test]
    fn fee_redirect_splits_13_87() {
        let (pool, burn) = staking_redirect_fee_zhu(1000);
        assert_eq!(pool, 130);
        assert_eq!(burn, 870);
    }

    #[test]
    fn min_stake_blocks_is_three_months_scale() {
        assert!(MIN_STAKE_BLOCKS > 20000);
        assert!(COOLDOWN_BLOCKS < 1000);
    }

    #[test]
    fn stake_owned_hacd_sets_staked_status() {
        let (_dir, mut state) = test_state();
        let staker = test_staker();
        let dian = seed_diamond(&mut state, "WTYUIA", &staker);
        let list = one_diamond_list("WTYUIA");
        let mut mint = MintState::wrap(&mut state);
        staking_apply_stake(&mut mint, &staker, &list, 1000).unwrap();
        let dia = mint.diamond(&dian).unwrap();
        assert_eq!(dia.status, DIAMOND_STATUS_STAKED);
        assert_eq!(mint.staking_global().total_staked_shares.uint(), 1);
    }

    #[test]
    fn transfer_staked_hacd_rejected() {
        let (_dir, mut state) = test_state();
        let staker = test_staker();
        let other = test_other();
        seed_diamond(&mut state, "WTYUIA", &staker);
        let list = one_diamond_list("WTYUIA");
        let mut mint = MintState::wrap(&mut state);
        staking_apply_stake(&mut mint, &staker, &list, 1000).unwrap();
        let dian = lit(b"WTYUIA");
        let err = hacd_move_one_diamond(&mut mint, &staker, &other, &dian).unwrap_err();
        assert!(format!("{}", err).contains("staked"));
    }

    #[test]
    fn unstake_before_min_stake_age_rejected() {
        let (_dir, mut state) = test_state();
        let staker = test_staker();
        seed_diamond(&mut state, "WTYUIA", &staker);
        let list = one_diamond_list("WTYUIA");
        let mut mint = MintState::wrap(&mut state);
        let stake_h = 1000u64;
        staking_apply_stake(&mut mint, &staker, &list, stake_h).unwrap();
        let too_early = stake_h + MIN_STAKE_BLOCKS - 1;
        let err = staking_apply_unstake(&mut mint, &staker, &list, too_early).unwrap_err();
        assert!(format!("{}", err).contains("at least"));
    }

    #[test]
    fn unstake_cooldown_unlock_pays_reward() {
        let (_dir, mut state) = test_state();
        let staker = test_staker();
        seed_diamond(&mut state, "WTYUIA", &staker);
        let list = one_diamond_list("WTYUIA");
        let stake_h = 1000u64;
        let mut mint = MintState::wrap(&mut state);
        staking_apply_stake(&mut mint, &staker, &list, stake_h).unwrap();
        staking_deposit_fee(&mut mint, 1000);
        staking_distribute_rewards(&mut mint, stake_h).unwrap();
        let unstake_h = stake_h + MIN_STAKE_BLOCKS;
        staking_apply_unstake(&mut mint, &staker, &list, unstake_h).unwrap();
        let unlock_h = unstake_h + COOLDOWN_BLOCKS;
        staking_on_block_close(&mut state, unlock_h).unwrap();
        let mint = MintStateDisk::wrap(&state);
        let dian = lit(b"WTYUIA");
        let dia = mint.diamond(&dian).unwrap();
        assert_eq!(dia.status, DIAMOND_STATUS_NORMAL);
        assert!(mint.staking_record(&dian).is_none());
        assert!(hac_balance(&state, &staker).is_positive());
    }

    #[test]
    fn two_stakers_split_rewards_proportionally() {
        let (_dir, mut state) = test_state();
        let s1 = test_staker();
        let s2 = test_other();
        seed_diamond(&mut state, "WTYUIA", &s1);
        seed_diamond(&mut state, "HXVMEK", &s2);
        let mut mint = MintState::wrap(&mut state);
        staking_apply_stake(&mut mint, &s1, &one_diamond_list("WTYUIA"), 100).unwrap();
        staking_apply_stake(&mut mint, &s2, &one_diamond_list("HXVMEK"), 100).unwrap();
        staking_deposit_fee(&mut mint, 1000);
        staking_distribute_rewards(&mut mint, 100).unwrap();
        let g = mint.staking_global();
        assert_eq!(g.global_reward_index.uint(), 500);
        let r1 = mint.staking_record(&lit(b"WTYUIA")).unwrap();
        staking_apply_unstake(
            &mut mint,
            &s1,
            &one_diamond_list("WTYUIA"),
            100 + MIN_STAKE_BLOCKS,
        )
        .unwrap();
        let pending = r1.reward_index.uint();
        let accrued = g.global_reward_index.uint().saturating_sub(pending);
        assert_eq!(accrued, 500);
    }

    #[test]
    fn pause_rejects_stake_allows_unstake() {
        let (_dir, mut state) = test_state();
        let staker = test_staker();
        seed_diamond(&mut state, "WTYUIA", &staker);
        seed_diamond(&mut state, "HXVMEK", &staker);
        let list = one_diamond_list("WTYUIA");
        let mut mint = MintState::wrap(&mut state);
        staking_apply_stake(&mut mint, &staker, &list, 1000).unwrap();
        staking_set_paused(&mut mint, true);
        let err =
            staking_apply_stake(&mut mint, &staker, &one_diamond_list("HXVMEK"), 2000).unwrap_err();
        assert!(format!("{}", err).contains("paused"));
        staking_apply_unstake(&mut mint, &staker, &list, 1000 + MIN_STAKE_BLOCKS).unwrap();
    }

    #[test]
    fn batch_over_200_rejected() {
        let mut list = DiamondNameListMax200::default();
        let chars = b"WTYUIAHXVMEKBSZN";
        for i in 0..201usize {
            let mut bytes = [b'W'; 6];
            for j in 0..6 {
                bytes[j] = chars[(i + j) % chars.len()];
            }
            list.push(DiamondName::cons(bytes)).unwrap();
        }
        let err = list.check().unwrap_err();
        assert!(format!("{}", err).contains("200"));
    }

    #[test]
    fn transfer_fee_redirect_uses_total_fee_not_fee_got() {
        let total = Amount::from_zhu(1000).unwrap();
        let (pool, miner) = staking_split_transfer_tx_fee(&total, false).unwrap();
        assert_eq!(pool, 130);
        assert_eq!(miner.to_zhu_unsafe() as u64, 870);
    }

    #[test]
    fn transfer_fee_redirect_applies_burn_90_on_remainder() {
        let total = Amount::from_zhu(1000).unwrap();
        let (pool, miner) = staking_split_transfer_tx_fee(&total, true).unwrap();
        assert_eq!(pool, 130);
        assert_eq!(miner.to_zhu_unsafe() as u64, 87);
    }

    #[test]
    fn cooldown_display_reward_uses_pending_not_live_index() {
        let (_dir, mut state) = test_state();
        let s1 = test_staker();
        let s2 = test_other();
        seed_diamond(&mut state, "WTYUIA", &s1);
        seed_diamond(&mut state, "HXVMEK", &s2);
        let list1 = one_diamond_list("WTYUIA");
        let stake_h = 1000u64;
        let mut mint = MintState::wrap(&mut state);
        staking_apply_stake(&mut mint, &s1, &list1, stake_h).unwrap();
        staking_apply_stake(&mut mint, &s2, &one_diamond_list("HXVMEK"), stake_h).unwrap();
        staking_deposit_fee(&mut mint, 1000);
        staking_distribute_rewards(&mut mint, stake_h).unwrap();
        let pending_at_unstake = staking_accrued_amount(
            &mint.staking_global().global_reward_index,
            &mint.staking_record(&lit(b"WTYUIA")).unwrap().reward_index,
        )
        .unwrap();
        let unstake_h = stake_h + MIN_STAKE_BLOCKS;
        staking_apply_unstake(&mut mint, &s1, &list1, unstake_h).unwrap();
        staking_deposit_fee(&mut mint, 2000);
        staking_distribute_rewards(&mut mint, unstake_h).unwrap();
        let rec = mint.staking_record(&lit(b"WTYUIA")).unwrap();
        let displayed = staking_display_accrued_reward(
            &mint.staking_global().global_reward_index,
            &rec,
        )
        .unwrap();
        assert_eq!(displayed, pending_at_unstake);
        let live = staking_accrued_amount(
            &mint.staking_global().global_reward_index,
            &rec.reward_index,
        )
        .unwrap();
        assert!(live > displayed);
    }

    #[test]
    fn stake_before_activation_height_rejected() {
        let (_dir, mut state) = test_state();
        let staker = test_staker();
        seed_diamond(&mut state, "WTYUIA", &staker);
        let list = one_diamond_list("WTYUIA");
        let mut mint = MintState::wrap(&mut state);
        let mut global = mint.staking_global();
        global.activation_height = BlockHeight::from(5000);
        mint.set_staking_global(&global);
        let err = staking_apply_stake(&mut mint, &staker, &list, 1000).unwrap_err();
        assert!(format!("{}", err).contains("not active"));
    }

    #[test]
    fn stake_emits_staked_on_chain_event() {
        let (_dir, mut state) = test_state();
        let staker = test_staker();
        seed_diamond(&mut state, "WTYUIA", &staker);
        let list = one_diamond_list("WTYUIA");
        let mut mint = MintState::wrap(&mut state);
        staking_apply_stake(&mut mint, &staker, &list, 1000).unwrap();
        assert_eq!(mint.staking_global().event_log_tail.uint(), 1);
        let ev = mint.staking_event(&Uint5::from(0)).unwrap();
        assert_eq!(ev.kind, STAKING_EVENT_STAKED);
        assert_eq!(ev.diamond.readable(), "WTYUIA");
        assert_eq!(ev.staker, staker);
    }

    #[test]
    fn script_execute_stake_via_hvm_wire() {
        use crate::vm::exec_staking_script;
        let (_dir, mut state) = test_state();
        let staker = test_staker();
        seed_diamond(&mut state, "WTYUIA", &staker);
        let mut wire = vec![STAKE_HACD_VMKIND];
        wire.extend(one_diamond_list("WTYUIA").serialize());
        exec_staking_script(&wire, &staker, 5000, &mut state).unwrap();
        let mint = MintStateDisk::wrap(&state);
        assert_eq!(mint.diamond(&lit(b"WTYUIA")).unwrap().status, DIAMOND_STATUS_STAKED);
    }

    #[test]
    fn hvm_opcode_stake_and_unstake_via_bridge() {
        let (_dir, mut state) = test_state();
        let staker = test_staker();
        seed_diamond(&mut state, "WTYUIA", &staker);
        let wire = one_diamond_list("WTYUIA").serialize();
        staking_exec_hvm_external(STAKE_HACD_VMKIND, &wire, &staker, 5000, &mut state).unwrap();
        let mint = MintStateDisk::wrap(&state);
        let dian = lit(b"WTYUIA");
        assert_eq!(mint.diamond(&dian).unwrap().status, DIAMOND_STATUS_STAKED);
        staking_exec_hvm_external(
            UNSTAKE_HACD_VMKIND,
            &wire,
            &staker,
            5000 + MIN_STAKE_BLOCKS,
            &mut state,
        )
        .unwrap();
        let mint = MintStateDisk::wrap(&state);
        assert_eq!(mint.diamond(&dian).unwrap().status, DIAMOND_STATUS_STAKING_COOLDOWN);
    }

    #[test]
    fn unlock_queue_missing_entry_fails_hard() {
        let (_dir, mut state) = test_state();
        let staker = test_staker();
        seed_diamond(&mut state, "WTYUIA", &staker);
        let list = one_diamond_list("WTYUIA");
        let stake_h = 1000u64;
        let mut mint = MintState::wrap(&mut state);
        staking_apply_stake(&mut mint, &staker, &list, stake_h).unwrap();
        staking_apply_unstake(&mut mint, &staker, &list, stake_h + MIN_STAKE_BLOCKS).unwrap();
        mint.del_staking_unlock_entry(&Uint5::from(0));
        let err = staking_process_unlock_queue(&mut state, stake_h + MIN_STAKE_BLOCKS + COOLDOWN_BLOCKS)
            .unwrap_err();
        assert!(format!("{}", err).contains("unlock queue corrupted"));
    }

    #[test]
    fn hip25_dev_flags_rejected_on_mainnet_chain_id() {
        use crate::config::{HIP25_DEV_CHAIN_ID, MintConf};
        let mut cnf = MintConf {
            chain_id: HIP25_DEV_CHAIN_ID + 99,
            difficulty_adjust_blocks: 288,
            each_block_target_time: 300,
            _test_mul: 1,
            staking_activation_height: 1,
            hip25_testnet_seed: true,
            hip25_testnet_seed_password: "hip25test".to_string(),
            hip25_testnet_demo_periods: false,
        };
        assert!(cnf.validate_hip25_dev_flags().is_err());
    }
}