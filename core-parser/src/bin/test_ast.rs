/// test_ast.rs — Structural correctness tests.
///
/// Parses a realistic MDX document and asserts that the output AST contains
/// exactly the nodes, attributes, and children we expect.
/// Run with: `cargo run --bin test_ast --features="dev-tools"`

use mdx_parser::ast::{AstNode, AttrValue};
use mdx_parser::parser::parse_mdx;

// Helpers

fn find<'a>(nodes: &'a [AstNode], node_type: &str) -> Option<&'a AstNode> {
    nodes.iter().find(|n| n.node_type == node_type)
}

fn find_all<'a>(nodes: &'a [AstNode], node_type: &str) -> Vec<&'a AstNode> {
    nodes.iter().filter(|n| n.node_type == node_type).collect()
}

fn text_content(node: &AstNode) -> String {
    node.children.iter()
        .filter(|c| c.node_type == "text")
        .filter_map(|c| c.content.as_deref())
        .collect::<Vec<_>>()
        .join("")
}

fn attr_text<'a>(node: &'a AstNode, key: &str) -> Option<&'a str> {
    match node.attributes.as_ref()?.get(key)? {
        AttrValue::Text(s) => Some(s.as_str()),
        _ => None,
    }
}

fn pass(label: &str) { println!("  ✅ {label}"); }
fn fail(label: &str, detail: &str) { eprintln!("  ❌ {label}: {detail}"); std::process::exit(1); }

// Test document

const MDX: &str = "
# Titre principal

Paragraphe avec du texte **gras** et *italique*.

<Note type=\"warning\" title=\"Point de vigilance\">
  Ce mod\u{00e8}le n'est valide que pour $t > 0$.
</Note>

<Details title=\"D\u{00e9}monstration\">
  L'\u{00e9}quation fondamentale :
  $$ E = mc^2 $$
  Voir ci-dessus.
</Details>

