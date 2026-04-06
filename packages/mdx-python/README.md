# omni-mdx

**A blazingly fast, headless MDX engine for Python, powered by a native Rust core.**

`omni-mdx` provides a high-performance bridge between the `pulldown-cmark` Rust parser and native Python applications. It parses MDX (Markdown + JSX) into a deeply manipulable Abstract Syntax Tree (AST) and offers zero-dependency native rendering solutions for both the Web (HTML/KaTeX) and Desktop (PyQt5/Matplotlib).


## ⚡ Key Features

* **🚀 Blazing Fast:** Parsing is handled by a pre-compiled Rust binary. Experience performance up to 10x faster than pure Python parsers.
* **🧠 Headless AST:** Manipulate Markdown and JSX tags as pure Python objects (`AstNode`). Perfect for data extraction and content analysis.
* **🖼️ Zero-HTML Desktop Rendering:** Render rich text, complex layouts, and math equations natively in PyQt5 without the overhead of heavy WebEngine/Chromium components.
* **📐 Universal Math Support:**
    * **Web:** Generates `data-math` attributes compatible with KaTeX.
    * **Desktop:** Generates high-quality native images via **Matplotlib** with automatic Unicode fallback.
* **📦 Fat Wheel Distribution:** The Rust binary is bundled directly into the Python package. **No Rust toolchain required** for end-users.

## 📦 Installation

```bash
pip install omni-mdx

# Optional: Required for high-quality Desktop math rendering
pip install matplotlib PyQt5
```

## 🛠️ Quick Start
### 1. Parsing MDX to AST
The core strength of `omni-mdx` is transforming raw text into a structured, searchable tree.

```python
import omni_mdx

mdx_content = r"""
# Physics 101
The kinetic energy is defined as:
$$\zeta(s) = \sum_{n=1}^\infty \frac{1}{n^s}$$

<Note type="warning">Check your units!</Note>
"""

# Parse the text into a list of AstNode objects
nodes = omni_mdx.parse(mdx_content)

# Search the AST for specific elements
math_blocks = [n for n in nodes if n.node_type == "BlockMath"]
if math_blocks:
    print(f"Formula found: {math_blocks[0].content}") 
    # Output: \zeta(s) = \sum_{n=1}^\infty \frac{1}{n^s}
```

### 2. Web Rendering (HTML)
Generate clean, standards-compliant HTML for FastAPI, Flask, or static site generators.

```python
from omni_mdx import render_html, parse

nodes = parse("<Speaker name='Leon'>Welcome to the show.</Speaker>")

# Register custom rendering logic for JSX components
def render_speaker(node, ctx):
    name = node.attr_text("name")
    return f'<div class="speaker-tag"><b>{name}:</b> {node.text_content()}</div>'

html_output = render_html(nodes, components={"Speaker": render_speaker})
```

### 3. Native Desktop Rendering (PyQt5)
Render MDX content directly into native Qt Widgets. No browser engine needed.

```python
from PyQt5.QtWidgets import QScrollArea
from omni_mdx.qt_renderer import QtRenderer

# 1. Parse content
nodes = omni_mdx.parse("# Hello Native!")

# 2. Render to Widget
renderer = QtRenderer()
content_widget = renderer.render(nodes)

# 3. Add to your UI (using a ScrollArea is recommended)
scroll = QScrollArea()
scroll.setWidget(content_widget)
scroll.setWidgetResizable(True)
```

## 🧠 Advanced AST Manipulation
Because `omni-mdx` generates a typed `AstNode` tree, it is an ideal tool for large-scale text analysis, TTS (Text-To-Speech) dataset generation, or automated content moderation.

```python
from omni_mdx import parse

script = """
<Speaker name="Dr. Aris" voiceId="v2">
We must look closer at the probability wave.
</Speaker>

<Speaker name="Leon" voiceId="v1">
Are you certain?
</Speaker>
"""

nodes = parse(script)

# Extract dialogue for dataset generation
dataset = []
for node in nodes:
    if node.node_type == "Speaker":
        dataset.append({
            "character": node.attr_text("name"),
            "voice_profile": node.attr_text("voiceId"),
            "text": node.text_content().strip()
        })

print(dataset[0]["text"]) # "We must look closer at the probability wave."
```

## 🏗️ Architecture
|**Module**|**Description**|
|:---|:---|
|`core_interface`|Bridge to the native Rust `_core` binary.|
|`renderer`|High-performance HTML generator.|
|`qt_renderer`|Native PyQt5 layout engine (uses a custom `FlowLayout`).|
|`math_render`|LaTeX logic: Unicode mapping & Matplotlib integration.|

## 🤝 Contributing
This package is part of the **TOAQ** open-source ecosystem.
* **Core Engine (Rust):** [TOAQ-oss/omni-mdx-core](https://github.com/TOAQ-oss/omni-mdx-core)
* **Bug Tracker:** [GitHub Issues](https://github.com/TOAQ-oss/omni-mdx-core/security/advisories)