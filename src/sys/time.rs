use chrono::{DateTime, Local, TimeZone, Utc};

pub const TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub fn curtimes() -> u64 {
    // std::time::SystemTime is unavailable on wasm32-unknown-unknown (browser WASM SDK).
    #[cfg(target_arch = "wasm32")]
    {
        return Utc::now().timestamp() as u64;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u64
    }
}


pub fn ctshow() -> String {
    Local::now().format(TIME_FORMAT).to_string()
}

pub fn timeshow(t: u64) -> String {
    Local.timestamp_opt(t as i64, 0).unwrap().format(TIME_FORMAT).to_string()
}

// &str = "%Y-%m-%d %H:%M:%S";   %Y%m%d
pub fn timefmt(t: u64, fmts: &str) -> String {
    Local.timestamp_opt(t as i64, 0).unwrap().format(fmts).to_string()
}


