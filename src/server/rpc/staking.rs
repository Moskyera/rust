
use crate::mint::component::*;
use crate::mint::operate::staking_display_accrued_reward;

defineQueryObject!{ QStakingStatus,
    diamond, String, s!(""),
}

async fn staking_status(State(ctx): State<ApiCtx>, q: Query<QStakingStatus>) -> impl IntoResponse {
    ctx_mintstate!(ctx, mintstate);
    q_unit!(q, unit);
    let name = q.diamond.replace(" ", "").replace("\n", "");
    if !DiamondName::is_valid(name.as_bytes()) {
        return api_error("diamond name error");
    }
    let dian = DiamondName::cons(name.as_bytes().try_into().unwrap());
    let diaobj = mintstate.diamond(&dian);
    if diaobj.is_none() {
        return api_error("cannot find diamond");
    }
    let diaobj = diaobj.unwrap();
    let global = mintstate.staking_global();
    let status = staking_status_label(&diaobj.status);
    let mut stake_height = 0u64;
    let mut unlock_height = 0u64;
    let mut min_unstake_height = 0u64;
    let mut accrued_reward = "0".to_string();
    if let Some(rec) = mintstate.staking_record(&dian) {
        stake_height = rec.stake_height.uint();
        unlock_height = rec.unlock_height.uint();
        if stake_height > 0 {
            min_unstake_height = stake_height + MIN_STAKE_BLOCKS;
        }
        if let Ok(amt) = staking_display_accrued_reward(&global.global_reward_index, &rec) {
            accrued_reward = amt.to_unit_string(&unit);
        }
    }
    let data = jsondata!{
        "literal", dian.readable(),
        "status", status,
        "staker", diaobj.address.readable(),
        "accrued_reward", accrued_reward,
        "min_unstake_height", min_unstake_height,
        "unlock_height", unlock_height,
        "stake_height", stake_height,
    };
    api_data(data)
}

defineQueryObject!{ QStakingSummary,
    address, String, s!(""),
}

async fn staking_summary(State(ctx): State<ApiCtx>, q: Query<QStakingSummary>) -> impl IntoResponse {
    ctx_mintstate!(ctx, mintstate);
    q_unit!(q, unit);
    let ads = q.address.replace(" ", "").replace("\n", "");
    let adr = match Address::from_readable(&ads) {
        Ok(a) => a,
        Err(_) => return api_error("address format error"),
    };
    let owned = mintstate.diamond_owned(&adr).unwrap_or_default();
    let names = owned.readable();
    let global = mintstate.staking_global();
    let mut staked_count = 0u64;
    let mut cooldown_count = 0u64;
    let mut total_accrued = Amount::default();
    let l = DiamondName::width();
    let bytes = names.as_bytes();
    for i in (0..bytes.len()).step_by(l) {
        if i + l > bytes.len() {
            break;
        }
        let dian = DiamondName::cons(bytes[i..i + l].try_into().unwrap());
        let Some(diaobj) = mintstate.diamond(&dian) else {
            continue;
        };
        if diaobj.status == DIAMOND_STATUS_STAKED {
            staked_count += 1;
        } else if diaobj.status == DIAMOND_STATUS_STAKING_COOLDOWN {
            cooldown_count += 1;
        }
        if let Some(rec) = mintstate.staking_record(&dian) {
            if let Ok(amt) = staking_display_accrued_reward(&global.global_reward_index, &rec) {
                total_accrued = total_accrued.add(&amt).unwrap_or(total_accrued);
            }
        }
    }
    let data = jsondata!{
        "staked_count", staked_count,
        "cooldown_count", cooldown_count,
        "total_accrued_reward", total_accrued.to_unit_string(&unit),
    };
    api_data(data)
}

defineQueryObject!{ QStakingGlobal,
    __nnn_, Option<bool>, None,
}

async fn staking_global(State(ctx): State<ApiCtx>, _q: Query<QStakingGlobal>) -> impl IntoResponse {
    ctx_mintstate!(ctx, mintstate);
    let global = mintstate.staking_global();
    let data = jsondata!{
        "total_staked_shares", global.total_staked_shares.uint(),
        "reward_pool_pending_zhu", global.reward_pool_zhu.uint(),
        "global_reward_index", global.global_reward_index.uint(),
        "activation_height", global.activation_height.uint(),
        "event_count", global.event_log_tail.uint(),
        "paused", global.is_paused(),
    };
    api_data(data)
}

defineQueryObject!{ QStakingEvents,
    from, String, s!("0"),
    limit, String, s!("50"),
}

async fn staking_events(State(ctx): State<ApiCtx>, q: Query<QStakingEvents>) -> impl IntoResponse {
    ctx_mintstate!(ctx, mintstate);
    q_unit!(q, unit);
    let from = q.from.parse::<u64>().unwrap_or(0);
    let mut limit = q.limit.parse::<u64>().unwrap_or(50);
    if limit == 0 {
        limit = 50;
    }
    if limit > 200 {
        limit = 200;
    }
    let global = mintstate.staking_global();
    let tail = global.event_log_tail.uint();
    let end = (from + limit).min(tail);
    let mut items: Vec<serde_json::Value> = Vec::new();
    for id in from..end {
        let Some(ev) = mintstate.staking_event(&Uint5::from(id)) else {
            continue;
        };
        let literal = if ev.diamond.readable().trim().is_empty() {
            "".to_string()
        } else {
            ev.diamond.readable()
        };
        let staker = if ev.staker.readable().is_empty() {
            "".to_string()
        } else {
            ev.staker.readable()
        };
        items.push(json!({
            "id": id,
            "kind": staking_event_kind_label(&ev.kind),
            "height": ev.height.uint(),
            "literal": literal,
            "staker": staker,
            "unlock_height": ev.unlock_height.uint(),
            "reward": ev.reward.to_unit_string(&unit),
            "shares": ev.shares.uint(),
        }));
    }
    let data = jsondata!{
        "from", from,
        "limit", limit,
        "total", tail,
        "events", items,
    };
    api_data(data)
}