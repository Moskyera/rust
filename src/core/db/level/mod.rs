#[cfg(target_arch = "wasm32")]
include!("wasm_stub.rs");

#[cfg(not(target_arch = "wasm32"))]
include!("native_level.rs");