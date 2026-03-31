# 🦀 Omni MDX: Core Parser

Welcome to the official parsing engine of the Omni MDX ecosystem.

Written entirely in Rust for maximum performance and absolute memory safety, this module is responsible for ingesting potentially hostile Markdown files (extended with JSX and LaTeX math) and transforming them into a universal Abstract Syntax Tree (AST).

Instead of relying on slow JavaScript parsers, Omni-Core processes text natively and streams the AST directly to the host language (TypeScript/React or Python) using our zero-copy **OCP Binary Protocol**.

## ✨ Key Features
* **Zero-Trust Security Shield:** Built for public-facing web environments. The core includes a deterministic pre-processor that detects and deflects Algorithmic Complexity attacks ($O(n^2)$ catastrophic backtracking), WASM Stack Overflows, and malformed symbol entropy bombs in constant time.

* **Surgical JSX Cloaking:** Custom tags (such as `<Speaker>` or `<DataChart>`) are temporarily hidden from the standard Markdown engine to prevent them from being accidentally wrapped in `` paragraphs.

* **Math Sanitization:** LaTeX equations (`$` and `$$`) are extracted before parsing. A `>` character in a math equation will never break the document structure.

* **Dynamic Attribute Typing:** The parser identifies whether a JSX prop is a string (`name="Dr. Laurent"`), an evaluated expression (`data={[1, 2, 3]}`), or a boolean.

* **True Multi-Target:** Compiled to **WebAssembly (WASM)** for the browser, native **Node.js addons** via `napi-rs`, and **Python wheels** via `PyO3/maturin` .

## 🏗️ Internal Architecture

The source code (`src/`) is divided into several specialized modules:

* `parser.rs`: The main entry point. Houses the event loop, the AST construction, and the **Omni-Core Security Shields**.

* `jsx.rs`: A custom parsing engine designed exclusively for React-style tags and the extraction of their complex attributes.

* `ast.rs`: Defines the data structures (nodes and attribute types) that are serialized into the OCP Binary format.

* `lib.rs`: The FFI bridges and public bindings for `wasm-bindgen`, `napi`, and Python exports.

## 🚀 Compilation & Testing
The parser's lifecycle is managed by the `Makefile` located at the root of the monorepo.

### 1. Run the Rust test suite:
We maintain strict performance and security benchmarks. Run the comprehensive test suites to ensure the shields hold against fuzzing payloads:

```bash
cargo test
```

### 2. Compile for the Web (WASM):
To generate the ultra-lightweight `.wasm` binary for the browser:

```bash
wasm-pack build --target web --features wasm
```

### 3. Build Native Addons (Node.js & Python)
* **Node.js:** `napi build --platform --release --features node`
* **Python:** `maturin develop --release` (inside `packages/mdx-python`)

## 🌳 Structure of the Generated AST
While the data is transferred in binary for speed, the parser's logical output represents a strict, predictable tree. Here is what the standardized structure looks like conceptually:

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
        "content": "Energy is "
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

## 🚨 Error Handling & Resilience
The philosophy of `core-parser` is "Zero Panic". If the document contains malformed JSX or exceeds complexity limits, the parser will never crash the host application.

Instead, it injects actionable error nodes directly into the AST or returns a graceful `ParseError`. This allows client-side interfaces (like Next.js) to display localized "Error Boundaries" without taking down the entire page.