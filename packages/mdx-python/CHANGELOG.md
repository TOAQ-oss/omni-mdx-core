# Changelog — omni-mdx (Python)

All notable changes to the Python package `omni-mdx` are documented here.

## [1.1.0] — 2026-04-07

### 🚀 Core & Compatibility

* **AST Structure Alignment:** Synchronized the Python package with the new `v1.1.0` Rust engine AST format. Mathematical formulas (`InlineMath`, `BlockMath`) are now extracted from the `data-math` attribute rather than the generic `content` field, ensuring perfect parity with the JavaScript/TypeScript ecosystem.

### 🐛 Bug Fixes

* **HTML Renderer (`renderer.py`):** Fixed an issue where `InlineMath` and `BlockMath` nodes would render as empty HTML tags. The `HtmlRenderer` now properly fetches the formula via `attr_text("data-math")` to populate the HTML data attributes required for KaTeX client-side hydration.
* **Qt Native Renderer (`qt_renderer.py`):** Restored native desktop math rendering. Both the Unicode fallback and the Matplotlib `QPixmap` generation (for block formulas) have been updated to read from the new `data-math` attribute, preventing empty widget rendering in PyQt5 applications.

---

## [0.1.14] — 2026-03-27

### Fix — `parse()` now returns `MdxAst` instead of a plain list

`omni_mdx.parse()` was previously unwrapping the `MdxAst` object returned by the Rust binary and returning a plain `list[MdxNode]` directly. This broke `ast.length`, `ast.nodes`, and any future methods on the root object.

#### Modified files

- **`parser.py`** — `MDXParser.parse()` now returns the `PyMdxAst` object directly without unwrapping
- **`engine.py`** — the `.nodes` unwrap is moved into `parse_to_ast()`, where it is semantically correct
- **`__init__.py`** — Quick Start docstring updated (`nodes = parse(...)` → `ast = parse(...)`)

#### Migration
```python
# BEFORE (0.1.2 → 0.1.13)
nodes = omni_mdx.parse("# Hello")
for node in nodes:
    print(node.node_type)

# AFTER (0.1.14+)
ast = omni_mdx.parse("# Hello")
print(ast.length)
for node in ast.nodes:
    print(node.node_type)
```

#### `MdxAst` API

| Property / Method | Type | Description |
|---|---|---|
| `ast.length` | `int` | Number of root nodes |
| `ast.nodes` | `list[MdxNode]` | List of root nodes |

---

## [0.1.2] — 2026-03-13

### Initial public release

First stable release of the Python package. `omni-mdx` exposes the Rust MDX parser via PyO3/maturin and provides a complete rendering pipeline — HTML for the web, native Qt for the desktop.

---

### Architecture
```
omni_mdx/
├── __init__.py          ← Public API + shared parse() instance
├── core_interface.py    ← Rust binary loader (.pyd / .so)
├── parser.py            ← MDXParser — Rust core wrapper
├── engine.py            ← OmniMDX — parse + render orchestration
├── ast.py               ← AstNode, AttrValue, parse_ast()
├── renderer.py          ← HtmlRenderer — HTML rendering with data-math
├── qt_renderer.py       ← QtRenderer — native Qt desktop rendering
├── math_render.py       ← latex_to_unicode + latex_to_pixmap (matplotlib)
└── exceptions.py        ← OmniMDXError, MDXSyntaxError, MDXRenderError
```

---

### Features

#### Functional API (quick usage)
```python
import omni_mdx

ast  = omni_mdx.parse("# Hello\n\n$E = mc^2$")
html = omni_mdx.render_html(ast.nodes)
```

#### `MDXParser` — Rust parser via PyO3

Direct wrapper around the Rust binary. The `.pyd` (Windows) or `.so` (Linux/macOS) is bundled inside the package — no separate installation required.
```python
from omni_mdx import MDXParser

parser = MDXParser()
ast    = parser.parse("# Title\n\n**bold**")
```

#### `OmniMDX` — full engine

Parse + render pipeline orchestration with custom component support.
```python
from omni_mdx import OmniMDX

engine = OmniMDX(components={"Note": my_note_fn})
ast    = engine.parse_to_ast(mdx_text)
result = engine.render(mdx_text)
```

#### `HtmlRenderer` — HTML rendering

Produces semantic HTML compatible with KaTeX. Formulas are rendered as `data-math` attributes for client-side hydration.

- `InlineMath` → `<span class="math-inline" data-math="...">`
- `BlockMath` → `<div class="math-block" data-math="...">`

#### `QtRenderer` — native desktop rendering

Zero-HTML rendering for PyQt5 applications. Each AST node is converted into a native Qt widget.
```python
from omni_mdx.qt_renderer import QtRenderer

ast      = omni_mdx.parse(mdx_text)
renderer = QtRenderer()
widget   = renderer.render(ast.nodes, parent=my_window)
```

Custom component registry API:
```python
renderer = QtRenderer(components={"Note": my_fn}, math_font_size=14)
renderer.register("Note", my_fn).register("Alert", my_fn2)  # chainable
renderer.unregister("Note")   # reverts to built-in
renderer.registered()         # dict of user-registered components
renderer.builtin_components   # frozenset({"Note", "Details"})
```

Custom component signature:
```python
def my_component(node: AstNode, renderer: QtRenderer) -> QWidget:
    title = node.attr_text("title")
    # renderer._render_mixed_children(node.children) for mixed content
```

#### `math_render` — math rendering

- `latex_to_unicode(latex)` — symbolic conversion (Greek, operators, superscripts, subscripts)
- `latex_to_pixmap(latex, font_size, dpi, bg, fg) → QPixmap | None` — renders via matplotlib, returns `None` silently on error

#### Exceptions
```python
from omni_mdx import OmniMDXError, MDXSyntaxError, MDXRenderError

try:
    ast = omni_mdx.parse(mdx_text)
except MDXSyntaxError as e:
    print(f"Syntax error: {e}")
except MDXRenderError as e:
    print(f"Render error: {e}")
```

---

### AST types
```python
from omni_mdx import AstNode, AttrValue

node.node_type          # str — "h1", "p", "Note", "InlineMath", ...
node.content            # Optional[str] — raw text content
node.children           # List[AstNode]
node.attributes         # Dict[str, AttrValue]
node.attr_text("title") # str | None — shorthand for text attributes
```

---

### Installation
```bash
pip install omni-mdx

# From the monorepo (development)
cd core-parser
maturin develop --release --features python
```

---

### Dependencies

| Package | Role | Optional |
|---|---|---|
| PyO3 (Rust) | Python ↔ Rust bindings | No |
| PyQt5 | Native desktop rendering | Yes |
| matplotlib | Math pixmap rendering | Yes |