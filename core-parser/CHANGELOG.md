# Changelog — core-parser (Rust)

All notable changes to the Rust MDX parser crate are documented here.

---

## [0.2.0] — 2026-03-13

### Version initiale publique

Première version stable du parser MDX en Rust. Ce crate constitue le cœur du moteur `omni-mdx-core` — il est consommé par le package npm `@toaq-oss/mdx-engine` via napi-rs, par le package Python `toaq-mdx` via PyO3/maturin, et peut être compilé en WASM via wasm-bindgen.

---

### Architecture

#### Pipeline de parsing

Le parsing suit un ordre strict en trois passes, conçu pour éviter les conflits entre les grammaires MDX, JSX et LaTeX :

```
input
  │
  ▼
extract_math()       ← 1re passe — extrait $...$ et $$...$$ avant tout
  │
  ▼
extract_jsx()        ← 2e passe — extrait les blocs JSX
  │
  ▼
parse_markdown()     ← 3e passe — parse le Markdown via pulldown-cmark
```

L'ordre est critique : les formules mathématiques doivent être extraites avant le lexer JSX, car `$t > 0$` contient des `>` et `<` qui seraient autrement interprétés comme des balises.

#### Système de placeholders

Les blocs extraits sont remplacés par des tokens non-imprimables pour éviter toute collision avec le contenu MDX :

| Token | Signification |
|---|---|
| `\x02JSXn\x03` | Bloc JSX numéro n |
| `\x02MATHBn\x03` | Bloc math display `$$...$$` |
| `\x02MATHIn\x03` | Bloc math inline `$...$` |

#### AST

```rust
pub enum AttrValue {
    Text(String),
    Expression(String),
    Boolean,
    Ast(Vec<AstNode>),
}

pub struct AstNode {
    pub node_type:     String,
    pub content:       Option<String>,
    pub self_closing:  bool,
    pub attributes:    HashMap<String, AttrValue>,
    pub children:      Vec<AstNode>,
}
```

---

### Features de compilation

| Feature | Backend | Usage |
|---|---|---|
| `node` | napi-rs | Addon Node.js natif (`.node`) |
| `python` | PyO3 | Extension Python (`.pyd` / `.so`) |
| `wasm` | wasm-bindgen | Module WASM (browser / Edge) |
| `ffi` | C FFI | Interop C générique |

---

### Fonctionnalités du parser

- **Markdown complet** via pulldown-cmark : headings, paragraphes, listes ordonnées et non-ordonnées, listes imbriquées, blockquotes, tables GFM, blocs de code fencés avec langue, code inline, gras, italique, strikethrough, liens, images, règles horizontales, sauts de ligne
- **JSX** : composants custom avec attributs typés (`text`, `expression`, `boolean`, `ast`), enfants mixtes (texte + inline + blocs), auto-fermeture, imbrication
- **Attributs JSX complexes** : `title="foo"` → `Text`, `left={<div>}` → `Ast`, `disabled` → `Boolean`, `count={42}` → `Expression`
- **Math inline** : `$...$` → nœud `InlineMath`
- **Math display** : `$$...$$` → nœud `BlockMath`
- **Déduplication de paragraphes** : les JSX block seuls ne sont pas wrappés dans un `<p>`
- **Dédent automatique** : le contenu indenté dans les composants JSX est normalisé

---

### Bugs corrigés durant le développement

1. **Placeholder `__JSX_N__`** — remplacé par `\x02JSXn\x03` pour éviter les collisions avec le contenu utilisateur
2. **Panic `bytes[i-1]` sur `i == 0`** — guard sur l'index zéro ajouté dans le lexer
3. **`<p>` wrappant un JSX seul** — `unwrap_solo_jsx_paragraph` réécrit pour détecter les paragraphes ne contenant qu'un seul nœud block
4. **`$t > 0$` crashant le lexer** — résolu par le réordonnancement du pipeline (math avant JSX)
5. **`init()` WASM n'est pas une fonction** — `wasm-bindgen` rendu optionnel, feature `wasm` isolée
6. **`Unclosed JSX block`** — machine d'état du lexer JSX réécrite pour gérer les balises imbriquées
7. **`Unexpected token '?'`** — `parse_children` appelait `parse_mdx` récursivement au lieu de `parse_markdown`
8. **Build wasm-pack échouant** — `wasm-bindgen` extrait en dépendance optionnelle conditionnelle
9. **Corruption UTF-8** — introduction de `utf8_char_len()` et `from_utf8` pour tous les slices de bytes
10. **`left={<div>}` rendu comme string** — `parse_html_wrapper` appelle maintenant `parse_markdown` pour les attributs `ast`
11. **Contenu indenté → `<pre>`** — fonction `dedent()` ajoutée en pré-traitement des enfants JSX
12. **`<div>` inside `<p>` (erreur d'hydratation React)** — `unwrap_solo_jsx_paragraph` réécrit une seconde fois pour couvrir les cas de BlockMath
13. **`InlineMath` hissé comme bloc** — `is_block()` exclut explicitement `InlineMath`

---

### Compilation

```bash
# Addon Node.js
napi build --platform --release --features node --no-js

# Extension Python
maturin develop --release --features python

# WASM
wasm-pack build --target bundler --features wasm
```

---

### Dépendances principales

| Crate | Rôle |
|---|---|
| `pulldown-cmark` | Parser Markdown (CommonMark + GFM) |
| `serde` + `serde_json` | Sérialisation AST → JSON |
| `napi` + `napi-derive` | Bindings Node.js (feature `node`) |
| `pyo3` | Bindings Python (feature `python`) |
| `wasm-bindgen` | Bindings WASM (feature `wasm`) |