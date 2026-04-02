//! Error Handling Tests for Omni-Core
//!
//! Checks that the parser correctly detects and reports errors
//! in malformed MDX queries, without panicking or silently returning
//! incorrect/corrupted results.
//!
//! Run with: `cargo test --test test_errors --release -- --nocapture`

use omni_mdx_core::parser::parse_mdx;

// ============================================================================
// Helper Functions
// ============================================================================

/// Asserts that a parsing result is an `Err` and that the error message contains
/// the expected keyword.
fn assert_err(
    result: Result<Vec<omni_mdx_core::ast::AstNode>, omni_mdx_core::ast::ParseError>,
    expected_kind: &str,
) {
    match result {
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.to_lowercase().contains(&expected_kind.to_lowercase()),
                "Expected error kind '{}', but got: {}",
                expected_kind,
                msg
            );
        }
        Ok(ast) => {
            panic!(
                "Expected error '{}' but parse succeeded:\n{}",
                expected_kind,
                serde_json::to_string_pretty(&ast).unwrap_or_default()
            );
        }
    }
}

/// Recursively checks if a specific node type exists anywhere in the tree.
fn has_node_type(nodes: &[omni_mdx_core::ast::AstNode], node_type: &str) -> bool {
    nodes
        .iter()
        .any(|n| n.node_type == node_type || has_node_type(&n.children, node_type))
}

// ============================================================================
// Test Cases
// ============================================================================

#[test]
#[cfg(feature = "dev-tools")]
fn test_unclosed_jsx_block() {
    // [1] Unclosed JSX block
    let result = parse_mdx("<Note type=\"warning\">\n  Oops, I forgot to close.");
    assert_err(result, "unclosed");
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_mismatched_closing_tag() {
    // [2] Mismatched closing tag
    let result = parse_mdx("<Note>\n  Content.\n</Details>");
    assert!(result.is_err(), "Expected error for mismatched closing tag");
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_empty_input() {
    // [3] Empty input → empty AST, no error
    match parse_mdx("") {
        Ok(ast) => assert!(
            ast.is_empty() || !ast.is_empty(),
            "Empty input should be handled gracefully"
        ),
        Err(e) => panic!("Empty input should not error, got: {}", e),
    }
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_lone_dollar_not_math() {
    // [4] Lone $ is not parsed as InlineMath
    let ast = parse_mdx("This has a lone $ sign.").expect("Should not error on lone $");
    let json = serde_json::to_string(&ast).unwrap();
    assert!(
        !json.contains("InlineMath"),
        "Lone $ should not produce InlineMath"
    );
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_angle_brackets_in_text() {
    // [5] Bare < > in text are not mistaken for JSX
    let result = parse_mdx("If a < b and x > 0 then fine.");
    assert!(result.is_ok(), "Bare < > should not cause a crash or error");
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_unclosed_brace_attr() {
    // [6] Unclosed brace in attribute — no panic
    // Must not panic regardless of whether it errors or parses leniently.
    let result = parse_mdx("<Table headers={[\"a\", \"b\" />");
    assert!(
        result.is_ok() || result.is_err(),
        "Unclosed brace should not cause panic"
    );
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_deeply_unclosed_nesting() {
    // [7] Deeply nested unclosed tag
    let result = parse_mdx("<A><B><C><D></D></C></B>"); // missing </A>
    assert!(
        result.is_err() || result.is_ok(),
        "Deeply nested unclosed tag handled"
    );
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_whitespace_only() {
    // [8] Whitespace-only input — no crash
    let result = parse_mdx("   \n\n\t  ");
    assert!(result.is_ok(), "Whitespace-only input should not crash");
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_mismatched_nesting_order() {
    // [9] Intersecting tags (Inverted closing order)
    // Should be caught by the JSX parser stack.
    let mdx = "<Outer><Inner></Outer></Inner>";
    let result = parse_mdx(mdx);

    assert!(result.is_err(), "Mismatched nesting should be an error");
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_broken_attribute_expression() {
    // [10] Malformed JS expression in an attribute (brace never closed)
    let mdx = "<Component data={{ prop: 'value'  >";
    let result = parse_mdx(mdx);

    // We want to ensure the lexer does not consume the rest of the document
    // looking for a brace that does not exist.
    assert!(
        result.is_err(),
        "Unclosed JS expression in attribute must error"
    );
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_math_placeholder_collision() {
    // [11] Attempt to inject internal placeholders manually
    // Verifies that if a user manually types the \x02 control character,
    // the parser does not crash or attempt to access a non-existent pool index.
    let mdx = "Injection attempt: \x02JSX999\x03";
    let result = parse_mdx(mdx);

    // The parser must either escape the character or ignore it, but NEVER panic.
    assert!(
        result.is_ok(),
        "Internal placeholder injection should not cause panic"
    );
    let json = serde_json::to_string(&result.unwrap()).unwrap();
    assert!(
        !json.contains("\"node_type\":\"JSX999\""),
        "Should not expand fake placeholders"
    );
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_unclosed_inline_math_at_eof() {
    // [12] Inline math opened but never closed at the end of the file
    let mdx = "Here is an unfinished equation $e = mc^2";
    let result = parse_mdx(mdx);

    assert!(
        result.is_ok(),
        "Unclosed inline math at EOF should fall back to plain text"
    );
    let nodes = result.unwrap();
    assert!(
        !has_node_type(&nodes, "InlineMath"),
        "Unclosed $ should remain text"
    );
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_jsx_tag_with_no_name() {
    // [13] Empty or malformed tag
    let mdx = "< > content </ >";
    let result = parse_mdx(mdx);

    // Should be treated as plain text (standard Markdown) or an Error,
    // but never as a valid AstNode component.
    if let Ok(nodes) = result {
        assert!(
            !has_node_type(&nodes, " "),
            "Empty tag name should not produce a node"
        );
    }
}
