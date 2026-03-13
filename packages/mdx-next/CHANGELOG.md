# Changelog

All notable changes to `@toaq-oss/mdx-engine` are documented here.

---

## [0.1.6] — 2026-03-13

### 🚀 SSR natif — refonte complète du moteur de rendu

Cette version est une réécriture majeure orientée performance serveur. Le parser tourne désormais en Rust natif côté serveur, et le rendu est assuré par un vrai React Server Component — aucun JavaScript n'est envoyé au navigateur pour le contenu MDX.

---

### Nouveautés

#### Parser Rust natif (napi-rs)
Le parser MDX est maintenant compilé en addon natif Node.js via napi-rs. Il remplace le pipeline JS/WASM précédent pour les environnements serveur.

- **10 à 50× plus rapide** que l'ancienne implémentation JS
- Chargement automatique du bon `.node` selon la plateforme (`darwin-arm64`, `linux-x64-gnu`, `win32-x64-msvc`)
- Fallback sur le dossier `native/` local pour les environnements de développement et monorepos

#### `MDXServerRenderer` — React Server Component
Nouveau composant de rendu côté serveur, sans aucun `"use client"`. Compatible Next.js App Router (RSC), SSG, ISR et SSR classique.

```tsx
import { parseMdx, MDXServerRenderer } from "@toaq-oss/mdx-engine/server";

const ast = await parseMdx(content);
return <MDXServerRenderer ast={ast} components={COMPONENTS} />;
```

Le HTML est généré entièrement sur le serveur — zéro runtime MDX dans le bundle client.

#### `MDXClientRenderer` — rendu client pour les éditeurs live
Conservé pour les cas où le contenu MDX change dans le navigateur (éditeurs, previews en temps réel). Inclut l'hydratation KaTeX automatique via `useEffect`.

```tsx
import { MDXClientRenderer } from "@toaq-oss/mdx-engine/client";

<MDXClientRenderer ast={ast} components={COMPONENTS} katex />
```

#### Nouveau système d'imports en trois entrées
| Import | Usage |
|---|---|
| `@toaq-oss/mdx-engine` | Types + `MDX_COMPONENTS` — safe partout |
| `@toaq-oss/mdx-engine/server` | `parseMdx`, `MDXServerRenderer` — Server Components uniquement |
| `@toaq-oss/mdx-engine/client` | `MDXClientRenderer`, `MDXErrorBoundary` — Client Components uniquement |

#### `MDXErrorBoundary` intégré
Chaque composant custom rendu par `MDXClientRenderer` est automatiquement isolé dans un `MDXErrorBoundary`. En cas d'erreur dans un composant (données nulles, crash runtime), le reste du document continue de s'afficher.

Également exporté pour usage direct :

```tsx
import { MDXErrorBoundary } from "@toaq-oss/mdx-engine/client";

<MDXErrorBoundary componentName="Chart">
  <Chart data={maybeNull} />
</MDXErrorBoundary>
```

#### Gestion des maths par le parser Rust
Les formules LaTeX sont extraites par le parser Rust avant tout autre traitement — plus besoin de `remark-math` ou `rehype-katex`.

- `$E = mc^2$` → `<span class="math math-inline" data-math="...">`
- `$$...$$` → `<div class="math math-display" data-math="...">`

KaTeX hydrate les attributs `data-math` côté client.

#### `parseProps` — utilitaire exporté
Fonction utilitaire pour parser les valeurs de props JSX dans les composants custom (objets, tableaux, booléens, nombres depuis des strings).

```tsx
import { parseProps } from "@toaq-oss/mdx-engine";

const data = parseProps(props.data); // "[1,2,3]" → [1, 2, 3]
```

#### Système de distribution par plateforme (optionalDependencies)
En production, les addons natifs sont distribués comme sous-packages npm indépendants. npm/pnpm installe uniquement le package correspondant à la plateforme courante :

```
@toaq-oss/mdx-engine-darwin-arm64   ← Mac M1/M2
@toaq-oss/mdx-engine-darwin-x64     ← Mac Intel
@toaq-oss/mdx-engine-linux-x64-gnu  ← Linux / Vercel / Docker
@toaq-oss/mdx-engine-win32-x64-msvc ← Windows
```

Aucune configuration requise — la détection est automatique au runtime.

---

### Changements majeurs (breaking)

- **`MDXViewer` supprimé** → utiliser `MDXServerRenderer` (server) ou `MDXClientRenderer` (client)
- **`MdxConfigProvider` supprimé** → passer `components` directement en prop
- **Le parser ne tourne plus dans le navigateur par défaut** → le WASM est réservé au client via `MDXClientRenderer`
- **WASM non supporté en SSR** → si le `.node` natif est absent côté serveur, une erreur claire est levée avec les instructions de build

---

### Corrections

- Table : normalisation de la structure HTML (`thead > tr > th`, `tbody > tr > td`)
- Blocs de code : le contenu JSX parsé à l'intérieur des ` ``` ` est extrait en texte brut
- Import `import.meta.url` incompatible CJS → le server entry est maintenant ESM uniquement
- Warning tsup `empty-import-meta` supprimé

---

## [0.1.5] et antérieur

Pipeline JS/WASM côté client. Parser via `mdx_parser.wasm`, rendu via `MDXViewer`. Voir les commits git pour l'historique détaillé.