
/**
 * HIP-25: Diamond Stake / Unstake actions
 */
ActionDefine!{
    DiamondStake : 34, (
        diamonds : DiamondNameListMax200
    ),
    ACTLV_MAIN,
    21,
    (self, ctx, state, store, gas),
    true,
    [],
    {
        gas += self.diamonds.count().uint() as i64 * DiamondName::width() as i64;
        diamond_stake(self, ctx, state, store)
    }
}

ActionDefine!{
    DiamondUnstake : 35, (
        diamonds : DiamondNameListMax200
    ),
    ACTLV_MAIN,
    21,
    (self, ctx, state, store, gas),
    true,
    [],
    {
        gas += self.diamonds.count().uint() as i64 * DiamondName::width() as i64;
        diamond_unstake(self, ctx, state, store)
    }
}

fn diamond_stake(
    this: &DiamondStake,
    ctx: &dyn ExecContext,
    sta: &mut dyn State,
    _sto: &dyn Store,
) -> Ret<Vec<u8>> {
    let staker = ctx.main_address();
    let height = ctx.pending_height();
    let mut state = MintState::wrap(sta);
    staking_apply_stake(&mut state, staker, &this.diamonds, height)?;
    Ok(vec![])
}

fn diamond_unstake(
    this: &DiamondUnstake,
    ctx: &dyn ExecContext,
    sta: &mut dyn State,
    _sto: &dyn Store,
) -> Ret<Vec<u8>> {
    let staker = ctx.main_address();
    let height = ctx.pending_height();
    let mut state = MintState::wrap(sta);
    staking_apply_unstake(&mut state, staker, &this.diamonds, height)?;
    Ok(vec![])
}