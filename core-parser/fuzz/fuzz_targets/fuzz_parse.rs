// fuzz/fuzz_targets/fuzz_parse.rs
//
// Fuzzing harness for the MDX parser.
// Run: cargo fuzz run fuzz_parse -- -max_len=65536 -timeout=10
//
// The harness must NEVER panic — all errors must be returned as Result::Err.
// Any panic = a fuzzer finding = a bug to fix.
 
#![no_main]
 
use libfuzzer_sys::fuzz_target;
 
fuzz_target!(|data: &[u8]| {
    // Reject invalid UTF-8 — the parser contract requires valid UTF-8 input.
    // We fuzz the parser logic, not the UTF-8 validation (that's tested separately).
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };
 
    // The parser must never panic — only return Ok or Err.
    let _ = omni_mdx_core::parser::parse_mdx(input);
});
 