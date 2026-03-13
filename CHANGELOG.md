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

### Refonte complète — SSR natif + parser Rust

Cette release marque le passage du moteur MDX d'un pipeline JS/WASM client-side vers une architecture server-first avec un core Rust partagé entre Node.js, Python et WASM.

**Voir les changelogs individuels pour le détail complet.**

### Résumé

- Le parser MDX tourne désormais en Rust natif via napi-rs (Node.js) et PyO3 (Python)
- Le rendu Next.js est assuré par un React Server Component — zéro JS envoyé au navigateur pour le contenu
- Le rendu Python supporte HTML (web) et Qt natif (desktop)
- Le WASM reste disponible comme fallback pour les environnements browser/Edge