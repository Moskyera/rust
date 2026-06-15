use std::ffi::{c_char, c_void, CString};
use std::ptr;

use leveldb_sys::*;
use libc::size_t;

include!("error.rs");
include!("bytes.rs");
include!("batch.rs");
include!("db.rs");