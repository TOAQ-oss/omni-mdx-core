# Changelog

All notable changes to `@toaq-oss/omni-mdx` are documented here.

## [1.1.0] - 2026-04-07

### 🚀 Core & Architecture

* **Robust Unified/Rehype Bridge:** Completely overhauled `unifiedBridge.ts` to make the Omni-MDX AST fully resilient against aggressive sanitization from external `rehype` plugins (like `rehype-slug` or `rehype-autolink-headings`). Custom Omni metadata is now safely cloaked during the HAST conversion and perfectly restored post-processing.
* **JSON-First Server Parsing:** Updated `parse.server.ts` to prioritize JSON serialization over the binary decoder during Server Component rendering. This guarantees absolute data integrity for deeply nested attributes and custom nodes crossing the Rust/V8 FFI boundary.

### 🐛 Bug Fixes

* **Custom Attribute Preservation:** Fixed a critical data-loss issue where `rehype` plugins would strip non-standard HTML attributes. Omni-specific properties like `data-math` (used for KaTeX hydration) and nested AST props (`kind: 'ast'`) now survive the entire unified pipeline intact.
* **Component Case Sensitivity:** Resolved an issue where standard HTML pipelines forced all tags to lowercase (e.g., transforming `<InlineMath>` to `<inlinemath>` or `<CustomComponent>` to `<customcomponent>`). The exact original casing and `self_closing` states are now meticulously saved (`dataOmniTag`) and restored, ensuring perfect matching with the React `components` registry.

---

## [0.1.24] - 2026-03-27

### 🚀 Core & Architecture

* **Buffer Input Normalization:** Added strict normalization for binary inputs (`Buffer` / `Uint8Array`) prior to being processed by the native Rust parser (`napi-rs`). This safeguards the Node.js/Rust boundary against encoding mismatch errors and ensures rock-solid stability when piping raw AST streams into the `MdxBinaryDecoder`.

### 🎨 Features & Developer Experience (DX)

* **Zero-Config Default Styling (`BASIC_STYLES`):** The package now ships with a built-in, highly polished "Dark Mode" aesthetic for all standard HTML elements (`h1`-`h6`, `p`, `ul`, `ol`, `blockquote`, `table`, etc.). This enables a beautiful out-of-the-box documentation experience without requiring users to write custom CSS or configure Tailwind Typography.
* **Smart Component Merging:** Both `MDXClientRenderer` and `MDXServerRenderer` have been updated to automatically merge the new `BASIC_STYLES` with user-provided `components`. User-defined components strictly take priority, allowing for effortless overrides while keeping the defaults for unspecified tags.

### 💅 Styling & UI Polish

* **Context-Aware Inline Code:** The default `<code>` component now distinguishes between standalone inline code (styled as a highlighted pill) and code nested inside `<pre>` blocks, preventing style collisions with external syntax highlighters (like Prism or Shiki).
* **Responsive Media & Links:** Added production-ready default styles for `<img>` (responsive scaling, rounded corners, subtle borders) and `<a>` (underlined with hover transitions) to prevent layout shifts and improve accessibility in markdown content.
* **Hydration Consistency:** Guaranteed 1:1 visual parity between Server-Side Rendering (SSR) and Client-Side Rendering (CSR) by injecting the shared styling logic directly into both renderers, eliminating the Flash of Unstyled Content (FOUC).

## [0.1.4] — 2026-03-18

### 🚀 Performance & Architecture

* **Zero-Copy Binary Protocol Integration:** Added support for decoding highly optimized binary AST buffers (`Uint8Array`) sent directly from the Rust core. The new `MdxBinaryDecoder` reconstructs the AST while bypassing V8's JSON parsing bottleneck entirely.
* **Robust N-API Module Loading:** Overhauled the OS-specific native module resolution in `parse.server.ts`. The loader now correctly maps N-API's default `camelCase` exports (`parseToBinary`, `parseToJson`) while maintaining `snake_case` fallbacks for local monorepo development.

### 🐛 Bug Fixes

* **React Server Components (RSC) Hydration:** Fixed a critical rendering issue where Next.js Server Components would silently strip N-API C++ class getters (such as `content` and `node_type`). The data flow now strictly enforces JSON or Binary serialization before crossing into React, guaranteeing pure JavaScript object hydration.
* **Math Node Content Preservation:** Ensured that `InlineMath` and `BlockMath` nodes correctly receive their inner LaTeX strings from the Rust core, preventing empty KaTeX renders on the server side.

## [0.1.3] — 2026-03-13

