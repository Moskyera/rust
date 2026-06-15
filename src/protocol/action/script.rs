



/**
 * execute script
 */
 ActionDefine!{
    ScriptExecute : 37, (
        mark: Fixed1
        vern: Fixed1
        codes: BytesW2
    ),
    ACTLV_TOP, // level
    11, // gas = 32
    (self, ctx, state, store, gas), // params
    true, // burn 90
    [], // req sign
    {
        script_execute_staking(self, ctx, state)
    }
}

fn script_execute_staking(
    this: &ScriptExecute,
    ctx: &dyn ExecContext,
    sta: &mut dyn State,
) -> Ret<Vec<u8>> {
    let staker = ctx.main_address();
    let height = ctx.pending_height();
    let codes = this.codes.as_ref();
    vm::exec_staking_script(codes, staker, height, ctx.chain_id(), sta)?;
    Ok(vec![])
}



