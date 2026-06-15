#[cfg(not(target_arch = "wasm32"))]
#[link(name = "x16rs", kind = "static")]
extern "C" {
    fn c_x16rs_hash(a: i32, b: *const u8, c: *const u8) -> ();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn x16rs_hash(loopnum: i32, indata: &[u8; 32]) -> [u8; 32] {
    let mut outdata = [0u8; 32];
    unsafe {
        let input: *const u8 = indata.as_ptr();
        let output: *mut u8 = outdata.as_mut_ptr();
        c_x16rs_hash(loopnum, input, output);
    }
    outdata
}

/// WASM SDK stub — stake/unstake signing uses sha3/sha2 only, not x16rs PoW hash.
#[cfg(target_arch = "wasm32")]
pub fn x16rs_hash(_loopnum: i32, indata: &[u8; 32]) -> [u8; 32] {
    *indata
}
