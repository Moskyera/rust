

pub fn routes(mut ctx: ApiCtx) -> Router {

    use ctx::*;
    
    let lrt = Router::new()
    .route("/hip25/wallet", get(hip25_wallet_page))
    .route("/pkg/hacash_sdk.js", get(hip25_sdk_js))
    .route("/pkg/hacash_sdk_bg.wasm", get(hip25_sdk_wasm))
    .route("/", get(console))
    
    // query
    .route(&query("latest"), get(latest))
    .route(&query("supply"), get(supply))
    .route(&query("hashrate"), get(hashrate))
    .route(&query("hashrate/logs"), get(hashrate_logs))
    .route(&query("balance"), get(balance))
    .route(&query("channel"), get(channel))
    .route(&query("coin/transfer"), get(scan_coin_transfer))

    .route(&query("block/intro"), get(block_intro))
    .route(&query("block/recents"), get(block_recents))
    .route(&query("block/views"), get(block_views))
    .route(&query("block/datas"), get(block_datas))

    .route(&query("transaction"), get(transaction_exist))

    .route(&query("diamond"), get(diamond))
    .route(&query("diamond/bidding"), get(diamond_bidding))
    .route(&query("diamond/views"), get(diamond_views))
    .route(&query("diamond/engrave"), get(diamond_engrave))
    .route(&query("diamond/inscription_protocol_cost"), get(diamond_inscription_protocol_cost))

    .route(&query("staking/status"), get(staking_status))
    .route(&query("staking/summary"), get(staking_summary))
    .route(&query("staking/global"), get(staking_global))
    .route(&query("staking/events"), get(staking_events))

    .route(&query("mortgage/global"), get(mortgage_global))
    .route(&query("mortgage/contract"), get(mortgage_contract))

    .route(&query("fee/average"), get(fee_average))

    .route(&query("miner/notice"), get(miner_notice))
    .route(&query("miner/pending"), get(miner_pending))
    .route(&query("diamondminer/init"), get(diamondminer_init))

    // create
    .route(&create("account"), get(account))
    .route(&create("transaction"), post(transaction_build))
    .route(&create("coin/transfer"), post(create_coin_transfer))
    
    // submit
    .route(&submit("transaction"), post(submit_transaction))
    .route(&submit("block"), post(submit_block))
    .route(&submit("miner/success"), get(miner_success))
    .route(&submit("diamondminer/success"), post(diamondminer_success))

    // operate
    .route(&operate("fee/raise"), post(raise_fee))

    // util
    .route(&util("transaction/check"), post(transaction_check))
    .route(&util("transaction/sign"), post(transaction_sign))


    ;

    // merge extend (unstable test routes disabled in release builds)
    let mut router = Router::new().merge(lrt).merge(extend::routes());
    #[cfg(debug_assertions)]
    {
        router = router.merge(unstable::routes());
    }
    router.with_state(ctx)
    
}



