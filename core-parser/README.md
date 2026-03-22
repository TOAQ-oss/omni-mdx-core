# 🦀 Omni MDX: Core Parser

Welcome to the official parsing engine of the Omni MDX ecosystem.

Written entirely in Rust for maximum performance and absolute memory safety, this module is responsible for ingesting submissions from researchers (Markdown files extended with JSX and math) and transforming them into a universal Abstract Syntax Tree (AST) in JSON format.

This parser ensures that the text, metadata, and structure of podcast episodes are fully validated before being sent to the front-end (React) or the machine learning pipeline (Python) for the voice dataset.

## ✨ Key Features
* **Surgical JSX Cloaking:** Custom tags (such as `<Speaker>` or `<DataChart>`) are temporarily hidden from the standard Markdown engine to prevent them from being accidentally wrapped in `` paragraphs.

* **Math Sanitization:** LaTeX equations (`$` and `$$`) are extracted before parsing. A `>` character in a math equation will never break the document structure.

* **Dynamic Attribute Typing:** The parser identifies whether a JSX prop is a string (`name=“Dr. Laurent”`), an evaluated expression (`data={[1, 2, 3]}`), or a boolean.

* **Zero Panic:** The architecture handles syntax errors gracefully. If a tag has not been closed, the engine returns a clean error node instead of crashing the application.

* **Multi-Target:** The code is designed to be compiled into **WebAssembly (WASM)** for the web, and exposed via **C-FFI** for the Python ecosystem.

## 🏗️ Internal Architecture

The source code (`src/`) is divided into several specialized modules:

* `lexer.rs`: Primary parser. Handles the wrapping of math blocks and the identification of JSX components.

* `jsx.rs`: Parsing engine specifically designed for custom tags and the extraction of their complex attributes.

* `markdown.rs`: Handles integration with `pulldown-cmark` for the CommonMark standard specification.

* `ast.rs`: Defines the data structures (nodes and attribute types) that will be serialized to JSON.

* `lib.rs` / `ffi.rs`: The public entry points for WASM (`wasm-bindgen`) and Python (`extern “C”`) exports.

## 🚀 Compilation & Testing
The parser's lifecycle is managed by the `Makefile` located at the root of the monorepo.

### 1. Run the Rust test suite:
Ensure that the parser correctly handles errors, the AST, and performance:

```bash
make test
```
*(This runs the `test_ast`, `test_errors`, and `test_perf` binaries located in `src/bin/`)*

### 2. Compile for the Web (WASM):
To generate the ultra-lightweight `.wasm` binary and TypeScript definitions for the `@toaq-oss/omni-mdx` module:

```bash
make build-web
```
*(The compiled files will be automatically copied to `packages/mdx-next/omni-core/`)*

### 3. Clean up build artifacts:

```bash
make clean
```

## 🌳 Structure of the Generated AST
The parser transforms any MDX script into a strict JSON stream. Here is what the standardized output looks like:

**Input (MDX):**

```mdx
<Speaker name="Dr. Dupont" time="00:15">
  Energy is $E = mc^2$.
</Speaker>
```
Output (JSON AST):

```json
[
  {
    "node_type": "Speaker",
    "attributes": {
      "name": { "kind": "text", "value": "Dr. Dupont" },
      "time": { "kind": "text", "value": "00:15" }
    },
    "children": [
      {
        "node_type": "text",
        "content": "L'énergie est "
      },
      {
        "node_type": "InlineMath",
        "content": "E = mc^2"
      },
      {
        "node_type": "text",
        "content": "."
      }
    ]
  }
]
```

## 🛡️ Error Handling
The philosophy of `core-parser` is to always provide actionable feedback. If the document contains malformed JSX, the parser will inject a specific error node directly into the AST:

```
{
  “node_type”: “error”,
  “content”: “Unexpected token near ‘<Speaker name=\”...’"
}
```
This allows client-side interfaces to display localized “Error Boundaries” without breaking the rest of the page or script.