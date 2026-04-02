//! Security and Denial of Service (DoS) Tests
//!
//! This module ensures the parser can survive hostile environments and targeted fuzzing.
//! Standard Markdown parsers are notorious for catastrophic backtracking and memory
//! exhaustion vulnerabilities. These tests guarantee our shields hold against:
//!   - Algorithmic complexity attacks (O(n²) backtracking via unclosed footnotes)
//!   - WASM Stack Overflow attacks (Ambiguous list parsing and deep blockquotes)
//!   - Memory exhaustion (Strict global payload size limits)
//!   - Zero-day structural exploits (Universal symbol entropy shields)
//!   - False positives (Ensuring legitimate large documents are never blocked)
//!
//! Run with: `cargo test --test test_security --release -- --nocapture`

use omni_mdx_core::parser::parse_mdx;

// ============================================================================
// Helper Functions
// ============================================================================

/// Asserts that a specifically crafted malicious payload is outright rejected
/// by the parser's pre-flight security shields, returning an `Err` before
/// any heavy allocation or parsing occurs.
fn test_payload_rejection(payload: &str) {
    let result = parse_mdx(payload);

    assert!(
        result.is_err(),
        "CRITICAL: The security shield failed to reject a malicious payload!"
    );
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_shield_1_legitimate_document_passes() {
    // A document can be large, as long as its symbol density is normal.
    let normal_text = "This is a normal document. ".repeat(10_000);
    let result = parse_mdx(&normal_text);
    assert!(
        result.is_ok(),
        "False Positive: The shield blocked a legitimate large document!"
    );
}

#[test]
fn test_shield_2_global_size_limit() {
    // Exactly 2.1 MB of 'A's (exceeds the 2,000,000 bytes limit).
    let huge_payload = "A".repeat(2_100_000);
    test_payload_rejection(&huge_payload);
}

#[test]
fn test_shield_3_footnote_backtracking_bomb() {
    // 105 unclosed footnotes trigger O(N^2) catastrophic backtracking in pulldown-cmark.
    let footnote_bomb = "[^".repeat(105);
    test_payload_rejection(&footnote_bomb);
}

#[test]
fn test_shield_4_list_ambiguity_stack_overflow() {
    // Malformed/ambiguous lists can cause infinite recursion or extreme stack depth.
    let mut list_bomb = String::new();
    for _ in 0..55 {
        list_bomb.push_str("-  - \n");
    }
    test_payload_rejection(&list_bomb);
}

#[test]
fn test_shield_5_blockquote_nesting_bomb() {
    // Deep blockquote nesting designed to trigger a stack overflow in WebAssembly limits.
    let quote_bomb = ">>>>>>>>> Depth attack";
    test_payload_rejection(&quote_bomb);
}

#[test]
fn test_shield_6_universal_entropy_bomb() {
    // An abnormal density of purely structural symbols designed to freeze the thread.
    let entropy_bomb = "*_~`|".repeat(55);
    test_payload_rejection(&entropy_bomb);
}
