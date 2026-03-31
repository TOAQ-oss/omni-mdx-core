/// test_security.rs — Security, Denial of Service (DoS) and malicious payload tests.
///
/// Chosen because the parser must survive hostile environments and fuzzing:
///   - Algorithmic complexity attacks (O(n²) backtracking via footnotes)
///   - WASM Stack Overflow attacks (Ambiguous lists and deep blockquotes)
///   - Memory exhaustion (Strict global payload size limits)
///   - Zero-day structural exploits (Universal symbol entropy shield)
///   - False positives (Ensuring legitimate large documents are never blocked)
///
/// Run with: `cargo test --test test_security --release -- --nocapture`
use omni_mdx_core::parser::parse_mdx;

fn test_payload_rejection(payload: &str) {
    let result = parse_mdx(payload);
    
    assert!(
        result.is_err(),
        "The shield failed to reject a malicious payload!"
    );
}

#[test]
fn test_shield_1_legitimate_document_passes() {
    let normal_text = "Ceci est un document normal. ".repeat(10_000); 
    let result = parse_mdx(&normal_text);
    assert!(result.is_ok(), "The shield blocked a legitimate document!");
}

#[test]
fn test_shield_2_global_size_limit() {
    let huge_payload = "A".repeat(2_100_000);
    test_payload_rejection(&huge_payload);
}

#[test]
fn test_shield_3_footnote_backtracking_bomb() {
    let footnote_bomb = "[^".repeat(105);
    test_payload_rejection(&footnote_bomb);
}

#[test]
fn test_shield_4_list_ambiguity_stack_overflow() {
    let mut list_bomb = String::new();
    for _ in 0..55 {
        list_bomb.push_str("-  - \n");
    }
    test_payload_rejection(&list_bomb);
}

#[test]
fn test_shield_5_blockquote_nesting_bomb() {
    let quote_bomb = ">>>>>>>>> Attaque de profondeur";
    test_payload_rejection(&quote_bomb);
}

#[test]
fn test_shield_6_universal_entropy_bomb() {
    let entropy_bomb = "*_~`|".repeat(55);
    test_payload_rejection(&entropy_bomb);
}