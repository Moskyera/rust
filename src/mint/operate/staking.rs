
fn staking_accrued_zhu(global_index: &Uint8, snapshot: &Uint8) -> u64 {
    global_index.uint().saturating_sub(snapshot.uint())
}

fn staking_accrued_amount(global_index: &Uint8, snapshot: &Uint8) -> Ret<Amount> {
    let zhu = staking_accrued_zhu(global_index, snapshot) as i64;
    if zhu <= 0 {
        return Ok(Amount::default());
    }
    Amount::from_zhu(zhu)
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

pub fn staking_deposit_fee(state: &mut MintState, fee_zhu: u64) {
    if fee_zhu == 0 {
        return;
    }
    let mut global = state.staking_global();
    global.reward_pool_zhu = Uint8::from(global.reward_pool_zhu.uint() + fee_zhu);
    state.set_staking_global(&global);
}

pub fn staking_distribute_rewards(state: &mut MintState) -> Ret<()> {
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
                    head += 1;
                    continue;
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
    {
        let mut mint_state = MintState::wrap(base_state);
        staking_distribute_rewards(&mut mint_state)?;
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

pub fn staking_apply_stake(
    state: &mut MintState,
    staker: &Address,
    diamonds: &DiamondNameListMax200,
    height: u64,
) -> Ret<()> {
    diamonds.check()?;
    let mut global = state.staking_global();
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
    }

    state.set_staking_global(&global);
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
        if height < stake_height + MIN_STAKE_BLOCKS {
            return errf!(
                "diamond {} must remain staked for at least {} blocks (~3 months)",
                dianame.readable(),
                MIN_STAKE_BLOCKS
            );
        }

        let reward = staking_accrued_amount(&reward_index, &record.reward_index)?;

        diaitem.status = DIAMOND_STATUS_STAKING_COOLDOWN;
        state.set_diamond(&dianame, &diaitem);

        let unlock_height = height + COOLDOWN_BLOCKS;
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
    }

    Ok(())
}

#[cfg(test)]
mod staking_tests {
    use super::*;

    #[test]
    fn fee_redirect_splits_40_60() {
        let (pool, burn) = staking_redirect_fee_zhu(1000);
        assert_eq!(pool, 400);
        assert_eq!(burn, 600);
    }

    #[test]
    fn min_stake_blocks_is_three_months_scale() {
        assert!(MIN_STAKE_BLOCKS > 20000);
        assert!(COOLDOWN_BLOCKS < 1000);
    }
}