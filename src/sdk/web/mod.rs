

// We need the trait in scope to use Utc::timestamp().
use chrono::{TimeZone, Utc};

use wasm_bindgen::prelude::*;

use crate::base::field::*;
use crate::core::field::*;
use crate::interface::field::*;
use crate::interface::protocol::{Transaction, TransactionRead};
use crate::protocol::action;
use crate::protocol::action::*;
use crate::protocol::transaction;

/******** sdk ********/

macro_rules! or_return {
    ($tip:expr, $gain:expr) => (
        match $gain {
            Ok(obj) => obj,
            Err(e) => {
                return format!("[ERROR] {}: {}", $tip, e)
            }
        }
    )
}


include!{"amount.rs"}
include!{"account.rs"}
include!{"sign.rs"}
include!{"transfer.rs"}