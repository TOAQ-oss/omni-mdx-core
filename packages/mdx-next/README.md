# @toaq-oss/omni-mdx

**The high-performance MDX engine.** A unified React & Next.js rendering layer powered by a dual Rust backend (Native Node.js + WebAssembly).

[![GitHub](https://img.shields.io/badge/GitHub-TOAQ--oss-181717?logo=github)](https://github.com/toaq-oss)
[![Documentation](https://img.shields.io/badge/Docs-omni--core.org-blue)](https://omni-core.org/mdx)

---

## ⚡ Why Omni-MDX?

Traditional MDX pipelines can be slow and often require heavy client-side hydration. `omni-mdx` offloads the heavy lifting to Rust while providing a **seamless bridge to the JS ecosystem**.

* 🚀 **Extreme Performance:** Parsing is done in native Rust (Server) or WASM (Client), up to 10x faster than pure JS alternatives.

* 🔌 **Unified/Rehype Bridge:** Native Rust core with full support for standard **Rehype plugins** (Highlighting, Slugs, Autolink, etc.).

* 📐 **Built-in Features:** GFM Tables, KaTeX math, and JSX components are handled natively—no extra configuration required.

* ⚛️ ***RSC Optimized:*** Built for Next.js App Router and Server Components with zero-hydration math rendering.
* 🧵 **Non-Blocking:** Offloads parsing from the Node.js main thread, keeping your server responsive.

---

## 🚀 Usage
### 1. Server-Side Rendering (RSC) with Plugins
Recommended for documentation, blogs, and research papers.

*Plugins are optional. Omni-Core lets you choose whether to use plugins or not !*

> Full example available here :
> * Basic setup: [TOAQ-oss/omni-core-sandox](https://github.com/TOAQ-oss/omni-mdx-sandbox/tree/main/next/basic-setup)
> * Advanced rendering : [TOAQ-oss/omni-core-sandox](https://github.com/TOAQ-oss/omni-mdx-sandbox/tree/main/next/advanced-rendering)

```tsx
import { parseMdx, MDXServerRenderer } from "@toaq-oss/omni-mdx/server";
import rehypeHighlight from "rehype-highlight";
import { MyComponent } from "@/components/mdx";

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

###  2. Live Client Editor (WASM)
Perfect for real-time previews or CMS interfaces.

> Full example available here : [TOAQ-oss/omni-core-sandox](https://github.com/TOAQ-oss/omni-mdx-sandbox/tree/main/next/client-rendering)

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

## 📦 Installation

```bash
npm install @toaq-oss/omni-mdx
# Required for math styles
npm install katex
```

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

## 🧩 Features
### 🔌 Extensibility (Rehype Support)
Omni-MDX uses a unique "Bridge" architecture. It parses Markdown to a high-performance AST in Rust, then optionally passes it through the Unified/Rehype pipeline in JS for transformations.
* **Support:** `rehype-slug`, `rehype-highlight`, `rehype-autolink-headings`, etc.
* **Performance:** Plugins only run on the final tree, keeping the heavy parsing phase in Rust.

### 📐 Native Professional Math
Math is handled via KaTeX. Include the CSS in your `layout.tsx`:
`import "katex/dist/katex.min.css";`
* **Inline:** `$E=mc^2$`
* **Block:** `$$\zeta(s) = \sum_{1}^{\infty} n^{-s}$$`

### 🎨 Hybrid Components
Mix standard HTML tags with custom React components. Omni-MDX preserves props and nested children perfectly between the Rust parser and your React tree.

---
## 📖 Documentation & Support

Full guides and API references: **[omni-core.org/mdx](https://omni-core.org/mdx)**

|Environment|Backend|Entry Point|
|:---|:---|:---|
|**Server** (Node.js)|Native Addon (`.node`)|`@toaq-oss/omni-mdx/server`|
|**Client** (Browser)|WebAssembly (`.wasm`)|`@toaq-oss/omni-mdx/client`|
|**Edge** (Vercel)|Native Addon (`.wasm`)|`@toaq-oss/omni-mdx/client`|

---
## 🤝 Contributing
This package is part of the TOAQ open-source ecosystem.
* **Core Parser (Rust):** [TOAQ-oss/omni-mdx-core](https://github.com/TOAQ-oss/omni-mdx-core)
* **Reporting Issues:** Please use the GitHub issue tracker for bugs or feature requests.