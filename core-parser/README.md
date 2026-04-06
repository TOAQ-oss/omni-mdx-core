# 🦀 Omni-MDX: The Core Engine

**The high-performance, security-hardened heart of the Omni-MDX ecosystem.**

Written entirely in **Rust** for maximum throughput and absolute memory safety, `core-parser` is responsible for ingesting potentially hostile MDX content (Markdown + JSX + LaTeX) and transforming it into a universal, language-agnostic Abstract Syntax Tree (AST).



Instead of relying on heavy JavaScript parsers, Omni-Core processes text natively and streams the resulting AST directly to the host environment (Node.js, Python, or the Browser) using our ultra-efficient **OCP (Omni-Core Protocol) Binary Bridge**.

---

## ✨ Key Features

* **🛡️ Zero-Trust Security Shield:** Built for public-facing production. The core includes a deterministic pre-processor that detects and deflects **Algorithmic Complexity (ReDoS)** attacks, WASM stack overflows, and malformed entropy bombs—all in constant time.
* **🎭 Surgical JSX Cloaking:** Custom tags (like `<Speaker />` or `<DataChart />`) are isolated from the standard Markdown engine. This prevents the "paragraph-wrapping bug" where JSX is accidentally treated as standard text.
* **➗ Math Sanitization:** LaTeX equations ($$ and $) are extracted via a specialized pass. Characters like `>` or `_` inside a formula will never break your document's block structure.
* **💎 Dynamic Attribute Typing:** The parser intelligently identifies prop types:
    * **Strings:** `name="Dr. Dupont"`
    * **Expressions:** `data={[1, 2, 3]}`
    * **Booleans:** `isHidden` (implicitly true)
* **🌍 True Multi-Target:** A single codebase compiled for every environment:
    * **Node.js:** Native addons via `napi-rs`.
    * **Python:** High-speed wheels via `PyO3` and `maturin`.
    * **Browser/Edge:** Lightweight `.wasm` binaries via `wasm-pack`.

---

## 🏗️ Internal Architecture

The `src/` directory is organized into specialized, decoupled modules:

| Module | Responsibility |
| :--- | :--- |
| **`parser.rs`** | The main orchestration loop, AST construction, and the security pre-processor. |
| **`jsx.rs`** | A dedicated lexer for React-style tags and complex attribute parsing. |
| **`ast.rs`** | Data structures for nodes and types, designed for zero-copy serialization. |
| **`lib.rs`** | The FFI gateway: Exports for `wasm-bindgen`, `napi`, and `Python`. |



---

## 🌳 AST Structure: Concept vs. Output

While the data travels via binary protocol for speed, the logical output is a predictable, strictly typed tree.

**Input (MDX):**
```mdx
<Speaker name="Dr. Dupont" time="00:15">
  Energy is $E = mc^2$.
</Speaker>
```

**Conceptual JSON Representation:**
```json
[
  {
    "node_type": "Speaker",
    "attributes": {
      "name": "Dr. Dupont",
      "time": "00:15"
    },
    "children": [
      { "node_type": "text", "content": "Energy is " },
      { "node_type": "InlineMath", "content": "E = mc^2" },
      { "node_type": "text", "content": "." }
    ]
  }
]
```

---

## 🚀 Compilation & Testing

The parser's lifecycle is managed via the root `Makefile` or standard `cargo` commands.

### 1. Test the Shields
We maintain a strict suite of performance and fuzzing benchmarks to ensure the engine never regresses.
```bash
cargo test
```

### 2. Build for Web (WASM)
Generates the optimized `.wasm` and JS glue code for browser-side rendering.
```bash
wasm-pack build --target web --features wasm
```

### 3. Build for Native Host
* **Node.js Addon:** `napi build --platform --release --features node`
* **Python Extension:** `maturin develop --release` (run from the Python package dir)

---

## 🚨 Error Handling & Resilience

Our philosophy is **"Zero Panic."** The core is designed to be indestructible; it will never crash the host process, even when faced with deeply nested or malformed JSX.

Instead of panicking, the engine injects **Actionable Error Nodes** directly into the AST. This allows modern UI frameworks (like React or PyQt5) to catch errors at the component level, displaying a "Warning" block instead of a blank page.

## 🤝 Contributing
This crate is part of the **TOAQ** open-source ecosystem.
* **Core Engine (Rust):** [TOAQ-oss/omni-mdx-core](https://github.com/TOAQ-oss/omni-mdx-core)
* **Bug Tracker:** [GitHub Issues](https://github.com/TOAQ-oss/omni-mdx-core/security/advisories)