# omni-mdx

**A blazingly fast, headless MDX engine for Python, powered by a Rust core.**

`omni-mdx` provides a bridge between the high-performance `pulldown-cmark` Rust parser and native Python applications. It parses MDX (Markdown + JSX) into a deeply manipulable Abstract Syntax Tree (AST) and offers zero-dependency native rendering solutions for both the Web (HTML/KaTeX) and Desktop (PyQt5/Matplotlib).



## 🚀 Features

- **Blazing Fast**: The core parsing is handled by a pre-compiled Rust binary.
- **Headless AST**: Manipulate Markdown and JSX tags as pure Python objects (`AstNode`).
- **Zero-HTML Desktop Rendering**: Render rich text, complex layouts, and math equations natively in PyQt5 without relying on heavy WebEngine components.
- **Universal Math Support**: 
  - Generates `data-math` attributes for KaTeX on the web.
  - Generates native `QPixmap` images using Matplotlib for desktop apps.
- **Fat Wheel Distribution**: The Rust binary is bundled directly into the Python package. No Rust toolchain is required for end-users.

## 📦 Installation

```bash
pip install omni-mdx
```

## 🛠️ Quick Start
### 1. Parsing MDX to AST
The core feature of omni-mdx is transforming text into a structured, easily searchable AST.

```python
import omni_mdx

mdx_content = """
# Physics 101
The kinetic energy is defined as:
$$E_k = \\frac{1}{2}mv^2$$

<Note type="warning">Check your units!</Note>
"""

# Parse the text into a list of AstNode objects
ast = omni_mdx.parse(mdx_content)

# Easily search the AST
math_blocks = [n for n in ast.nodes if n.node_type == "BlockMath"]
print(math_blocks[0].content) # Output: E_k = \frac{1}{2}mv^2
```

### 2. Web Rendering (HTML)
Generate clean, highly customizable HTML, perfectly suited for modern web frameworks like Next.js or FastAPI.

```python
from omni_mdx import HtmlRenderer, parse

ast = parse("<Speaker name='Leon'>Welcome to the show.</Speaker>")

# Register custom rendering logic for JSX components
def render_speaker(node, ctx):
    name = node.attr_text("name")
    return f'<div class="speaker-tag">{name}</div><p>{node.text_content()}</p>'

renderer = HtmlRenderer(components={"Speaker": render_speaker})
html_output = renderer.render(ast.nodes)
```

### 3. Native Desktop Rendering (PyQt5)
Render MDX content directly into native Qt Widgets. Math equations are seamlessly converted to high-quality images via Matplotlib.

```python
from omni_mdx.qt_renderer import QtRenderer

ast = parse("# Hello\\nNative rendering without WebViews!")
renderer = QtRenderer()
widget = renderer.render(ast.nodes, parent=window)
```

## 🧠 Advanced AST Manipulation
Because the parser generates a typed AstNode tree, it is an ideal tool for large-scale text analysis, data extraction, or automated moderation.

For instance, when processing researcher submissions or generating structured vocal datasets for distinct podcast series, you can programmatically extract specific nodes while ignoring the rest of the document formatting:

```python
from omni_mdx import parse

script = """
# Episode 4: Quantum Mechanics

<Speaker name="Dr. Aris" voiceId="v2">
We must look closer at the probability wave.
</Speaker>

<Speaker name="Leon" voiceId="v1">
Are you certain?
</Speaker>
"""

ast = parse(script)

# Extract dialogue for Text-To-Speech (TTS) dataset generation
dataset_entries = []
for node in ast.nodes:
    if node.node_type == "Speaker":
        dataset_entries.append({
            "character": node.attr_text("name"),
            "voice_profile": node.attr_text("voiceId"),
            "text": node.text_content().strip()
        })

import json
print(json.dumps(dataset_entries, indent=2))
```

## 🏗️ Architecture
* `parser.py`: High-level wrapper calling the Rust _core.pyd binary.

* `ast.py`: Python dataclasses representing the parsed nodes and attributes.

* `renderer.py`: Web-ready HTML generator.

* `qt_renderer.py` / `engine.py`: Native PyQt5 widget generator.

* `math_render.py`: Utilites for converting LaTeX strings to Unicode or QPixmap.