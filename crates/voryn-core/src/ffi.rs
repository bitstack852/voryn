//! C FFI exports — callable from Swift/Objective-C via the static library.
//!
//! These functions use C-compatible types and return heap-allocated strings
//! that must be freed by the caller using voryn_free_string().

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::bridge;

/// Returns a greeting string.
#[no_mangle]
pub extern "C" fn voryn_hello() -> *const c_char {
    let s = bridge::hello_from_rust();
    let c_str = CString::new(s).unwrap_or_default();
    c_str.into_raw()
}

/// Generate a new identity. Returns a JSON string:
/// {"public_key_hex": "...", "secret_key_seed_hex": "..."}
#[no_mangle]
pub extern "C" fn voryn_generate_identity() -> *const c_char {
    let identity = bridge::generate_identity();
    let json = format!(
        r#"{{"public_key_hex":"{}","secret_key_seed_hex":"{}"}}"#,
        identity.public_key_hex,
        hex_encode(&identity.secret_key_seed),
    );
    let c_str = CString::new(json).unwrap_or_default();
    c_str.into_raw()
}

/// Compute safety number from two 32-byte public keys.
#[no_mangle]
pub extern "C" fn voryn_compute_safety_number(
    our_pk: *const u8,
    their_pk: *const u8,
) -> *const c_char {
    if our_pk.is_null() || their_pk.is_null() {
        let c_str = CString::new("").unwrap_or_default();
        return c_str.into_raw();
    }

    let our_bytes = unsafe { std::slice::from_raw_parts(our_pk, 32) };
    let their_bytes = unsafe { std::slice::from_raw_parts(their_pk, 32) };

    let sn = bridge::compute_safety_number(our_bytes.to_vec(), their_bytes.to_vec());
    let c_str = CString::new(sn).unwrap_or_default();
    c_str.into_raw()
}

/// Free a string returned by any voryn_ function.
#[no_mangle]
pub extern "C" fn voryn_free_string(s: *const c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s as *mut c_char));
        }
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
