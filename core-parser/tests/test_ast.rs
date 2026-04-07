//! Structural Correctness Tests for Omni-Core AST
//!
//! Parses realistic MDX documents and asserts that the output AST contains
//! exactly the nodes, attributes, and children we expect.
//!
//! Run with: `cargo test --test test_ast --release -- --nocapture`

use omni_mdx_core::ast::{AstNode, AttrValue};
use omni_mdx_core::parser::parse_mdx;

// ============================================================================
// Helper Functions
// ============================================================================

/// Finds the first child node matching the given `node_type` at the top level.
fn find<'a>(nodes: &'a [AstNode], node_type: &str) -> Option<&'a AstNode<'a>> {
    nodes.iter().find(|n| n.node_type == node_type)
}

/// Recursively performs a Depth-First Search (DFS) to find all nodes of a specific type.
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

/// Extracts and concatenates all text content from the immediate children of a node.
fn text_content(node: &AstNode) -> String {
    node.children
        .iter()
        .filter(|c| c.node_type == "text")
        .filter_map(|c| c.content.as_deref())
        .collect::<Vec<_>>()
        .join("")
}

/// Extracts the string value of a specific attribute, if it exists and is a text attribute.
fn attr_text<'a>(node: &'a AstNode, key: &str) -> Option<&'a str> {
    match node.attributes.as_ref()?.get(key)? {
        AttrValue::Text(s) => Some(&*s),
        _ => None,
    }
}

// ============================================================================
// Mock Data
// ============================================================================

const MDX: &str = "
# Titre principal

Paragraphe avec du texte **gras** et *italique*.

<Note type=\"warning\" title=\"Point de vigilance\">
  Ce modèle n'est valide que pour $t > 0$.
</Note>

<Details title=\"Démonstration\">
  L'équation fondamentale :
  $$ E = mc^2 $$
  Voir ci-dessus.
</Details>

