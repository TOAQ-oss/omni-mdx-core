# Changelog — core-parser (Rust)

All notable changes to the Rust MDX parser crate are documented here.

## [1.1.0] - 2026-04-07

### Changed
- **Math AST Structure**: LaTeX formulas (`InlineMath` and `BlockMath`) now store their raw content inside the `data-math` attribute rather than the generic `content` field. This ensures mathematical expressions survive transformations by HTML-focused external plugins (like Unified/Rehype).
- **Memory Management**: Introduced the `promote_to_owned` utility in the JSX parser. This safely escalates `Cow::Borrowed` node lifetimes to `Cow::Owned` when local string mutations are required, maintaining the engine's zero-copy architecture without triggering borrow checker conflicts.

### Fixed
- **Lowercase HTML Tag Parsing**: Upgraded the lexer and JSX parser to use `.is_ascii_alphabetic()` instead of `.is_ascii_uppercase()`. Standard HTML tags (e.g., `<div>`, `<span>`) within MDX are now correctly extracted into the JSX pool and parsed as structured nodes with attributes, rather than being downgraded to raw text.
- **JSX Indentation Code-Block Bug**: Implemented a "Dindent Hack" in `jsx.rs`. Text indented with 4 spaces inside custom JSX components is no longer erroneously parsed as Markdown code blocks (`<pre><code>`). The parser now automatically cleans relative indentation before passing the block to the Markdown engine.
- **Dependency Compatibility**: Updated pattern matching for `Tag::BlockQuote` to resolve compilation errors with recent versions of the `pulldown-cmark` crate.

---
## [0.2.4] - 2026-03-31

### Security
- **Universal Omni-Core Shield**: Implemented an advanced symbol entropy filter within the Rust engine. The parser now mathematically detects and drops anomalous, malformed payloads (unnatural density of non-alphanumeric characters) before AST generation.
- **WASM Memory Protection**: Added structural limits to prevent WebAssembly Stack Overflow (OOM) crashes caused by extreme nesting, deep blockquotes (`>`), and ambiguous nested lists.

### Fixed
- Prevented browser main-thread freezes in the React/Next.js client by instantly rejecting malicious payloads at the Rust/C level with a clear `ParseError: ComplexityLimitExceeded`.

---

## [0.2.3] - 2026-03-31

### Security
- **Algorithmic Complexity Mitigation (DoS)**: Introduced strict execution bounding for specific markdown tokens. The parser now successfully deflects $O(n^2)$ catastrophic backtracking attacks (CWE-400 Resource Exhaustion) caused by unclosed references or footnotes.
- **Payload Limits**: Enforced a strict 2MB global size limit at the core level to ensure stable RAM usage across all supported environments (WASM, Node.js, and Python).

[0.2.0] — 2026-03-18
## 🚀 Features & Architecture
* **Zero-Copy Memory Management:** Eliminated the FFI JSON serialization bottleneck. The AST is now safely stored in Rust memory behind an Arc and accessed lazily from JavaScript via lightweight N-API method calls (MdxAst and MdxNode). Memory is automatically reclaimed when V8 garbage-collects the JS references, dropping the Arc refcount to zero.

* **Optimized Binary Protocol:** Introduced a custom high-performance binary encoder (parse_to_binary) that bypasses V8's JSON parser entirely, returning a highly compressed Uint8Array directly to Node.js.

* **Native JSX Compilation:** Implemented compile_to_jsx to compile MDX directly into a JSX string entirely within Rust. This provides the absolute fastest rendering path for React/Next.js applications.

* **Industrial Fuzz Testing:** Integrated cargo-fuzz (libFuzzer) to bombard the parser with millions of malformed byte sequences, guaranteeing complete panic-resistance and memory safety for unpredictable inputs.

## 🐛 Bug Fixes
* **LaTeX Content Preservation:** Fixed a critical data-loss issue where InlineMath and BlockMath nodes were structurally recognized but their inner string contents were dropped. The expand_text pipeline now explicitly unmasks and assigns the raw LaTeX string to the content field.

* **React Server Components (RSC) Hydration:** Resolved an issue where Next.js Server Components silently stripped N-API C++ class getters (like content and node_type) during prop passing. Enforced parseToJson (camelCase mapped by napi-rs) as a secure fallback to ensure pure JavaScript object hydration.

* **Trailing Text Masking Leaks:** Patched a vulnerability in expand_text where the final text chunk following the last placeholder skipped the unmask_code pass, potentially leaking raw \x01 bytes into the final AST.

* **Math Detection Edge Cases:** Corrected the has_math flag logic inside extract_math to guarantee STX/ETX placeholders are reliably generated even for short formulas or edge-case string indices.

## 🧪 Testing
* **AST Integrity:** Added the test_math_content_persistence integration test to strictly verify that formulas survive the entire extraction, unmasking, and serialization pipeline without truncation or mutation.

## [0.1.3] — 2026-03-13

