use std::sync::Once;

use crate::core::account::Account;
use crate::core::field::DiamondNameListMax200;
use crate::interface::field::Field;
use crate::mint::action::{DiamondStake, DiamondUnstake};

static SDK_INIT: Once = Once::new();

fn ensure_sdk_init() {
    SDK_INIT.call_once(|| {
        crate::mint::action::init_reg();
    });
}
use crate::protocol::action::{HacToTransfer, SubChainID};
use crate::protocol::transaction::TransactionType2;

fn if_add_chain_id(chain_id: u64, tx: &mut TransactionType2) {
    if chain_id > 0 {
        let mut act = SubChainID::new();
        act.chain_id = Uint8::from(chain_id);
        let _ = tx.push_action(Box::new(act));
    }
}

fn get_time_set(timestamp: i64) -> i64 {
    let mut time_set = timestamp;
    if time_set <= 0 {
        time_set = Utc::now().timestamp();
    }
    time_set
}

fn parse_diamond_list(diamond_name_list: String) -> Result<DiamondNameListMax200, String> {
    DiamondNameListMax200::from_readable(&diamond_name_list).map_err(|e| e.to_string())
}

fn stake_tx_json(
    tx: &TransactionType2,
    dlist: &DiamondNameListMax200,
    fee: &Amount,
    acc: &Account,
    time_set: i64,
    action_label: &str,
) -> String {
    let ok = format!(
        r##""tx_hash":"{}","tx_body":"{}","action":"{}","diamond_count":{},"diamonds":"{}","fee":"{}","main_address":"{}","timestamp":{}"##,
        tx.hash().hex(),
        hex::encode(tx.serialize()),
        action_label,
        dlist.count().uint(),
        dlist.readable(),
        fee.to_fin_string(),
        acc.readable(),
        time_set
    );
    format!("{{{}}}", ok)
}

fn build_signed_stake_tx(
    chain_id: u64,
    from_pass: String,
    diamond_name_list: String,
    fee: String,
    timestamp: i64,
    stake: bool,
) -> String {
    ensure_sdk_init();
    let time_set = get_time_set(timestamp);
    let dlist = or_return! { "Diamond Name parse", parse_diamond_list(diamond_name_list) };
    let fee = or_return! { "Fee parse", Amount::from_string_unsafe(&fee) };
    let acc = or_return! { "From Account", Account::create_by(&from_pass) };
    let addr = or_return! { "From Address", Address::from_readable(acc.readable()) };
    let mut tx = TransactionType2::build(addr, fee.clone());
    tx.timestamp = Timestamp::from(time_set as u64);
    if_add_chain_id(chain_id, &mut tx);
    if stake {
        let mut act = DiamondStake::new();
        act.diamonds = dlist.clone();
        let _ = tx.push_action(Box::new(act));
    } else {
        let mut act = DiamondUnstake::new();
        act.diamonds = dlist.clone();
        let _ = tx.push_action(Box::new(act));
    }
    let _ = tx.fill_sign(&acc);
    let label = if stake { "stake" } else { "unstake" };
    stake_tx_json(&tx, &dlist, &fee, &acc, time_set, label)
}

#[wasm_bindgen]
pub fn trs_test(x: i32) -> usize {
    let mut bt = Fixed4::default();
    let data = vec![x as u8 + 1, x as u8 + 2, x as u8 + 3, x as u8 + 4];
    let mut res = bt.parse(&data, 0).unwrap();
    let vals = bt.serialize();
    res += 1;
    res = res + vals[x as usize] as usize;
    res += x as usize;
    let vvs = bt.hex().into_bytes();
    res += vvs[2] as usize;
    res
}

#[wasm_bindgen]
pub fn create_acc_random() -> usize {
    let acc = Account::create_by_password(&"123456".to_string());
    if let Err(_) = acc {
        return 0;
    }
    let accstr = acc.unwrap().readable().clone();
    let bts = accstr.as_bytes();
    bts[1] as usize
}

#[wasm_bindgen]
pub fn set_api_return_json() {}

