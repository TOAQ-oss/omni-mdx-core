/// test_code.rs — Code block protection tests.
///
/// Verifies that JSX/HTML inside code blocks (fenced and inline) is NOT
/// extracted by the JSX lexer and appears correctly in the final AST.
///
/// Run with: `cargo run --bin test_code --release -- --nocapture`
use omni_mdx_core::ast::{AstNode, AttrValue};
use omni_mdx_core::parser::parse_mdx;

// Helpers

fn find<'a>(nodes: &'a [AstNode], node_type: &str) -> Option<&'a AstNode<'a>> {
    nodes.iter().find(|n| n.node_type == node_type)
}

fn find_all<'a>(nodes: &'a [AstNode], node_type: &str) -> Vec<&'a AstNode<'a>> {
    let mut result = Vec::new();
    for node in nodes {
        if node.node_type == node_type {
            result.push(node);
        }
        if !node.children.is_empty() {
            result.extend(find_all(&node.children, node_type));
        }
    }
    result
}

fn has_node_type(nodes: &[AstNode], node_type: &str) -> bool {
    nodes
        .iter()
        .any(|n| n.node_type == node_type || has_node_type(&n.children, node_type))
}

fn check_no_hidden_bytes(nodes: &[AstNode]) {
    for node in nodes {
        if let Some(ref content) = node.content {
            assert!(
                !content.contains('\x01'),
                "Byte \\x01 trouvé dans le nœud {}: {:?}",
                node.node_type,
                content
            );
            assert!(
                !content.contains('\x04'),
                "Byte \\x04 trouvé dans le nœud {}: {:?}",
                node.node_type,
                content
            );
            assert!(
                !content.contains('\x05'),
                "Byte \\x05 trouvé dans le nœud {}: {:?}",
                node.node_type,
                content
            );
        }
        check_no_hidden_bytes(&node.children);
    }
}

#[test]
fn test_fenced_jsx_is_protected() {
    let mdx = r#"
```tsx
return <Component prop="value" />;
```"#;

    let ast = parse_mdx(mdx).expect("parse_mdx failed");
    // <Component /> must NOT appear as a node in the AST
    assert!(
        !has_node_type(&ast, "Component"),
        "<Component> was incorrectly extracted from fenced block"
    );
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_fenced_html_is_protected() {
    let mdx = r#"
```html
<div class="container">
    <p>Hello</p>
</div>
```
    "#;
    let ast = parse_mdx(mdx).expect("parse_mdx failed");

    if find_all(&ast, "pre").is_empty() {
        println!("DEBUG AST: {}", serde_json::to_string_pretty(&ast).unwrap());
    }
    // Should only have 1 pre node and NO div or p nodes
    let pres = find_all(&ast, "pre");
    assert_eq!(pres.len(), 1);
    assert!(
        !has_node_type(&ast, "div"),
        "div incorrectly extracted from HTML code block"
    );
}

#[test]
fn test_inline_code_is_protected() {
    let mdx = r#"Use the `<Note />` component."#;
    let ast = parse_mdx(mdx).expect("parse_mdx failed");

    assert!(
        !has_node_type(&ast, "Note"),
        "Note extracted from inline code"
    );

    let p = find(&ast, "p").expect("paragraph not found");
    assert!(
        p.children.iter().any(|c| c.node_type == "code"),
        "Inline code node missing"
    );
}

#[test]
fn test_fenced_block_followed_by_real_jsx() {
    let mdx = r#"
```typescript
const x = <Foo />;
```
<Note type="info">Real</Note>
"#;
    let ast = parse_mdx(mdx).expect("parse_mdx failed");
    // Foo is inside code (protected), Note is outside (extracted)
    assert!(!has_node_type(&ast, "Foo"), "Foo extracted from code block");

    let note = find(&ast, "Note").expect("Real Note component not found");
    let note_type = note
        .attributes
        .as_ref()
        .and_then(|a| a.get("type"))
        .and_then(|v| {
            if let AttrValue::Text(s) = v {
                Some(s.as_ref())
            } else {
                None
            }
        });

    assert_eq!(note_type, Some("info"));
}

#[test]
fn test_math_inside_code_is_protected() {
    let mdx = r#"
```
$E = mc^2$
$$\int_0^\infty$$
```
Real math: $x^2$
"#;
    let ast = parse_mdx(mdx).expect("parse_mdx failed");

    let inline_maths = find_all(&ast, "InlineMath");

    // Only the real $x^2$ should be extracted
    let has_complex_math = inline_maths.iter().any(|m| {
        let c = m.content.as_deref().unwrap_or("");
        c.contains("mc^2") || c.contains("int")
    });

    assert!(
        !has_complex_math,
        "Math inside code block was incorrectly extracted"
    );
    assert!(
        inline_maths
            .iter()
            .any(|m| m.content.as_deref() == Some("x^2")),
        "Real math after code block missing"
    );
}

#[test]
fn test_multiple_sequential_fenced_blocks() {
    let mdx = r#"
```typescript
<ComponentA />
```

Text

```typescript
<ComponentB />
```
"#;
    let ast = parse_mdx(mdx).expect("parse_mdx failed");

    assert!(!has_node_type(&ast, "ComponentA"));
    assert!(!has_node_type(&ast, "ComponentB"));

    let pres = find_all(&ast, "pre");
    assert_eq!(pres.len(), 2, "Should have 2 distinct pre nodes");
}

#[test]
fn test_restoration_integrity() {
    let mdx = r#"
```tsx
const x = <MDXServerRenderer ast={ast} />;
```

`<Inline />` and $math$
"#;
    let ast = parse_mdx(mdx).expect("Le parsing a échoué");

    check_no_hidden_bytes(&ast);
}

#[test]
fn test_code_language_fallbacks() {
    let markdown_input = "\
```rust
fn main() {}
```

```
plaintext
```
";
    let ast = parse_mdx(markdown_input).expect("Markdown parsing failed");
    let pres = find_all(&ast, "pre");
    assert_eq!(pres.len(), 2, "Should find exactly 2 'pre' blocks");

    let rust_block = pres[0];
    let rust_attrs = rust_block
        .attributes
        .as_ref()
        .expect("First block should have attributes");
    match rust_attrs.get("className") {
        Some(AttrValue::Text(t)) => {
            assert_eq!(t, "language-rust", "Should extract 'rust' language")
        }
        _ => panic!("Expected className='language-rust'"),
    }

    let plain_block = pres[1];
    if let Some(attrs) = &plain_block.attributes {
        assert!(
            !attrs.contains_key("className"),
            "Second block should NOT have a className attribute (Fallback to plaintext)"
        );
    }
}