### 🚀 Native SSR — Complete Redesign of the Rendering Engine

This release is a major rewrite focused on server performance. The parser now runs in native Rust on the server side, and rendering is handled by a true React Server Component—no JavaScript is sent to the browser for MDX content.

---

### What's New

#### Native Rust Parser (napi-rs)
The MDX parser is now compiled into a native Node.js add-on using napi-rs. It replaces the previous JS/WASM pipeline for server environments.

- **10 to 50× faster** than the old JS implementation
- Automatic loading of the correct `.node` file based on the platform (`darwin-arm64`, `linux-x64-gnu`, `win32-x64-msvc`)
- Fallback to the local `native/` folder for development environments and monorepos

#### `MDXServerRenderer` — React Server Component
New server-side rendering component, with no `“use client”`. Compatible with Next.js App Router (RSC), SSG, ISR, and classic SSR.

```tsx
import { parseMdx, MDXServerRenderer } from "@toaq-oss/omni-mdx/server";

const ast = await parseMdx(content);
return <MDXServerRenderer ast={ast} components={COMPONENTS} />;
```

HTML is generated entirely on the server — zero MDX runtime in the client bundle.

#### `MDXClientRenderer` — client-side rendering for live editors
Retained for cases where MDX content changes in the browser (editors, real-time previews). Includes automatic KaTeX hydration via `useEffect`.

```tsx
import { MDXClientRenderer } from "@toaq-oss/omni-mdx/client";

<MDXClientRenderer ast={ast} components={COMPONENTS} katex />
```

#### New three-entry import system
| Import | Usage |
|---|---|
| `@toaq-oss/omni-mdx` | Types + `MDX_COMPONENTS` — safe everywhere |
| `@toaq-oss/omni-mdx/server` | `parseMdx`, `MDXServerRenderer` — Server Components only |
| `@toaq-oss/omni-mdx/client` | `MDXClientRenderer`, `MDXErrorBoundary` — Client Components only |

#### Built-in `MDXErrorBoundary`
Every custom component rendered by `MDXClientRenderer` is automatically isolated within an `MDXErrorBoundary`. If an error occurs in a component (null data, runtime crash), the rest of the document continues to display.

Also exported for direct use:

```tsx
import { MDXErrorBoundary } from "@toaq-oss/omni-mdx/client";

<MDXErrorBoundary componentName="Chart">
  <Chart data={maybeNull} />
</MDXErrorBoundary>
```

#### Math handling by the Rust parser
LaTeX formulas are extracted by the Rust parser before any other processing takes place — no need for `remark-math` or `rehype-katex`.

- `$E = mc^2$` → `<span class="math math-inline" data-math=“...”>`
- `$$...$$` → `<div class="math math-display" data-math=“...”>`

KaTeX populates the `data-math` attributes on the client side.

#### `parseProps` — exported utility
Utility function for parsing JSX prop values in custom components (objects, arrays, booleans, numbers from strings).

```tsx
import { parseProps } from “@toaq-oss/omni-mdx”;

const data = parseProps(props.data); // “[1,2,3]” → [1, 2, 3]
```

#### Platform-specific distribution system (optionalDependencies)
In production, native add-ons are distributed as independent npm sub-packages. npm/pnpm installs only the package corresponding to the current platform:

```
@toaq-oss/omni-mdx-darwin-arm64   ← Mac M1/M2
@toaq-oss/omni-mdx-darwin-x64     ← Mac Intel
@toaq-oss/omni-mdx-linux-x64-gnu  ← Linux / Vercel / Docker
@toaq-oss/omni-mdx-win32-x64-msvc ← Windows
```

No configuration required — detection is automatic at runtime.

---

### Major (breaking) changes

- **`MDXViewer` removed** → use `MDXServerRenderer` (server) or `MDXClientRenderer` (client)
- **`MdxConfigProvider` removed** → pass `components` directly as a prop
- **The parser no longer runs in the browser by default** → WASM is reserved for the client via `MDXClientRenderer`
- **WASM not supported in SSR** → if the native `.node` is missing on the server side, a clear error is thrown with build instructions

---

### Fixes

- Table: HTML structure normalization (`thead > tr > th`, `tbody > tr > td`)
- Code blocks: JSX content parsed inside ` ``` ` is extracted as plain text
- `import.meta.url` import incompatible with CJS → the server entry is now ESM-only
- tsup `empty-import-meta` warning removed

---

## [0.1.2] and earlier

Client-side JS/WASM pipeline. Parsed using `mdx_parser.wasm`, rendered using `MDXViewer`. See the Git commits for a detailed history.