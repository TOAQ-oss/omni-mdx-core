# 🦀 Omni-MDX
## A fast, cross-platform, cross-language MDX rendering engine powered by Rust.

Omni-MDX is designed to transform complex MDX documents into a universal data tree (AST). Initially created for the TOAQ platform, this engine allows MDX (Markdown Extended) to be viewed on web, app, and software projects.

## 🚀 Project Vision
Omni-MDX aims to solve the problem of parser fragmentation. Instead of having one parser in JavaScript for the website and another in Python for data processing, Omni-MDX offers a **Single Core in Rust** compiled for each environment.
* **Native Speed:** High-performance parsing thanks to Rust.
* **Math-Ready:** Native support for KaTeX ($E=mc^2$) and equation blocks.
* **Component Agnostic:** The engine identifies JSX/MDX components and delegates rendering to the host platform.
* **Multi-Target:** A single source code for the web (WASM), Python (AI/Datasets), and soon mobile (Dart/FFI).

## 🏗️ Monorepo Architecture
```
omni-mdx/
├── core-parser/
│   ├── src/
│   └── bindings/
└── packages/
    ├── mdx-next/
    └── mdx-python/

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
2. Get the sandbox
```bash
mkdir tests/
cd tests/
git pull https://github.com/TOAQ-oss/omni-mdx-sandbox

# To use Next sandbox
cd omni-mdx-sandbox/next
npm install
npm run dev
```

## 📦 Package Distribution
Each module is published independently on its respective registry:

|Package|Platform|Installation|
|:--|:--|:--|
| [@toaq-oss/omni-mdx](https://www.npmjs.com/package/@toaq-oss/omni-mdx) | NPM / Next.js | npm install @toaq-oss/omni-mdx |
| [omni-mdx](https://pypi.org/project/omni-mdx/) | PyPI / Python | pip install omni-mdx |
| omni_mdx (Soon) | Pub / Dart | pub add toaq_mdx |

## 📄 License
This project is licensed under the MIT License. You are free to use, modify, and distribute it.