#[cfg(not(target_arch = "wasm32"))]
#[wasm_bindgen]
pub fn general_transfer(
    chain_id: u64,
    from_pass: String,
    to_addr: String,
    amountex: String,
    fee: String,
    timestamp: i64,
) -> String {
    let amount = amountex.clone().to_uppercase().replace(" ", "");
    if DiamondNameListMax200::from_readable(&amount).is_ok() {
        return hacd_transfer(
            chain_id,
            from_pass.clone(),
            from_pass.clone(),
            to_addr,
            amount,
            fee,
            timestamp,
        );
    }
    let res2 = amount.find("SAT");
    if res2.is_some() {
        let v = amount.replace("S", "").replace("AT", "").replace("OHI", "");
        if let Ok(sat) = v.parse::<u64>() {
            return sat_transfer(
                chain_id,
                from_pass.clone(),
                from_pass.clone(),
                to_addr,
                sat,
                fee,
                timestamp,
            );
        }
    }
    if Amount::from_string_unsafe(&amount).is_ok() {
        return hac_transfer(chain_id, from_pass.clone(), to_addr, amount, fee, timestamp);
    }
    or_return! { "Amount format", Err(amount) };
    "[ERROR]".to_string()
}

#[cfg(not(target_arch = "wasm32"))]
#[wasm_bindgen]
pub fn hac_transfer(
    chain_id: u64,
    from_pass: String,
    to_addr: String,
    amount: String,
    fee: String,
    timestamp: i64,
) -> String {
    let time_set = get_time_set(timestamp);
    let amt = or_return! { "Amount parse", Amount::from_string_unsafe(&amount) };
    let fee = or_return! { "Fee parse", Amount::from_string_unsafe(&fee) };
    let acc = or_return! { "From Account", Account::create_by(&from_pass) };
    let toaddr = or_return! { "To Address", Address::from_readable(&to_addr) };
    let mut tx = TransactionType2::build(*acc.address(), fee.clone());
    tx.timestamp = Timestamp::from(time_set as u64);
    if_add_chain_id(chain_id, &mut tx);
    let mut act = HacToTransfer::new();
    act.to = AddrOrPtr::from_addr(toaddr.clone());
    act.hacash = amt.clone();
    let _ = tx.push_action(Box::new(act));
    let _ = tx.fill_sign(&acc);
    let ok = format!(
        r##""tx_hash":"{}","tx_body":"{}","amount":"{}","fee":"{}","payment_address":"{}","fee_address":"{}","collection_address":"{}","timestamp":{}"##,
        tx.hash().hex(),
        hex::encode(tx.serialize()),
        amt.to_fin_string(),
        fee.to_fin_string(),
        acc.readable(),
        acc.readable(),
        toaddr.readable(),
        time_set
    );
    format!("{{{}}}", ok)
}

#[cfg(not(target_arch = "wasm32"))]
#[wasm_bindgen]
pub fn sat_transfer(
    chain_id: u64,
    from_pass: String,
    fee_pass: String,
    to_addr: String,
    satoshi: u64,
    fee: String,
    timestamp: i64,
) -> String {
    let time_set = get_time_set(timestamp);
    let sat = Satoshi::from(satoshi);
    let fee = or_return! { "Fee parse", Amount::from_string_unsafe(&fee) };
    let acc = or_return! { "From Account", Account::create_by(&from_pass) };
    let feeacc = or_return! { "Fee Account", Account::create_by(&fee_pass) };
    let toaddr = or_return! { "To Address", Address::from_readable(&to_addr) };
    let is_main_single = feeacc.address() == acc.address();
    let mut tx = TransactionType2::build(*feeacc.address(), fee.clone());
    tx.timestamp = Timestamp::from(time_set as u64);
    if_add_chain_id(chain_id, &mut tx);
    if is_main_single {
        let mut act = SatoshiToTransfer::new();
        act.to = AddrOrPtr::from_addr(toaddr.clone());
        act.satoshi = sat.clone();
        let _ = tx.push_action(Box::new(act));
    } else {
        let mut act = SatoshiFromToTransfer::new();
        act.from = AddrOrPtr::from_addr(acc.address().clone());
        act.to = AddrOrPtr::from_addr(toaddr.clone());
        act.satoshi = sat.clone();
        let _ = tx.push_action(Box::new(act));
    }
    let _ = tx.fill_sign(&acc);
    if !is_main_single {
        let _ = tx.fill_sign(&feeacc);
    }
    let ok = format!(
        r##""tx_hash":"{}","tx_body":"{}","amount":"{} SAT","fee":"{}","payment_address":"{}","fee_address":"{}","collection_address":"{}","timestamp":{}"##,
        tx.hash().hex(),
        hex::encode(tx.serialize()),
        sat.to_u64(),
        fee.to_fin_string(),
        acc.readable(),
        feeacc.readable(),
        toaddr.readable(),
        time_set
    );
    format!("{{{}}}", ok)
}

