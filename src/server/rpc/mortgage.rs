
use crate::mint::component::*;
use crate::mint::operate::{mortgage_calc_ransom, mortgage_compute_principal};

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
        "early_grace_periods", MORTGAGE_EARLY_GRACE_PERIODS,
        "early_interest_bps_per_period", MORTGAGE_EARLY_INTEREST_BPS_PER_PERIOD,
        "apr_bps", MORTGAGE_APR_BPS,
        "blocks_per_year", MORTGAGE_BLOCKS_PER_YEAR,
        "auction_floor_bps", MORTGAGE_AUCTION_FLOOR_BPS,
        "economics_version", "v2.1",
    };
    api_data(data)
}

defineQueryObject!{ QMortgageContract,
    id, String, s!(""),
    redeemer, String, s!(""),
    height, Option<String>, None,
}

defineQueryObject!{ QMortgagePortfolio,
    address, String, s!(""),
}

defineQueryObject!{ QMortgagePrincipal,
    diamonds, String, s!(""),
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
        let height = q
            .height
            .as_deref()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or_else(|| ctx.engine.latest_block().objc().height().uint());
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

async fn mortgage_portfolio(State(ctx): State<ApiCtx>, q: Query<QMortgagePortfolio>) -> impl IntoResponse {
    ctx_mintstate!(ctx, mintstate);
    q_unit!(q, unit);
    let ads = q.address.replace(" ", "").replace("\n", "");
    let adr = match Address::from_readable(&ads) {
        Ok(a) => a,
        Err(_) => return api_error("address format error"),
    };
    let global = mintstate.mortgage_global();
    let period_blocks = global.effective_period_blocks();
    let height = ctx.engine.latest_block().objc().height().uint();
    let index = mintstate.mortgage_owner_index(&adr).unwrap_or_default();
    let mut contracts: Vec<serde_json::Value> = Vec::new();
    for lend_id in index.iter_ids() {
        let Some(contract) = mintstate.diamond_syslend(&lend_id) else {
            continue;
        };
        if contract.redeemed() {
            continue;
        }
        let id_hex = hex::encode(lend_id.as_ref());
        let diamonds: Vec<String> = contract
            .mortgage_diamonds
            .list()
            .iter()
            .map(|d| d.readable())
            .collect();
        let (phase, min_ransom) = mortgage_calc_ransom(&contract, &adr, height, period_blocks)
            .map(|(ph, amt)| (ph.label().to_string(), amt.to_unit_string(&unit)))
            .unwrap_or_else(|_| ("".to_string(), "0".to_string()));
        contracts.push(json!({
            "lending_id": id_hex,
            "loan_principal": contract.loan_principal.to_unit_string(&unit),
            "borrow_period": contract.borrow_period.uint(),
            "diamonds": diamonds,
            "create_height": contract.create_block_height.uint(),
            "redeem_phase": phase,
            "min_ransom": min_ransom,
        }));
    }
    let data = jsondata!{
        "address", adr.readable(),
        "active_count", contracts.len() as u64,
        "contracts", contracts,
        "economics_version", "v2.1",
    };
    api_data(data)
}

async fn mortgage_principal(State(ctx): State<ApiCtx>, q: Query<QMortgagePrincipal>) -> impl IntoResponse {
    ctx_mintstore!(ctx, mintstore);
    q_unit!(q, unit);
    let dialist = DiamondNameListMax200::from_readable(&q.diamonds.replace(" ", ""));
    let list = match dialist {
        Ok(l) => l,
        Err(e) => return api_error(&format!("diamonds {}", e)),
    };
    if list.count().uint() == 0 {
        return api_error("diamonds required");
    }
    let principal = match mortgage_compute_principal(&mintstore, &list) {
        Ok(p) => p,
        Err(e) => return api_error(&e),
    };
    let data = jsondata!{
        "loan", principal.to_unit_string(&unit),
        "diamonds", list.readable(),
        "origination_fee_bps", MORTGAGE_ORIGINATION_FEE_BPS,
    };
    api_data(data)
}