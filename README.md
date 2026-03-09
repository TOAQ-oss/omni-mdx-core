# 🦀 Omni-MDX
## A fast, cross-platform, cross-language MDX rendering engine powered by Rust.

Omni-MDX is designed to transform complex scientific documents (LaTeX, MDX, interactive components) into a universal data tree (AST). Initially created for the TOAQ platform, this engine allows MDX (Markdown Extended) to be viewed on web, app, and software projects.

## 🚀 Project Vision
Omni-MDX aims to solve the problem of parser fragmentation. Instead of having one parser in JavaScript for the website and another in Python for data processing, Omni-MDX offers a **Single Core in Rust** compiled for each environment.
* **Native Speed:** High-performance parsing thanks to Rust.
* **Math-Ready:** Native support for KaTeX ($E=mc^2$) and equation blocks.
* **Component Agnostic:** The engine identifies JSX/MDX components and delegates rendering to the host platform.
* **Multi-Target:** A single source code for the web (WASM), Python (AI/Datasets), and soon mobile (Dart/FFI).

## 🏗️ Architecture du Monorepo
```
omni-mdx/
├── core-parser/
│   ├── src/
│   └── bindings/
├── packages/
│   ├── mdx-next/
│   └── mdx-python/ [In progress]
└── sandbox/
    └── next/
```

##  🛠️ Installation & Development
### Prerequisites
* [Rust](https://rustup.rs/) (latest stable version)
* [Node.js](https://nodejs.org/fr) (v18+)
* [wasm-pack](https://www.google.com/search?q=https://rustwasm.github.io/wasm-pack/installer/) (for the web part)

### 1. Compile the engine (WASM)
From the project root directory:
```bash
make setup

make build-web
```
2. Launch the sandbox (Next.js)
```bash
cd sandbox/next
npm install
npm run dev --webpack
```
## 📦 Package Distribution
Each module is published independently on its respective registry:

|Package|Platform|Installation|
|:--|:--|:--|
| @toaq-oss/mdx-engine | NPM / Next.js |npm install @toaq-oss/mdx-engine |
| toaq-mdx (Soon) | PyPI / Python | pip install toaq-mdx |
| toaq_mdx (Soon) | Pub / Dart | pub add tpaq_mdx |

## 📄 License
This project is licensed under the MIT License. You are free to use, modify, and distribute it.
