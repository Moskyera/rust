
/**
 * HIP-2 v2: HACD system mortgage — open (15) and redeem (16).
 */
ActionDefine!{
    MortgageOpen : 15, (
        lending_id        : DiamondSyslendId
        mortgage_diamonds : DiamondNameListMax200
        loan_total_amount : Amount
        borrow_period     : Uint1
    ),
    ACTLV_MAIN,
    21,
    (self, ctx, state, store, gas),
    false,
    [],
    {
        gas += self.mortgage_diamonds.count().uint() as i64 * DiamondName::width() as i64;
        mortgage_open(self, ctx, state, store)
    }
}

ActionDefine!{
    MortgageRedeem : 16, (
        lending_id   : DiamondSyslendId
        ransom_amount : Amount
    ),
    ACTLV_MAIN,
    21,
    (self, ctx, state, store, gas),
    false,
    [],
    mortgage_redeem(self, ctx, state, store)
}

fn mortgage_open(
    this: &MortgageOpen,
    ctx: &dyn ExecContext,
    sta: &mut dyn State,
    sto: &dyn Store,
) -> Ret<Vec<u8>> {
    let owner = ctx.main_address();
    let height = ctx.pending_height();
    mortgage_apply_open(
        sta,
        sto,
        owner,
        &this.lending_id,
        &this.mortgage_diamonds,
        &this.loan_total_amount,
        this.borrow_period.uint() as u8,
        height,
    )?;
    Ok(vec![])
}

fn mortgage_redeem(
    this: &MortgageRedeem,
    ctx: &dyn ExecContext,
    sta: &mut dyn State,
    _sto: &dyn Store,
) -> Ret<Vec<u8>> {
    let redeemer = ctx.main_address();
    let height = ctx.pending_height();
    mortgage_apply_redeem(
        sta,
        redeemer,
        &this.lending_id,
        &this.ransom_amount,
        height,
    )?;
    Ok(vec![])
}