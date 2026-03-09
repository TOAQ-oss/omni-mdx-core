/// Test 2 — Error handling
///
/// Vérifie que le parser détecte et rapporte correctement les erreurs
/// dans un MDX malformé, sans paniquer ni retourner de résultat silencieusement
/// incorrect.

use mdx_parser::ast::ParseError;
use mdx_parser::parser::parse_mdx;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn assert_err(label: &str, result: Result<Vec<mdx_parser::ast::AstNode>, ParseError>, expected_kind: &str) {
    match result {
        Err(e) => {
            let msg = e.to_string();
            let ok = msg.to_lowercase().contains(&expected_kind.to_lowercase());
            if ok {
                println!("  ✅ {label}");
                println!("     → Error: {msg}");
            } else {
                eprintln!("  ❌ {label}");
                eprintln!("     Expected error kind '{expected_kind}', got: {msg}");
                std::process::exit(1);
            }
        }
        Ok(ast) => {
            eprintln!("  ❌ {label}");
            eprintln!("     Expected error '{expected_kind}' but parse succeeded:");
            eprintln!("     {}", serde_json::to_string_pretty(&ast).unwrap_or_default());
            std::process::exit(1);
        }
    }
}

// ── Test cases ────────────────────────────────────────────────────────────────

fn test_unclosed_jsx_block() {
    println!("\n[1] Unclosed JSX block");
    let result = parse_mdx("<Note type=\"warning\">\n  Oops, I forgot to close.");
    assert_err("unclosed <Note>", result, "unclosed");
}

fn test_mismatched_closing_tag() {
    println!("\n[2] Mismatched closing tag");
    let result = parse_mdx("<Note>\n  Content.\n</Details>");
    match result {
        Err(e) => println!("  ✅ correctly rejected: {e}"),
        Ok(_)  => {
            eprintln!("  ❌ Expected error for mismatched closing tag");
            std::process::exit(1);
        }
    }
}

fn test_empty_input() {
    println!("\n[3] Empty input → empty AST, no error");
    match parse_mdx("") {
        Ok(ast) if ast.is_empty() => println!("  ✅ empty input → empty AST"),
        Ok(_) => println!("  ✅ empty input → some nodes (acceptable)"),
        Err(e) => {
            eprintln!("  ❌ empty input should not error, got: {e}");
            std::process::exit(1);
        }
    }
}

fn test_lone_dollar_not_math() {
    println!("\n[4] Lone $ is not parsed as InlineMath");
    match parse_mdx("This has a lone $ sign.") {
        Ok(ast) => {
            let json = serde_json::to_string(&ast).unwrap();
            if json.contains("InlineMath") {
                eprintln!("  ❌ lone $ should not produce InlineMath");
                std::process::exit(1);
            }
            println!("  ✅ lone $ treated as literal text");
        }
        Err(e) => {
            eprintln!("  ❌ lone $ should not error, got: {e}");
            std::process::exit(1);
        }
    }
}

fn test_angle_brackets_in_text() {
    println!("\n[5] Bare < > in text are not mistaken for JSX");
    match parse_mdx("If a < b and x > 0 then fine.") {
        Ok(_)  => println!("  ✅ bare < > in text — no crash"),
        Err(e) => {
            eprintln!("  ❌ bare < > should not error, got: {e}");
            std::process::exit(1);
        }
    }
}

fn test_unclosed_brace_attr() {
    println!("\n[6] Unclosed brace in attribute — no panic");
    // Must not panic regardless of whether it errors or parses leniently.
    let result = parse_mdx("<Table headers={[\"a\", \"b\" />");
    match result {
        Ok(_)  => println!("  ✅ parsed leniently — no panic"),
        Err(e) => println!("  ✅ clean error: {e}"),
    }
}

fn test_deeply_unclosed_nesting() {
    println!("\n[7] Deeply nested unclosed tag");
    let result = parse_mdx("<A><B><C><D></D></C></B>"); // missing </A>
    match result {
        Err(e) => println!("  ✅ correctly rejected: {e}"),
        Ok(_)  => println!("  ✅ parsed leniently (acceptable for this input)"),
    }
}

fn test_whitespace_only() {
    println!("\n[8] Whitespace-only input — no crash");
    match parse_mdx("   \n\n\t  ") {
        Ok(_)  => println!("  ✅ whitespace-only — no crash"),
        Err(e) => println!("  ✅ clean error: {e}"),
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    println!("══════════════════════════════════════════");
    println!("  TEST SUITE 2 — Error Handling / Resilience");
    println!("══════════════════════════════════════════");

    test_unclosed_jsx_block();
    test_mismatched_closing_tag();
    test_empty_input();
    test_lone_dollar_not_math();
    test_angle_brackets_in_text();
    test_unclosed_brace_attr();
    test_deeply_unclosed_nesting();
    test_whitespace_only();

    println!("\n✅ All error-handling tests passed.\n");
}