# @toaq-oss/omni-mdx

[![Build][build-badge]][build]
[![Coverage][coverage-badge]][coverage]
[![Downloads][downloads-badge]][downloads]
[![Size][size-badge]][size]
[![Chat][chat-badge]][chat]

**The high-performance MDX engine.** A unified React & Next.js rendering layer powered by a dual Rust backend (Native Node.js + WebAssembly).

---

## Contents

* [What is this?](#what-is-this)
* [Why Omni-MDX?](#why-omni-mdx)
* [Install](#install)
* [Use](#use)
  * [Server-Side Rendering (RSC)](#server-side-rendering-rsc)
  * [Live Client Editor (WASM)](#live-client-editor-wasm)
* [Monorepo Structure](#monorepo-structure)
* [Next.js Configuration](#nextjs-configuration)
* [Contributing](#contributing)
* [License](#license)

---

## What is this?

`omni-mdx` is an ecosystem designed to overcome the performance bottlenecks of traditional MDX pipelines. Instead of relying on heavy JavaScript regular expressions, it utilizes a core written in **Rust** to transform MDX into a structured Abstract Syntax Tree (AST) nearly instantaneously.

Whether you are building for the Web (Next.js, React), or the Server (Node.js), Omni-MDX provides a unified interface for smooth and precise rendering.

---

## Why Omni-MDX?

* 🚀 **Extreme Performance:** Parsing is offloaded to Rust, performing up to 10x faster than pure JS alternatives.
* 🔌 **Unified/Rehype Bridge:** Native Rust core with full support for standard **Rehype plugins** (Highlighting, Slugs, Autolink, etc.).
* 📐 **Built-in Features:** GFM Tables, KaTeX math, and JSX components handled natively—no extra configuration required.
* ⚛️ **RSC Optimized:** Built for Next.js App Router and Server Components with zero-hydration math rendering.
* 🧵 **Non-Blocking:** Offloads parsing from the Node.js main thread, keeping your server responsive.

---

## Install

This package is [ESM only](https://gist.github.com/sindresorhus/a39789f98801d908bbc7ff3ecc99d99c). In Node.js, install with [npm]:

```bash
npm install @toaq-oss/omni-mdx
# Required for math styles
npm install katex
```

---

## Use

### Server-Side Rendering (RSC)

Recommended for documentation, blogs, and research papers.

```tsx
import { parseMdx, MDXServerRenderer } from "@toaq-oss/omni-mdx/server";
import rehypeHighlight from "rehype-highlight";

export default async function Page({ content }) {
  // 1. Parse via Native Rust Addon (.node)
  const ast = await parseMdx(content);

  return (
    <MDXServerRenderer 
      ast={ast} 
      components={{ MyComponent }} 
      // 2. Seamlessly use your favorite Rehype plugins
      rehypePlugins={[rehypeHighlight]}
    />
  );
}
```

### Live Client Editor (WASM)

Perfect for real-time previews or CMS interfaces.

```tsx
"use client";
import { useState } from "react";
import { parseMdx, MDXClientRenderer } from "@toaq-oss/omni-mdx/client";

export default function Editor() {
  const [ast, setAst] = useState(null);

  const handleChange = async (text) => {
    // Runs 100% locally in the browser via WebAssembly
    const result = await parseMdx(text);
    setAst(result);
  };

  return (
    <div>
      <textarea onChange={(e) => handleChange(e.target.value)} />
      <MDXClientRenderer ast={ast} />
    </div>
  );
}
```

---

## Monorepo Structure

This repository houses the various components of the ecosystem:

| Package | Language | Description |
| :--- | :---: | :--- |
| [**`core-parser`**](./packages/core-parser) | ![Rust] | The high-performance parsing core. |
| [**`mdx-next`**](./packages/mdx-next) | ![TypeScript] | React/Next.js layer (Native + WASM). |

---

## Next.js Configuration

To enable the WebAssembly engine for client-side rendering, update your `next.config.js`:

```typescript
/** @type {import('next').NextConfig} */
const nextConfig = {
  webpack: (config) => {
    config.experiments = { ...config.experiments, asyncWebAssembly: true };
    config.module.rules.push({
      test: /\.wasm$/,
      type: "asset/resource",
    });
    return config;
  },
};

export default nextConfig;
```

---

## Contributing

We enthusiastically welcome contributions! Omni-MDX is a sophisticated project blending Rust, C++, TypeScript, and Python.

1. **Clone the repository:**
   ```bash
   git clone https://github.com/TOAQ-oss/omni-mdx-core.git
   ```
2. **Check the contribution guide:** [CONTRIBUTING.md](./CONTRIBUTING.md) to learn how to set up your local development environment.

---

## License

[MIT][license] © [TOAQ-oss][author-url]

---

[build-badge]: https://github.com/TOAQ-oss/omni-mdx-core/actions/workflows/publish-next.yml/badge.svg
[build]: https://github.com/TOAQ-oss/omni-mdx-core/actions

[coverage-badge]: https://img.shields.io/codecov/c/github/TOAQ-oss/omni-mdx-core/main.svg?flag=npm&logo=codecov
[coverage]: https://codecov.io/github/TOAQ-oss/omni-mdx-core

[downloads-badge]: https://img.shields.io/npm/dm/@toaq-oss/omni-mdx.svg?style=flat-square&color=blue
[downloads]: https://www.npmjs.com/package/@toaq-oss/omni-mdx

[size-badge]: https://img.shields.io/bundlejs/size/@toaq-oss/omni-mdx?color=brightgreen
[size]: https://bundlejs.com/?q=@toaq-oss/omni-mdx

[chat-badge]: https://img.shields.io/badge/chat-discussions-success.svg?logo=github
[chat]: https://github.com/TOAQ-oss/omni-mdx-core/discussions

[license]: https://github.com/TOAQ-oss/omni-mdx-core/blob/main/LICENSE
[author-url]: https://github.com/toaq-oss

[Rust]: https://img.shields.io/badge/rust-000000?logo=rust
[TypeScript]: https://img.shields.io/badge/typescript-3178C6?logo=typescript