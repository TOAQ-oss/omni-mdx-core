# Changelog — core-parser (Rust)

All notable changes to the Rust MDX parser crate are documented here.

---

## [0.2.0] — 2026-03-13

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