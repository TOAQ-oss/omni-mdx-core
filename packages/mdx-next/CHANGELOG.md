# Changelog

All notable changes to `@toaq-oss/omni-mdx` are documented here.

---

## [0.1.8] — 2026-03-13

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

## [0.1.5] and earlier

Client-side JS/WASM pipeline. Parsed using `mdx_parser.wasm`, rendered using `MDXViewer`. See the Git commits for a detailed history.