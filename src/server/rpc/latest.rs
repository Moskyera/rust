

defineQueryObject!{ Q4376,
    __nnn_, Option<bool>, None,
}

async fn latest(State(ctx): State<ApiCtx>, q: Query<Q4376>) -> impl IntoResponse {
    ctx_mintstate!(ctx, mintstate);
    //
    let lasthei = ctx.engine.latest_block().objc().height().uint();
    let lastdia = mintstate.latest_diamond();
    // return data
    let chain_id = ctx.engine.config().chain_id;
    let mut data = jsondata!{
        "height", lasthei,
        "diamond", lastdia.number.uint(),
        "chain_id", chain_id,
        "hip25_dev", chain_id == crate::config::HIP25_DEV_CHAIN_ID,
    };
    api_data(data)
}

