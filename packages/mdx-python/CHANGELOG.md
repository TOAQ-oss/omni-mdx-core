# Changelog — toaq-mdx (Python)

All notable changes to the Python package `toaq-mdx` are documented here.

---

## [0.1.2] — 2026-03-13

### Version initiale publique

Première version stable du package Python. `toaq-mdx` expose le parser MDX Rust via PyO3/maturin et fournit un pipeline complet de rendu — HTML pour le web, Qt natif pour le desktop.

---

### Architecture

```
toaq_mdx/
├── __init__.py          ← API publique + instance partagée parse()
├── core_interface.py    ← Chargement du binaire Rust (.pyd / .so)
├── parser.py            ← MDXParser — wrapping du core Rust
├── engine.py            ← OmniMDX — orchestration parse + render
├── ast.py               ← AstNode, AttrValue, parse_ast()
├── renderer.py          ← HtmlRenderer — rendu HTML avec data-math
├── qt_renderer.py       ← QtRenderer — rendu Qt natif (desktop)
├── math_render.py       ← latex_to_unicode + latex_to_pixmap (matplotlib)
└── exceptions.py        ← OmniMDXError, MDXSyntaxError, MDXRenderError
```

---

### Fonctionnalités

#### API fonctionnelle (usage rapide)

```python
import toaq_mdx

nodes = toaq_mdx.parse("# Hello\n\n$E = mc^2$")
html  = toaq_mdx.render_html(nodes)
```

#### `MDXParser` — parser Rust via PyO3

Wrapping direct du binaire Rust. Le `.pyd` (Windows) ou `.so` (Linux/macOS) est embarqué dans le package — aucune installation séparée.

```python
from toaq_mdx import MDXParser

parser = MDXParser()
nodes  = parser.parse("# Titre\n\n**gras**")
```

#### `OmniMDX` — moteur complet

Orchestration du pipeline parse + render avec support de composants custom.

```python
from toaq_mdx import OmniMDX

engine = OmniMDX(components={"Note": my_note_fn})
ast    = engine.parse_to_ast(mdx_text)
result = engine.render(mdx_text)
```

#### `HtmlRenderer` — rendu HTML

Produit du HTML sémantique compatible KaTeX. Les formules sont rendues comme attributs `data-math` pour hydratation côté client.

- `InlineMath` → `<span class="math-inline" data-math="...">`
- `BlockMath` → `<div class="math-block" data-math="...">`

#### `QtRenderer` — rendu natif desktop

Rendu zéro-HTML pour applications PyQt5. Chaque nœud AST est converti en widget Qt.

```python
from toaq_mdx import OmniMDX

engine = OmniMDX()
widget = engine.render_qt(nodes, parent=my_window)
```

API de registre des composants custom :

```python
renderer = QtRenderer(components={"Note": my_fn}, math_font_size=14)
renderer.register("Note", my_fn).register("Alert", my_fn2)  # chainable
renderer.unregister("Note")   # revient au composant built-in
renderer.registered()         # dict des composants utilisateur
renderer.builtin_components   # frozenset({"Note", "Details"})
```

Signature d'un composant custom :

```python
def my_component(node: AstNode, renderer: QtRenderer) -> QWidget:
    title = node.attr_text("title")
    # renderer._render_mixed_children(node.children) pour contenu mixte
```

#### `math_render` — rendu mathématique

- `latex_to_unicode(latex)` — conversion symbolique (grec, opérateurs, exposants, indices)
- `latex_to_pixmap(latex, font_size, dpi, bg, fg) → QPixmap | None` — rendu via matplotlib, retourne `None` silencieusement en cas d'erreur

#### Exceptions

```python
from toaq_mdx import OmniMDXError, MDXSyntaxError, MDXRenderError

try:
    nodes = toaq_mdx.parse(mdx_text)
except MDXSyntaxError as e:
    print(f"Erreur de syntaxe : {e}")
except MDXRenderError as e:
    print(f"Erreur de rendu : {e}")
```

---

### Types AST

```python
from toaq_mdx import AstNode, AttrValue

node.node_type          # str — "h1", "p", "Note", "InlineMath", ...
node.content            # Optional[str] — contenu texte brut
node.children           # List[AstNode]
node.attributes         # Dict[str, AttrValue]
node.attr_text("title") # str | None — raccourci pour les attrs text
```

---

### Installation

```bash
pip install toaq-mdx

# Depuis le monorepo (développement)
cd core-parser
maturin develop --release --features python
```

---

### Dépendances

| Package | Rôle | Optionnel |
|---|---|---|
| PyO3 (Rust) | Bindings Python ↔ Rust | Non |
| PyQt5 | Rendu desktop natif | Oui |
| matplotlib | Rendu math en pixmap | Oui |