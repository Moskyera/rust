
use crate::mint::component::*;
use crate::mint::operate::mortgage_calc_ransom;

defineQueryObject!{ QMortgageGlobal,
    __nnn_, Option<bool>, None,
}

async fn mortgage_global(State(ctx): State<ApiCtx>, _q: Query<QMortgageGlobal>) -> impl IntoResponse {
    ctx_mintstate!(ctx, mintstate);
    let global = mintstate.mortgage_global();
    let data = jsondata!{
        "outstanding_ioo_zhu", global.outstanding_ioo_zhu.uint(),
        "cumulative_loan_zhu", global.cumulative_loan_zhu.uint(),
        "cumulative_ransom_burn_zhu", global.cumulative_ransom_burn_zhu.uint(),
        "cumulative_origination_burn_zhu", global.cumulative_origination_burn_zhu.uint(),
        "active_contracts", global.active_contracts.uint(),
        "activation_height", global.activation_height.uint(),
        "max_outstanding_ioo_zhu", global.max_outstanding_ioo_zhu.uint(),
        "period_blocks", global.effective_period_blocks(),
        "origination_fee_bps", MORTGAGE_ORIGINATION_FEE_BPS,
        "early_interest_bps_per_period", MORTGAGE_EARLY_INTEREST_BPS_PER_PERIOD,
        "committed_interest_bps_per_period", MORTGAGE_COMMITTED_INTEREST_BPS_PER_PERIOD,
        "auction_floor_bps", MORTGAGE_AUCTION_FLOOR_BPS,
    };
    api_data(data)
}

defineQueryObject!{ QMortgageContract,
    id, String, s!(""),
    redeemer, String, s!(""),
    height, String, s!(""),
}

async fn mortgage_contract(State(ctx): State<ApiCtx>, q: Query<QMortgageContract>) -> impl IntoResponse {
    ctx_mintstate!(ctx, mintstate);
    q_unit!(q, unit);
    let id_hex = q.id.replace(" ", "").replace("\n", "");
    let id_bytes = match hex::decode(&id_hex) {
        Ok(b) if b.len() == DIAMOND_SYSLEND_ID_SIZE => b,
        _ => return api_error("mortgage lending id hex error"),
    };
    let lend_id = DiamondSyslendId::cons(id_bytes.try_into().unwrap());
    let contract = mintstate.diamond_syslend(&lend_id);
    if contract.is_none() {
        return api_error("mortgage contract not found");
    }
    let contract = contract.unwrap();
    let global = mintstate.mortgage_global();
    let period_blocks = global.effective_period_blocks();

    let mut phase = "".to_string();
    let mut min_ransom = "0".to_string();
    if !contract.redeemed() {
        let redeemer_ads = q.redeemer.replace(" ", "").replace("\n", "");
        let height = q.height.parse::<u64>().unwrap_or_else(|_| {
            ctx.engine.latest_block().objc().height().uint()
        });
        let redeemer = if redeemer_ads.is_empty() {
            contract.main_address.clone()
        } else {
            match Address::from_readable(&redeemer_ads) {
                Ok(a) => a,
                Err(_) => return api_error("redeemer address format error"),
            }
        };
        if let Ok((ph, amt)) = mortgage_calc_ransom(&contract, &redeemer, height, period_blocks) {
            phase = ph.label().to_string();
            min_ransom = amt.to_unit_string(&unit);
        }
    }

    let diamonds: Vec<String> = contract
        .mortgage_diamonds
        .list()
        .iter()
        .map(|d| d.readable())
        .collect();

    let data = jsondata!{
        "lending_id", id_hex,
        "redeemed", contract.redeemed(),
        "create_height", contract.create_block_height.uint(),
        "main_address", contract.main_address.readable(),
        "loan_principal", contract.loan_principal.to_unit_string(&unit),
        "borrow_period", contract.borrow_period.uint(),
        "diamonds", diamonds,
        "ransom_height", contract.ransom_block_height.uint(),
        "ransom_address", contract.ransom_address.readable(),
        "redeem_phase", phase,
        "min_ransom", min_ransom,
        "period_blocks", period_blocks,
    };
    api_data(data)
}