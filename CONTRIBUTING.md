# Contributing to TOAQ MDX Core 🚀

First off, thank you for considering contributing to the TOAQ MDX ecosystem! It's people like you who make the open-source community such an amazing place to learn, inspire, and create.

## 🏗️ Project Architecture

This is a monorepo consisting of:
- **`core-parser/`**: The heart of the engine, written in **Rust**. It handles the heavy lifting of parsing MDX into a high-performance AST.
- **`packages/mdx-next/`**: The **TypeScript/React** wrapper. It consumes the WebAssembly output and provides the `<MDXViewer />` component for Next.js.
- **`packages/mdx-python/`**: The **Python** bindings. Powered by PyO3 and Maturin, it provides a native, zero-copy Python API for parsing MDX at blazing speeds.

*Note: The test environments and playgrounds have been moved to a dedicated repository: [TOAQ-oss/omni-mdx-sandbox](https://github.com/TOAQ-oss/omni-mdx-sandbox).*

## 🛠️ Development Setup

To get started, make sure you have **Node.js**, **Rust**, and **wasm-pack** installed.

1. **Clone and Install**:
   ```bash
   git clone https://github.com/toaq-oss/omni-mdx-core.git
   cd omni-mdx-core
   make setup
   ```
2. **The Build Loop:**

    **For Web/Next.js (WASM & Native)**
    If you modify the Rust code, you must recompile the WebAssembly bridge for the React package to see the changes:
    ```bash
    make build-web
    ```

    **For Python**
    ```bash
    make build-python
    ```

3. **Running the Sandbox:**
    ```bash
    git clone https://github.com/TOAQ-oss/omni-mdx-sandbox.git
    cd omni-mdx-sandbox/next # Or /python
    # Link your local build to test changes in real-time
    ```

## 🧪 Testing
* **Rust:** Run `cargo test` inside the `core-parser` directory.
* **React:** Ensure the sandbox renders without hydration errors (especially important for our WASM integration).
* **Python:** Run your tests (e.g, `pytest`) inside the `packages/mdx-python` directory.

## 📬 Pull Request Process
1. Create a feature branch: `git checkout -b feat/my-new-feature`.
2. Follow [Conventional Commits](https://www.conventionalcommits.org/) for your commit messages.
3. Ensure the build commands  (`make build-web` or `make build-python`) command passes without errors.
4. Update the documentation if you introduce new MDX components, Python APIs, or parser logic.

## ⚖️ License
By contributing, you agree that your contributions will be licensed under its **MIT License**.