
use crate::mint::coinbase::*;

defineQueryObject!{ Q9364,
    __nnn_, Option<bool>, None,
}

async fn supply(State(ctx): State<ApiCtx>, q: Query<Q9364>) -> impl IntoResponse {
    ctx_state!(ctx, state);
    ctx_mintstate!(ctx, mintstate);
    //
    let lasthei = ctx.engine.latest_block().objc().height().uint();
    let lastdia = mintstate.latest_diamond();
    // total supply
    const ZHU: u64 = 1_0000_0000;
    let supply = mintstate.total_count();
    let mortgage = mintstate.mortgage_global();
    let blk_rwd = cumulative_block_reward(lasthei) * ZHU;
    let mortgage_burn = mortgage.cumulative_origination_burn_zhu.uint()
        + mortgage.cumulative_ransom_burn_zhu.uint();
    let burn_fee = *supply.hacd_bid_burn_zhu + *supply.diamond_insc_burn_zhu + mortgage_burn;
    let curr_ccl = blk_rwd + *supply.channel_interest_zhu - burn_fee;
    //
    let z2m = |zhu|zhu as f64  / ZHU as f64;
    
    // return data
    let mut data = jsondata!{
        "latest_height", lasthei,
        "current_circulation", z2m(curr_ccl),

        "burned_fee",  z2m(burn_fee),
        "burned_diamond_bid", z2m(*supply.hacd_bid_burn_zhu),
        
        "channel_deposit", z2m(*supply.channel_deposit_zhu),
        "channel_interest", z2m(*supply.channel_interest_zhu),
        "channel_opening", *supply.opening_channel,
        
        "diamond_engraved", *supply.diamond_engraved,

        "transferred_bitcoin", 0,
        "trsbtc_subsidy", 0,

        "block_reward", z2m(blk_rwd),

        "mortgage_outstanding_ioo_zhu", mortgage.outstanding_ioo_zhu.uint(),
        "mortgage_active_contracts", mortgage.active_contracts.uint(),
        "mortgage_cumulative_loan_zhu", mortgage.cumulative_loan_zhu.uint(),
        "mortgage_origination_burn_zhu", mortgage.cumulative_origination_burn_zhu.uint(),
        "mortgage_ransom_burn_zhu", mortgage.cumulative_ransom_burn_zhu.uint(),
        "mortgage_economics_version", "v2.1",

        "minted_diamond", lastdia.number.uint(),
    };
    api_data(data)
}