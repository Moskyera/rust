
#[wasm_bindgen]
pub fn create_account_by(s: String) -> String {
    let acc = or_return!{ "create account", Account::create_by(&s) };
    // ok
    let accstr = acc.readable();
    let accpub = hex::encode(acc.public_key().serialize_compressed());
    let ok = format!(r##""public_key":"{}","address":"{}""##, accpub, accstr);
    format!("{{{}}}", ok)
}
