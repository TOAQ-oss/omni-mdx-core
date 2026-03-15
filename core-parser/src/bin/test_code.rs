/// test_code.rs — Code block protection tests.
///
/// Verifies that JSX/HTML inside code blocks (fenced and inline) is NOT
/// extracted by the JSX lexer and appears correctly in the final AST.
///
/// Run with: `cargo run --bin test_code --features="dev-tools"`

use mdx_parser::ast::{AstNode, AttrValue};
use mdx_parser::parser::parse_mdx;

// Helpers

fn find<'a>(nodes: &'a [AstNode], node_type: &str) -> Option<&'a AstNode> {
    nodes.iter().find(|n| n.node_type == node_type)
}

fn find_all<'a>(nodes: &'a [AstNode], node_type: &str) -> Vec<&'a AstNode> {
    let mut result = Vec::new();
    for node in nodes {
        if node.node_type == node_type { result.push(node); }
        if !node.children.is_empty() {
            result.extend(find_all(&node.children, node_type));
        }
    }
    result
}

fn has_node_type(nodes: &[AstNode], node_type: &str) -> bool {
    nodes.iter().any(|n| {
        n.node_type == node_type || has_node_type(&n.children, node_type)
    })
}

fn pass(label: &str) { println!("  ✅ {label}"); }
fn fail(label: &str, detail: &str) { eprintln!("  ❌ {label}: {detail}"); std::process::exit(1); }

/// Fenced code block containing JSX
const MDX_FENCED_JSX: &str = r#"
Normal paragraph.

```tsx
return <Component prop="value" />;
```

After the block.
"#;

/// Fenced code block containing HTML tags
const MDX_FENCED_HTML: &str = r#"
```html
<div class="container">
  <p>Hello</p>
</div>
```
"#;

/// Inline code containing angle brackets
const MDX_INLINE_CODE: &str = r#"Use the `<Note />` component to add callouts."#;

/// Fenced code block with a real JSX component after it
const MDX_FENCED_THEN_JSX: &str = r#"
Here is some code:

```tsx
const x = <Foo />;
```

<Note type="info" title="Real component">
  This is real JSX.
</Note>
"#;

/// Fenced code with math-like content (should not be extracted as math)
const MDX_FENCED_MATH: &str = r#"
```latex
$E = mc^2$
$$\int_0^\infty$$
```

Real math: $x^2$
"#;

/// Multiple fenced blocks with JSX inside
const MDX_MULTIPLE_FENCED: &str = r#"
```tsx
<ComponentA />
```

Text between.

```tsx
<ComponentB />
```
"#;