<Table
  caption=\"Métriques\"
  headers={[\"Nom\", \"Valeur\"]}
  data={[[\"Latence\", 12], [\"Débit\", 1024]]}
/>

$$
\\sum_{i=1}^{n} x_i
$$

## Section 2

- Premier élément
- Deuxième élément

## Link
[Link text](https://example.com)
";

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_markdown_headers_and_paragraphs() {
    let ast = parse_mdx(MDX).expect("parse_mdx failed");

    let h1 = find(&ast, "h1").expect("h1 missing");
    assert_eq!(text_content(h1), "Titre principal");

    assert!(find(&ast, "h2").is_some());

    let paras = find_all(&ast, "p");
    assert!(paras.len() >= 2);

    let first_p = paras[0];
    assert!(first_p.children.iter().any(|c| c.node_type == "strong"));
    assert!(first_p.children.iter().any(|c| c.node_type == "em"));
}

#[test]
fn test_jsx_note_component_with_inline_math() {
    let ast = parse_mdx(MDX).expect("parse_mdx failed");
    let note = find(&ast, "Note").expect("Note component missing");

    assert_eq!(attr_text(note, "type"), Some("warning"));
    assert_eq!(attr_text(note, "title"), Some("Point de vigilance"));

    // Inline Math Check (Using data-math attribute)
    let math = note
        .children
        .iter()
        .find(|c| c.node_type == "InlineMath")
        .expect("InlineMath inside Note missing");
    assert_eq!(attr_text(math, "data-math"), Some("t > 0"));

    // Checking for non-wrapping (React hoisting rules)
    let is_wrapped = note
        .children
        .iter()
        .filter(|c| c.node_type == "p")
        .any(|p| p.children.iter().any(|c| c.node_type == "InlineMath"));
    assert!(
        !is_wrapped,
        "InlineMath should not be wrapped in <p> inside JSX"
    );
}

#[test]
fn test_jsx_details_with_block_math() {
    let ast = parse_mdx(MDX).expect("parse_mdx failed");
    let details = find(&ast, "Details").expect("Details missing");

    assert_eq!(attr_text(details, "title"), Some("Démonstration"));

    let bmath = details
        .children
        .iter()
        .find(|c| c.node_type == "BlockMath")
        .expect("BlockMath inside Details missing");
    assert_eq!(attr_text(bmath, "data-math"), Some("E = mc^2"));

    // No <p> around BlockMath
    let is_wrapped = details
        .children
        .iter()
        .filter(|c| c.node_type == "p")
        .any(|p| p.children.iter().any(|c| c.node_type == "BlockMath"));
    assert!(!is_wrapped, "BlockMath should not be wrapped in <p>");
}

#[test]
fn test_jsx_table_expression_attributes() {
    let ast = parse_mdx(MDX).expect("parse_mdx failed");
    let table = find(&ast, "Table").expect("Table missing");

    assert!(table.self_closing);
    assert_eq!(attr_text(table, "caption"), Some("Métriques"));

    let headers = table.attributes.as_ref().and_then(|a| a.get("headers"));
    match headers {
        Some(AttrValue::Expression(_)) => (),
        _ => panic!("Expected Expression for headers attribute"),
    }
}

#[test]
fn test_root_level_block_math() {
    let ast = parse_mdx(MDX).expect("parse_mdx failed");
    let bmaths = find_all(&ast, "BlockMath");

    assert!(bmaths.len() >= 2, "Should find at least 2 BlockMath nodes");

    let has_math = bmaths.iter().any(|n| {
        attr_text(n, "data-math").map_or(false, |c| c.contains("int") || c.contains("sum"))
    });

    assert!(
        has_math,
        "Root-level math not found. Available BlockMath contents: {:?}",
        bmaths
            .iter()
            .map(|n| attr_text(n, "data-math").unwrap_or(""))
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_markdown_lists() {
    let ast = parse_mdx(MDX).expect("parse_mdx failed");
    let ul = find(&ast, "ul").expect("ul missing");
    let items = find_all(&ul.children, "li");
    assert_eq!(items.len(), 2);
}

#[test]
fn test_utf8_and_mojibake_protection() {
    let ast = parse_mdx(MDX).expect("parse_mdx failed");
    let json = serde_json::to_string(&ast).unwrap();

    // Mojibake Test (C3 A9 -> é)
    assert!(
        !json.contains("\u{00c3}\u{00a9}"),
        "Mojibake detected in JSON"
    );
    assert!(json.contains('é'), "UTF-8 character 'é' corrupted");
}

#[test]
fn test_markdown_links() {
    let ast = parse_mdx(MDX).expect("parse_mdx failed");

    let links = find_all(&ast, "a");
    let link = links.get(0).expect("Link (a) node missing in AST");

    assert_eq!(attr_text(link, "href"), Some("https://example.com"));
    assert_eq!(text_content(link), "Link text");
}

#[test]
fn test_math_content_persistence() {
    let mdx = "Note with formula : $e = mc^2$";

    let ast = parse_mdx(mdx).expect("parse_mdx failed");

    let mut math_node_found = false;

    for node in &ast {
        for child in &node.children {
            if child.node_type == "InlineMath" {
                math_node_found = true;

                let data_math = attr_text(child, "data-math");
                assert!(
                    data_math.is_some(),
                    "FAILED: 'data-math' attribute is missing for InlineMath"
                );

                let content = data_math.unwrap();
                assert!(
                    !content.is_empty(),
                    "FAILED: The 'data-math' attribute is an empty string"
                );
                assert_eq!(
                    content, "e = mc^2",
                    "FAILED: The formula content was altered"
                );
            }
        }
    }

    assert!(
        math_node_found,
        "FAILED: The InlineMath node was not created in the AST"
    );
}

#[test]
fn test_table_ast_contains_row_in_head() {
    let markdown_input = "\
| Header 1 | Header 2 |
| -------- | -------- |
| Cell 1   | Cell 2   |
";

    let ast = parse_mdx(markdown_input).expect("Markdown parsing failed");

    let table = ast
        .iter()
        .find(|n| n.node_type.to_lowercase() == "table")
        .expect("Table not found in the AST");

    let thead = table
        .children
        .first()
        .expect("The table has no children (thead missing)");

    assert_eq!(
        thead.node_type.to_lowercase(),
        "thead",
        "The first child of the table must be a thead"
    );

    let first_head_child = thead
        .children
        .first()
        .expect("The thead is empty, it is missing its children!");

    assert_eq!(
        first_head_child.node_type.to_lowercase(),
        "tr",
        "The thead contains a '{}' instead of a 'tr'. The parser omits the header row!",
        first_head_child.node_type
    );

    let first_cell = first_head_child.children.first().expect("The tr is empty");

    assert!(
        first_cell.node_type.to_lowercase() == "th" || first_cell.node_type.to_lowercase() == "td",
        "The tr must contain cells (th/td), but contains '{}'",
        first_cell.node_type
    );
}

#[test]
fn test_table_ast_contains_tbody_with_rows() {
    let markdown_input = "\
| Header 1 | Header 2 |
| -------- | -------- |
| Cell 1   | Cell 2   |
| Cell 3   | Cell 4   |
";

    let ast = parse_mdx(markdown_input).expect("Markdown parsing failed");

    let table = ast
        .iter()
        .find(|n| n.node_type.to_lowercase() == "table")
        .expect("Table not found in the AST");

    let tbody = table
        .children
        .iter()
        .find(|n| n.node_type.to_lowercase() == "tbody")
        .expect("The table has no 'tbody' node!");

    assert_eq!(
        tbody.children.len(),
        2,
        "The tbody should contain exactly 2 rows (tr)"
    );

    for (i, row) in tbody.children.iter().enumerate() {
        assert_eq!(
            row.node_type.to_lowercase(),
            "tr",
            "Child {} of the tbody is a '{}' instead of a 'tr'",
            i,
            row.node_type
        );

        for (j, cell) in row.children.iter().enumerate() {
            assert_eq!(
                cell.node_type.to_lowercase(),
                "td",
                "Cell {} of row {} is a '{}' instead of a 'td'",
                j,
                i,
                cell.node_type
            );
        }
    }
}
