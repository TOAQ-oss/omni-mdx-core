// fuzz/fuzz_targets/fuzz_utf8.rs
//
// Tests the parser with arbitrary bytes including invalid UTF-8.
// Verifies that invalid UTF-8 is rejected gracefully, not panicked on.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    match std::str::from_utf8(data) {
        Ok(s) => { let _ = mdx_parser::parser::parse_mdx(s); }
        Err(_) => {
            // Invalid UTF-8: verify ffi layer handles it without panic
            use std::ffi::CString;
            if let Ok(cs) = CString::new(data) {
                unsafe { mdx_parser::ffi::mdx_parse(cs.as_ptr()); }
            }
        }
    }
});