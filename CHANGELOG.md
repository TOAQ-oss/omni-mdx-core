# Changelog — omni-mdx-core

This monorepo contains multiple packages, each with their own changelog.

---

## Packages

| Package | Version | Changelog |
|---|---|---|
| `core-parser` — Rust crate | `0.2.0` | [core-parser/CHANGELOG.md](./core-parser/CHANGELOG.md) |
| `@toaq-oss/mdx-engine` — npm | `0.1.6` | [packages/mdx-next/CHANGELOG.md](./packages/mdx-next/CHANGELOG.md) |
| `toaq-mdx` — Python | `0.1.2` | [packages/mdx-python/CHANGELOG.md](./packages/mdx-python/CHANGELOG.md) |

---

## [2026-03-13] — v0.2.0 / v0.1.6 / v0.1.2

### Complete Redesign — Native SSR + Rust Parser

This release marks the transition of the MDX engine from a client-side JS/WASM pipeline to a server-first architecture with a Rust core shared between Node.js, Python, and WASM.

**See the individual changelogs for full details.**

### Summary

- The MDX parser now runs in native Rust via napi-rs (Node.js) and PyO3 (Python)
- Next.js rendering is handled by a React Server Component — zero JS sent to the browser for content
- Python rendering supports HTML (web) and native Qt (desktop)
- WASM remains available as a fallback for browser/Edge environments