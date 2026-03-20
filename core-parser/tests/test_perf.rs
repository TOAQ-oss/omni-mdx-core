/// test_perf.rs — Performance and edge-case robustness tests.
///
/// Chosen because the parser must survive real-world content:
///   - Large documents (won't be slow or OOM)
///   - Deeply nested JSX (won't stack-overflow)
///   - Heavy UTF-8 (accents, CJK, emoji) everywhere in the pipeline
///   - Math and JSX interleaved in every possible position
///   - Attribute expressions with complex JS literals
///
/// Run with: `cargo test --test test_perf --release -- --nocapture`

use omni_mdx_core::parser::parse_mdx;
use std::time::{Duration, Instant};

// Helper function for timed parsing assertions
fn assert_timed(label: &str, input: &str, max: Duration) {
    let t = Instant::now();
    match parse_mdx(input) {
        Err(e) => panic!("{}: unexpected error: {}", label, e),
        Ok(_) => {
            let elapsed = t.elapsed();
            assert!(
                elapsed <= max,
                "{}: too slow: {:?} > {:?}",
                label,
                elapsed,
                max
            );
            println!("   {} ({:?})", label, elapsed);
        }
    }
}

#[test]
fn test_large_document_performance() {
    let mut doc = String::new();
    for i in 0..100 {
        doc.push_str(&format!(
            "## Section {i}\n\nTexte avec **gras** et *italique* au niveau {i}.\n\n\
             <Note type=\"info\" title=\"Note {i}\">\n  \
               Contenu de la note numéro {i} avec $x_{{i}} > 0$.\n\
             </Note>\n\n\
             <Table\n  \
               caption=\"Tableau {i}\"\n  \
               headers={{[\"A\", \"B\"]}}\n  \
               data={{[[{i}, {}]]}}\n\
             />\n\n",
            i + 1
        ));
    }
    assert_timed("500-component document", &doc, Duration::from_millis(500));
}

#[test]
fn test_deeply_nested_jsx() {
    let open: String = (0..20).map(|i| format!("<Box level=\"{i}\">\n")).collect();
    let inner: &str = "  Contenu profondément imbriqué.";
    let close: String = (0..20).map(|_| "</Box>\n").collect();
    let doc = format!("{open}{inner}\n{close}");
    assert_timed("20-level nesting", &doc, Duration::from_millis(200));
}

#[test]
fn test_utf8_stress() {
    let doc = format!(
        "# T\u{00ed}tre avec \u{00e9}l\u{00e9}ments sp\u{00e9}ciaux\n\n\
         \u{4e2d}\u{6587}\u{5185}\u{5bb9} (Chinese) et contenu arabe mixed.\n\n\
         <Note type=\"info\" title=\"Caf\u{00e9} \u{00e9}t\u{00e9}\">\n  \
           Résumé : à propos de l’équation $E = mc^2$.\n  \
           \u{1f600}\u{1f680}\u{2764} emoji dans le texte.\n\
         </Note>\n\n\
         <Table\n  \
           caption=\"Données élément\"\n  \
           headers={{[\"étape\", \"résultat\"]}}\n  \
           data={{[[\"étape 1\", 42]]}}\n\
         />\n"
    );

    let ast = parse_mdx(&doc).expect("UTF-8 stress parse failed");
    let json = serde_json::to_string(&ast).unwrap();

    assert!(
        !json.contains("\u{00c3}\u{00a9}") && !json.contains("\u{00c3}\u{00a0}"),
        "UTF-8 corruption detected in output"
    );
    assert!(json.contains("T\u{00ed}tre"), "Title not found in JSON round-trip");
}

#[test]
fn test_math_positions() {
    let doc = "\
# $\\alpha$ in heading
Inline $x^2 + y^2 = z^2$ in paragraph.
$$
\\int_0^\\infty e^{-x} dx = 1
$$
<Note type=\"info\" title=\"Inline in attr is text\">
  Block math inside JSX children:
  $$ F = ma $$
  And inline $v = d/t$ too.
</Note>
<Details title=\"Math demo\">
  First $a > b$, then:
  $$ c = \\sqrt{a^2 + b^2} $$
  Result.
</Details>";

    let ast = parse_mdx(doc).expect("Math positions parse failed");
    let json = serde_json::to_string(&ast).unwrap();

    let inline_count = json.matches("InlineMath").count();
    let block_count = json.matches("BlockMath").count();

    assert!(inline_count >= 4, "Expected >= 4 InlineMath, got {inline_count}");
    assert!(block_count >= 3, "Expected >= 3 BlockMath, got {block_count}");
}

#[test]
fn test_complex_attribute_expressions() {
    let doc = r####"
<SplitLayout
  left={
    <div className="prose">
### Titre de gauche
Contenu avec $x > 0$ et **gras**.
    </div>
  }
  right={<Note>Droit</Note>}
/>
"####;

    let ast = parse_mdx(doc).expect("Complex expressions parse failed");
    let json = serde_json::to_string(&ast).unwrap();

    assert!(
        json.contains(r#""node_type":"h3""#) || json.contains(r#""node_type": "h3""#),
        "h3 not found in nested markdown. Check if parser handles indented block headers."
    );
}