<Table
  caption=\"M\u{00e9}triques\"
  headers={[\"Nom\", \"Valeur\"]}
  data={[[\"Latence\", 12], [\"D\u{00e9}bit\", 1024]]}
/>

$$
\\sum_{i=1}^{n} x_i
$$

## Section 2

- Premier \u{00e9}l\u{00e9}ment
- Deuxi\u{00e8}me \u{00e9}l\u{00e9}ment

## Link
[Link text](https://example.com)
";

fn main() {
    println!("=== test_ast: structural correctness ===\n");

    let ast = match parse_mdx(MDX) {
        Ok(a)  => a,
        Err(e) => { eprintln!("❌ parse_mdx failed: {e}"); std::process::exit(1); }
    };

    // 1. Top-level structure
    println!("[1] Top-level structure");

    let h1 = find(&ast, "h1").unwrap_or_else(|| { fail("h1 present", "not found"); unreachable!() });
    let h1_text = text_content(h1);
    if h1_text != "Titre principal" { fail("h1 text", &h1_text); }
    pass("h1 with correct text");

    if find(&ast, "h2").is_none() { fail("h2 present", "not found"); }
    pass("h2 present");

    let paras = find_all(&ast, "p");
    if paras.is_empty() { fail("paragraphs present", "none found"); }
    pass(&format!("{} paragraph(s) at root", paras.len()));

    // 2. Inline formatting
    println!("\n[2] Inline formatting");

    let first_p = paras[0];
    let has_strong = first_p.children.iter().any(|c| c.node_type == "strong");
    let has_em    = first_p.children.iter().any(|c| c.node_type == "em");
    if !has_strong { fail("strong in paragraph", "not found"); }
    if !has_em     { fail("em in paragraph", "not found"); }
    pass("strong and em inside paragraph");

    // 3. <Note> component
    println!("\n[3] <Note> component");

    let note = find(&ast, "Note").unwrap_or_else(|| { fail("Note present", "not found"); unreachable!() });
    if attr_text(note, "type") != Some("warning") { fail("Note type attr", "expected 'warning'"); }
    if attr_text(note, "title") != Some("Point de vigilance") { fail("Note title attr", "wrong value"); }
    pass("Note type and title attributes correct");

    // Note children should include InlineMath for $t > 0$
    let has_inline_math = note.children.iter().any(|c| c.node_type == "InlineMath");
    if !has_inline_math { fail("InlineMath in Note", "not found"); }
    let math = note.children.iter().find(|c| c.node_type == "InlineMath").unwrap();
    if math.content.as_deref() != Some("t > 0") { fail("InlineMath content", math.content.as_deref().unwrap_or("None")); }
    pass("InlineMath($t > 0$) inside Note");

    // InlineMath must NOT be wrapped in a <p> (no block-level hoisting for inline)
    let p_wrapping_math = note.children.iter()
        .filter(|c| c.node_type == "p")
        .any(|p| p.children.iter().any(|c| c.node_type == "InlineMath"));
    if p_wrapping_math { fail("InlineMath not in <p>", "InlineMath is wrapped in <p>"); }
    pass("InlineMath is NOT wrapped in <p>");

    // 4. <Details> with BlockMath
    println!("\n[4] <Details> with BlockMath");

    let details = find(&ast, "Details").unwrap_or_else(|| { fail("Details present", "not found"); unreachable!() });
    let got_title = attr_text(details, "title").unwrap_or("<none>");
    if got_title != "D\u{00e9}monstration" {
        fail("Details title", &format!("expected 'Démonstration', got '{got_title}'"));
    }
    pass("Details title attribute correct");

    let has_block_math = details.children.iter().any(|c| c.node_type == "BlockMath");
    if !has_block_math { fail("BlockMath in Details", "not found"); }
    let bmath = details.children.iter().find(|c| c.node_type == "BlockMath").unwrap();
    if bmath.content.as_deref() != Some("E = mc^2") { fail("BlockMath content", bmath.content.as_deref().unwrap_or("None")); }
    pass("BlockMath($$ E = mc^2 $$) inside Details");

    // BlockMath must NOT be inside a <p>
    let p_wrapping_bmath = details.children.iter()
        .filter(|c| c.node_type == "p")
        .any(|p| p.children.iter().any(|c| c.node_type == "BlockMath"));
    if p_wrapping_bmath { fail("BlockMath not in <p>", "BlockMath is wrapped in <p>"); }
    pass("BlockMath is NOT wrapped in <p>");

    // 5. <Table> self-closing with expression attrs
    println!("\n[5] <Table> self-closing with expression attributes");

    let table = find(&ast, "Table").unwrap_or_else(|| { fail("Table present", "not found"); unreachable!() });
    if !table.self_closing { fail("Table self_closing", "expected true"); }
    pass("Table is self-closing");
    if attr_text(table, "caption") != Some("M\u{00e9}triques") { fail("Table caption", "wrong"); }
    pass("Table caption attribute correct");

    let headers = table.attributes.as_ref().and_then(|a| a.get("headers"));
    match headers {
        Some(AttrValue::Expression(_)) => pass("Table headers is Expression"),
        _ => fail("Table headers", "expected Expression variant"),
    }

    // 6. Root-level BlockMath
    println!("\n[6] Root-level BlockMath");

    let root_bmath = find(&ast, "BlockMath");
    if root_bmath.is_none() { fail("root BlockMath", "not found"); }
    pass("Root-level BlockMath present");

    // 7. Lists
    println!("\n[7] Lists");

    let ul = find(&ast, "ul").unwrap_or_else(|| { fail("ul present", "not found"); unreachable!() });
    let items = find_all(&ul.children, "li");
    if items.len() != 2 { fail("li count", &format!("expected 2, got {}", items.len())); }
    pass("ul with 2 li items");

    // 8. UTF-8 integrity
    println!("\n[8] UTF-8 integrity");

    let json = serde_json::to_string(&ast).unwrap();
    // Corruption signature: é becomes Ã© (C3 → U+00C3, A9 → U+00A9)
    if json.contains("\u{00c3}\u{00a9}") || json.contains("\u{00c3}\u{00a0}") {
        fail("UTF-8 not corrupted", "found Mojibake sequences in JSON output");
    }
    pass("No UTF-8 corruption in JSON output");

    println!("\n[9] Link attributes and content");

    let p_link = find_all(&ast, "p").into_iter()
        .find(|p| p.children.iter().any(|c| c.node_type == "a"))
        .unwrap_or_else(|| { fail("Link container paragraph", "not found"); unreachable!() });

    let link = p_link.children.iter().find(|c| c.node_type == "a")
        .unwrap_or_else(|| { fail("Link (a) node", "not found"); unreachable!() });

    if attr_text(link, "href") != Some("https://example.com") {
        let got = attr_text(link, "href").unwrap_or("<none>");
        fail("Link href", &format!("expected 'https://example.com', got '{got}'"));
    }
    pass("Link href attribute correct");

    let link_text = text_content(link);
    if link_text != "Link text" {
        fail("Link text content", &format!("expected 'Link text', got '{link_text}'"));
    }
    pass("Link text content correct");

    // Done
    println!("\n✅ All tests passed.");
}