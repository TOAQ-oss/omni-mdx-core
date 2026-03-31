# 🦀 Omni-MDX
## A blazing-fast, secure, cross-language MDX rendering engine powered by Rust.

Omni-MDX is designed to transform complex MDX documents (Markdown + JSX + LaTeX) into a universal, high-performance data tree (AST). Initially created for the TOAQ platform, this engine eliminates the JavaScript parsing bottleneck, allowing heavy MDX content to be processed instantly and securely across Web, Edge, and Data-Science environments.

## 🚀 Project Vision
Omni-MDX aims to solve parser fragmentation and performance limits in the modern content ecosystem. Instead of relying on slow, memory-heavy JavaScript parsers for the web and completely different tools for Python, Omni-MDX provides a **Single Source of Truth in Rust**.

* **Native Speed & Zero GC:** Bypasses V8 garbage collection spikes. The Rust engine parses documents in microseconds using flat memory and zero-copy string references (`Cow<'a, str>`).
* **Zero-Trust Security:** Built-in heuristics and execution bounding (The Omni-Core Shield) protect host environments from Algorithmic Complexity attacks (CWE-400), WASM Stack Overflows, and malformed payload DoS.
* **Math & Component Ready:** Flawless native support for KaTeX ($E=mc^2$) and surgical JSX cloaking. The engine identifies components without mutating their attributes.
* **Multi-Target Protocol:** The AST is serialized via our custom **OCP Binary Protocol** and streamed directly to WebAssembly (WASM), Node.js (napi), or Python (PyO3).

## 🏗️ Monorepo Architecture

This polyglot monorepo houses the core engine and its language-specific bindings:
```
omni-mdx/
├── core-parser/        # The heart of the engine (Rust AST, Lexer, Security Shields)
│   └── src/
└── packages/
    ├── mdx-next/       # TypeScript/React wrapper (Consumes WASM/Native C-FFI)
    └── mdx-python/     # Python bindings (Powered by PyO3 and Maturin)
```

##  🛠️ Installation & Development
### Prerequisites
* [Rust](https://rustup.rs/) (latest stable version)
* [Node.js](https://nodejs.org/fr) (v18+)
* [Python 3](https://www.python.org/downloads/) & `pip` (for the Python Package)
* [wasm-pack](https://www.google.com/search?q=https://rustwasm.github.io/wasm-pack/installer/) (for the web part)

### 1. Compile the Core Engine
From the project root directory, set up your environment and build the WebAssembly bridges:
```bash
# Install required Rust targets and build tools
make setup

# Compile the Rust core into WASM for the React package
make build-web
```
### 2. Run the Sandbox Environments

To keep this repository lightweight, all test applications and visual playgrounds have been moved to a dedicated sandbox repository.
```bash
# Clone the sandbox repository next to or inside your workspace
git clone [https://github.com/TOAQ-oss/omni-mdx-sandbox.git](https://github.com/TOAQ-oss/omni-mdx-sandbox.git) sandbox

# Example: Run the Next.js / React Playground
cd sandbox/next
npm install
npm run dev
```
*(You can link your local `@toaq-oss/omni-mdx` build to test core changes in real-time).*

## 📦 Package Distribution
Each module is published independently on its respective registry:

|Package|Platform|Installation|
|:--|:--|:--|
| [@toaq-oss/omni-mdx](https://www.npmjs.com/package/@toaq-oss/omni-mdx) | NPM / Next.js | `npm install @toaq-oss/omni-mdx` |
| [omni-mdx](https://pypi.org/project/omni-mdx/) | PyPI / Python | `pip install omni-mdx` |
| omni_mdx *(Soon)* | Pub / Dart | `pub add omni_mdx` |

## 🤝 Contributing
We welcome contributions! Please see our `CONTRIBUTING.md` for details on how to run the fuzzing suites, benchmark the engine, and submit Pull Requests.

## 📄 License
This project is licensed under the MIT License. You are free to use, modify, and distribute it.