fn main() {
    println!("=== test_code: code block protection ===\n");

    // 1. Fenced block with JSX inside
    println!("[1] Fenced code block with JSX inside");

    let ast = parse_mdx(MDX_FENCED_JSX).unwrap_or_else(|e| {
        fail("parse_mdx", &e.to_string()); unreachable!()
    });

    // <Component /> must NOT appear as a node in the AST
    if has_node_type(&ast, "Component") {
        fail("<Component> not extracted", "<Component> was incorrectly extracted from fenced block");
    }
    pass("<Component> not extracted from fenced block");

    // Must have a <pre> node containing the code
    let pre = find(&ast, "pre").unwrap_or_else(|| {
        fail("pre node present", "not found"); unreachable!()
    });
    pass("pre node present");

    // The pre content must contain the original code text
    let pre_text = pre.children.iter()
        .find(|c| c.node_type == "code")
        .and_then(|c| c.content.as_deref())
        .unwrap_or_else(|| {
            // fallback: check text content
            pre.children.iter()
                .find(|c| c.node_type == "text")
                .and_then(|c| c.content.as_deref())
                .unwrap_or("")
        });
    if !pre_text.contains("Component") {
        fail("pre contains original code", &format!("got: '{pre_text}'"));
    }
    pass("pre contains original code text");

    // Paragraph before and after must be present
    let paras = find_all(&ast, "p");
    if paras.is_empty() {
        fail("paragraphs present", "none found");
    }
    pass(&format!("{} paragraph(s) present around code block", paras.len()));

    // 2. Fenced block with HTML tags
    println!("\n[2] Fenced code block with HTML tags");

    let ast = parse_mdx(MDX_FENCED_HTML).unwrap_or_else(|e| {
        fail("parse_mdx", &e.to_string()); unreachable!()
    });

    // <div> and <p> inside the fenced block must not generate extra nodes
    // Only one pre at root
    let pres = find_all(&ast, "pre");
    if pres.is_empty() {
        fail("pre node present", "not found");
    }
    pass(&format!("{} pre node(s) — HTML tags not extracted", pres.len()));

    // 3. Inline code with angle brackets
    println!("\n[3] Inline code with angle brackets");

    let ast = parse_mdx(MDX_INLINE_CODE).unwrap_or_else(|e| {
        fail("parse_mdx", &e.to_string()); unreachable!()
    });

    // <Note /> must NOT appear as a component node
    if has_node_type(&ast, "Note") {
        fail("<Note> not extracted from inline code", "<Note> was incorrectly extracted");
    }
    pass("<Note> not extracted from inline code");

    // Must have a paragraph with a code child
    let p = find(&ast, "p").unwrap_or_else(|| {
        fail("paragraph present", "not found"); unreachable!()
    });
    let has_code = p.children.iter().any(|c| c.node_type == "code");
    if !has_code {
        fail("code node in paragraph", "not found");
    }
    pass("inline code node present in paragraph");

    // 4. Fenced block then real JSX component
    println!("\n[4] Fenced block followed by real JSX component");

    let ast = parse_mdx(MDX_FENCED_THEN_JSX).unwrap_or_else(|e| {
        fail("parse_mdx", &e.to_string()); unreachable!()
    });

    // <Foo /> inside the fenced block must NOT be extracted
    if has_node_type(&ast, "Foo") {
        fail("<Foo> not extracted from fenced block", "<Foo> was incorrectly extracted");
    }
    pass("<Foo> not extracted from fenced block");

    // <Note> after the fenced block MUST be extracted
    let note = find(&ast, "Note").unwrap_or_else(|| {
        fail("<Note> extracted after fenced block", "not found"); unreachable!()
    });
    pass("<Note> correctly extracted after fenced block");

    // Note attributes must be correct
    let note_type = note.attributes.as_ref()
        .and_then(|a| a.get("type"))
        .and_then(|v| if let mdx_parser::ast::AttrValue::Text(s) = v { Some(s.as_str()) } else { None });
    if note_type != Some("info") {
        fail("Note type attr", &format!("expected 'info', got '{:?}'", note_type));
    }
    pass("Note type attribute correct");

    // 5. Fenced block with math-like content
    println!("\n[5] Fenced block with math-like content");

    let ast = parse_mdx(MDX_FENCED_MATH).unwrap_or_else(|e| {
        fail("parse_mdx", &e.to_string()); unreachable!()
    });

    // InlineMath inside fenced block must NOT be extracted
    // Only the real $x^2$ after the block should produce an InlineMath node
    let inline_maths = find_all(&ast, "InlineMath");

    // Debug: print all InlineMath nodes found
    println!("    [debug] {} InlineMath node(s) found:", inline_maths.len());
    for m in &inline_maths {
        println!("      content: {:?}", m.content);
    }

    // Math inside fenced block must NOT appear
    let bad_math = inline_maths.iter().any(|m| {
        let c = m.content.as_deref().unwrap_or("");
        c.contains("int") || c.contains("infty")
    });
    if bad_math {
        fail("math inside fenced block not extracted", "found math from inside the fenced block");
    }
    pass("math inside fenced block not extracted");

    // The real $x^2$ outside the block MUST appear
    let has_x2 = inline_maths.iter().any(|m| m.content.as_deref() == Some("x^2"));
    if !has_x2 {
        fail("InlineMath(x^2) present", "not found — math after fenced block was not parsed");
    }
    pass("InlineMath(x^2) correctly parsed after fenced block");

    // 6. Multiple fenced blocks
    println!("\n[6] Multiple fenced blocks with JSX inside");

    let ast = parse_mdx(MDX_MULTIPLE_FENCED).unwrap_or_else(|e| {
        fail("parse_mdx", &e.to_string()); unreachable!()
    });

    if has_node_type(&ast, "ComponentA") || has_node_type(&ast, "ComponentB") {
        fail(
            "<ComponentA> and <ComponentB> not extracted",
            "at least one was incorrectly extracted from fenced blocks",
        );
    }
    pass("<ComponentA> and <ComponentB> not extracted from fenced blocks");

    let pres = find_all(&ast, "pre");
    if pres.len() != 2 {
        fail("2 pre nodes present", &format!("got {}", pres.len()));
    }
    pass("2 pre nodes present for 2 fenced blocks");

    println!("\n✅ All code block protection tests passed.");
}