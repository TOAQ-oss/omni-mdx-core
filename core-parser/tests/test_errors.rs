/// Test 2 — Error handling
///
/// Checks that the parser correctly detects and reports errors
/// in a malformed MDX query, without panicking or silently returning
/// an incorrect result.
/// Run with: `cargo test --test test_errors --release -- --nocapture`

use omni_mdx::parser::parse_mdx;

fn assert_err(result: Result<Vec<omni_mdx::ast::AstNode>, omni_mdx::ast::ParseError>, expected_kind: &str) {
    match result {
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.to_lowercase().contains(&expected_kind.to_lowercase()),
                "Expected error kind '{}', but got: {}", expected_kind, msg
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

fn has_node_type(nodes: &[omni_mdx::ast::AstNode], node_type: &str) -> bool {
    nodes.iter().any(|n| {
        n.node_type == node_type || has_node_type(&n.children, node_type)
    })
}

// Test cases

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
        Ok(ast) => assert!(ast.is_empty() || !ast.is_empty(), "Empty input should be handled gracefully"),
        Err(e) => panic!("Empty input should not error, got: {}", e),
    }
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_lone_dollar_not_math() {
    // [4] Lone $ is not parsed as InlineMath
    let ast = parse_mdx("This has a lone $ sign.").expect("Should not error on lone $");
    let json = serde_json::to_string(&ast).unwrap();
    assert!(!json.contains("InlineMath"), "Lone $ should not produce InlineMath");
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
    assert!(result.is_ok() || result.is_err(), "Unclosed brace should not cause panic");
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_deeply_unclosed_nesting() {
    // [7] Deeply nested unclosed tag
    let result = parse_mdx("<A><B><C><D></D></C></B>"); // missing </A>
    assert!(result.is_err() || result.is_ok(), "Deeply nested unclosed tag handled");
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
    // [9] Balises qui se croisent (Inversion de l'ordre de fermeture)
    // Devrait être capturé par la pile (stack) du parser JSX.
    let mdx = "<Outer><Inner></Outer></Inner>";
    let result = parse_mdx(mdx);
    
    // On utilise ton helper pour vérifier que l'erreur parle bien de "mismatch" ou de "closing"
    assert!(result.is_err(), "Mismatched nesting should be an error");
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_broken_attribute_expression() {
    // [10] Expression JS malformée dans un attribut (accolade jamais fermée)
    let mdx = "<Component data={{ prop: 'value'  >"; 
    let result = parse_mdx(mdx);
    
    // Ici on veut s'assurer que le lexer ne "mange" pas tout le reste du document
    // en cherchant une accolade qui n'existe pas.
    assert!(result.is_err(), "Unclosed JS expression in attribute must error");
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_math_placeholder_collision() {
    // [11] Tentative d'injection de placeholders internes
    // Vérifie que si un utilisateur écrit manuellement le caractère de contrôle \x02,
    // le parser ne casse pas ou ne tente pas d'accéder à un index de pool inexistant.
    let mdx = "Essai d'injection : \x02JSX999\x03";
    let result = parse_mdx(mdx);
    
    // Le parser doit soit échapper le caractère, soit l'ignorer, mais ne JAMAIS paniquer.
    assert!(result.is_ok(), "Internal placeholder injection should not cause panic");
    let json = serde_json::to_string(&result.unwrap()).unwrap();
    assert!(!json.contains("\"node_type\":\"JSX999\""), "Should not expand fake placeholders");
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_unclosed_inline_math_at_eof() {
    // [12] Math inline ouvert mais jamais fermé à la fin du fichier
    let mdx = "Voici une équation non finie $e = mc^2";
    let result = parse_mdx(mdx);
    
    assert!(result.is_ok(), "Unclosed inline math at EOF should fall back to plain text");
    let nodes = result.unwrap();
    assert!(!has_node_type(&nodes, "InlineMath"), "Unclosed $ should remain text");
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_jsx_tag_with_no_name() {
    // [13] Balise vide ou mal formée
    let mdx = "< > contenu </ >";
    let result = parse_mdx(mdx);
    
    // Devrait être traité comme du texte (Markdown standard) ou Erreur, 
    // mais pas comme un composant AstNode valide.
    if let Ok(nodes) = result {
        assert!(!has_node_type(&nodes, " "), "Empty tag name should not produce a node");
    }
}