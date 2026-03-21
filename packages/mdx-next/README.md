# @toaq-oss/omni-mdx

**The high-performance MDX engine for the TOAQ ecosystem.** A unified React & Next.js rendering layer powered by a dual Rust backend (Native Node.js + WebAssembly).

[![GitHub](https://img.shields.io/badge/GitHub-TOAQ--oss-181717?logo=github)](https://github.com/toaq-oss)
[![Documentation](https://img.shields.io/badge/Docs-omni--core.org-blue)](https://omni-core.org/mdx)

---

## ⚡ Why Omni-MDX?

Traditional MDX pipelines are often slow or require complex client-side hydration. `omni-mdx` solves this by moving the heavy lifting to Rust, providing a seamless bridge between raw content and React components.

- **Dual-Engine Architecture:** Uses native `.node` binaries on the server for maximum speed and **WASM** in the browser for instant live previews.
- **Zero-Hydration Math:** LaTeX is pre-parsed by Rust and rendered via KaTeX with zero layout shift.
- **RSC Optimized:** Built from the ground up for Next.js App Router and Server Components.
- **Non-Blocking:** Offloads parsing from the Node.js main thread to Rust, keeping your server responsive even under heavy load.

---

## 📖 Documentation

For full guides, API references, and advanced configuration, visit:
👉 **[omni-core.org/mdx](https://omni-core.org/mdx)**

---

## 📦 Installation

```bash
npm install @toaq-oss/omni-mdx
# KaTeX is required for math rendering
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

## 🚀 Usage

### 1. Server-Side Rendering (RSC)
Recommended for documentation, blogs, and research papers.

```tsx
import { parseMdx, MDXServerRenderer } from "@toaq-oss/omni-mdx/server";
import { MyComponent } from "@/components/mdx";

export default async function Page({ content }) {
  // Parsed via Native Rust Addon (.node)
  const ast = await parseMdx(content);

  return (
    <MDXServerRenderer 
      ast={ast} 
      components={{ MyComponent }} 
    />
  );
}
```

###  2. Live Client Editor (WASM)
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
|Environment|Backend|Entry Point|
|:---|:---|:---|
|**Server** (Node.js)|Native Addon (`.node`)|`@toaq-oss/omni-mdx/server`|
|**Client** (Browser)|WebAssembly (`.wasm`)|`@toaq-oss/omni-mdx/client`|
|**Edge** (Vercel)|Native Addon (`.wasm`)|`@toaq-oss/omni-mdx/client`|

---

## 🧩 Features
### 📐 Professional Math
Math is handled via KaTeX. Simply include the CSS in your `layout.tsx`:
```tsx
import "katex/dist/katex.min.css";
```
* **Inline:** $E=mc^2$
* **Block:** $$\int f(x)dx$$

### 🎨 Custom Components
Register any React component (Server or Client) to handle custom tags:
```tsx
const components = {
  Callout: ({ children }) => <div className="p-4 bg-blue-50">{children}</div>,
  VocalDataset: dynamic(() => import('./VocalDataset'), { ssr: false })
};
```

---
## 🤝 Contributing
This package is part of the TOAQ open-source ecosystem.
* **Core Parser (Rust):** [TOAQ-oss/omni-mdx-core](https://github.com/TOAQ-oss/omni-mdx-core)
* **Reporting Issues:** Please use the GitHub issue tracker for bugs or feature requests.