### Initial Public Release

First stable version of the MDX parser in Rust. This crate forms the core of the `omni-mdx-core` engine—it is used by the npm package `@toaq-oss/omni-mdx` via napi-rs, by the Python package `omni-mdx` via PyO3/maturin, and can be compiled to WASM via wasm-bindgen.

---

### Architecture

#### Parsing Pipeline

Parsing follows a strict three-pass process designed to prevent conflicts between MDX, JSX, and LaTeX grammars:

```
input
  │
  ▼
extract_math()       ← First pass — extract $...$ and $$...$$ first
  │
  ▼
extract_jsx()        ← 2nd pass — extracts the JSX blocks
  │
  ▼
parse_markdown()     ← 3rd pass — parses the Markdown using pulldown-cmark
```

The order is critical: mathematical expressions must be parsed before the JSX lexer, because `$t > 0$` contains `>` and `<` characters that would otherwise be interpreted as tags.

#### Placeholder System

Extracted blocks are replaced with non-printable tokens to prevent any conflicts with the MDX content:

| Token | Meaning |
|---|---|
| `\x02JSXn\x03` | JSX block number n |
| `\x02MATHBn\x03` | Display math block `$$...$$` |
| `\x02MATHIn\x03` | Inline math block `$...$` |

#### AST

```rust
pub enum AttrValue {
    Text(String),
    Expression(String),
    Boolean,
    Ast(Vec<AstNode>),
}

pub struct AstNode {
    pub node_type:     String,
    pub content:       Option<String>,
    pub self_closing:  bool,
    pub attributes:    HashMap<String, AttrValue>,
    pub children:      Vec<AstNode>,
}
```

---

### Features de compilation

| Feature | Backend | Usage |
|---|---|---|
| `node` | napi-rs | Addon Node.js natif (`.node`) |
| `python` | PyO3 | Extension Python (`.pyd` / `.so`) |
| `wasm` | wasm-bindgen | Module WASM (browser / Edge) |
| `ffi` | C FFI | Interop C générique |

---

### Parser Features

- **Full Markdown support** via pulldown-cmark: headings, paragraphs, ordered and unordered lists, nested lists, blockquotes, GFM tables, fenced code blocks with language, inline code, bold, italics, strikethrough, links, images, horizontal lines, line breaks
- **JSX**: custom components with typed attributes (`text`, `expression`, `boolean`, `ast`), mixed children (text + inline + blocks), auto-closing, nesting
- **Complex JSX attributes**: `title=“foo”` → `Text`, `left={}` → `Ast`, `disabled` → `Boolean`, `count={42}` → `Expression`
- **Inline math**: `$...$` → `InlineMath` node
- **Display math**: `$$...$$` → `BlockMath` node
- **Paragraph deduplication**: standalone JSX blocks are not wrapped in a ``
- **Automatic indentation**: indented content within JSX components is normalized

---

### Bugs fixed during development

1. **`__JSX_N__` placeholder** — replaced with `\x02JSXn\x03` to avoid conflicts with user content
2. **`bytes[i-1]` panic when `i == 0`** — zero-index guard added to the lexer
3. **`` wrapping a single JSX node** — `unwrap_solo_jsx_paragraph` rewritten to detect paragraphs containing only a single block node
4. **`$t > 0$` causing the lexer to crash** — resolved by reordering the (math before JSX)
5. **`init()` WASM is not a function** — `wasm-bindgen` made optional, `wasm` feature isolated
6. **`Unclosed JSX block`** — JSX lexer state machine rewritten to handle nested tags
7. **`Unexpected token ‘?’`** — `parse_children` was calling `parse_mdx` recursively instead of `parse_markdown`
8. **wasm-pack build failing** — `wasm-bindgen` extracted as a conditional optional dependency
9. **UTF-8 corruption** — introduction of `utf8_char_len()` and `from_utf8` for all byte slices
10. **`left={}` rendered as a string** — `parse_html_wrapper` now calls `parse_markdown` for `ast` attributes
11. **Indented content → `````** — `dedent()` function added to JSX child preprocessing
12. **`` inside `` (React hydration error)** — `unwrap_solo_jsx_paragraph` rewritten a second time to cover BlockMath cases
13. **`InlineMath` promoted to a block** — `is_block()` explicitly excludes `InlineMath`

---

### Compilation

```bash
# Node.js add-on
napi build --platform --release --features node --no-js

# Python extension
maturin develop --release --features python

# WASM
wasm-pack build --target bundler --features wasm
```

---

### Main Dependencies

| Crate | Role |
|---|---|
| `pulldown-cmark` | Markdown parser (CommonMark + GFM) |
| `serde` + `serde_json` | AST to JSON serialization |
| `napi` + `napi-derive` | Node.js bindings (`node` feature) |
| `pyo3` | Python bindings (`python` feature) |
| `wasm-bindgen` | WASM bindings (`wasm` feature) |