#[cfg(not(target_arch = "wasm32"))]
#[wasm_bindgen]
pub fn hacd_transfer(
    chain_id: u64,
    from_pass: String,
    fee_pass: String,
    to_addr: String,
    diamond_name_list: String,
    fee: String,
    timestamp: i64,
) -> String {
    let time_set = get_time_set(timestamp);
    let dlist = or_return! { "Diamond Name parse", parse_diamond_list(diamond_name_list) };
    let fee = or_return! { "Fee parse", Amount::from_string_unsafe(&fee) };
    let acc = or_return! { "From Account", Account::create_by(&from_pass) };
    let feeacc = or_return! { "Fee Account", Account::create_by(&fee_pass) };
    let toaddr = or_return! { "To Address", Address::from_readable(&to_addr) };
    let is_main_single = feeacc.address() == acc.address();
    let mut tx = TransactionType2::build(*feeacc.address(), fee.clone());
    tx.timestamp = Timestamp::from(time_set as u64);
    if_add_chain_id(chain_id, &mut tx);
    if is_main_single && dlist.count().uint() == 1 {
        let mut act = DiamondSingleTransfer::new();
        act.diamond = dlist.lists[0].clone();
        act.to = AddrOrPtr::from_addr(toaddr.clone());
        let _ = tx.push_action(Box::new(act));
    } else {
        let mut act = DiamondFromToTransfer::new();
        act.from = AddrOrPtr::from_addr(acc.address().clone());
        act.to = AddrOrPtr::from_addr(toaddr.clone());
        act.diamonds = dlist.clone();
        let _ = tx.push_action(Box::new(act));
    }
    let _ = tx.fill_sign(&acc);
    if !is_main_single {
        let _ = tx.fill_sign(&feeacc);
    }
    let ok = format!(
        r##""tx_hash":"{}","tx_body":"{}","diamond_count":{},"diamonds":"{}","fee":"{}","payment_address":"{}","fee_address":"{}","collection_address":"{}","timestamp":{}"##,
        tx.hash().hex(),
        hex::encode(tx.serialize()),
        dlist.count().uint(),
        dlist.readable(),
        fee.to_fin_string(),
        acc.readable(),
        feeacc.readable(),
        toaddr.readable(),
        time_set
    );
    format!("{{{}}}", ok)
}

/// HIP-25: stake owned HACD (action kind 34).
#[wasm_bindgen]
pub fn hacd_stake(
    chain_id: u64,
    from_pass: String,
    diamond_name_list: String,
    fee: String,
    timestamp: i64,
) -> String {
    build_signed_stake_tx(chain_id, from_pass, diamond_name_list, fee, timestamp, true)
}

/// HIP-25: unstake HACD after min stake period (action kind 35).
#[wasm_bindgen]
pub fn hacd_unstake(
    chain_id: u64,
    from_pass: String,
    diamond_name_list: String,
    fee: String,
    timestamp: i64,
) -> String {
    build_signed_stake_tx(chain_id, from_pass, diamond_name_list, fee, timestamp, false)
}