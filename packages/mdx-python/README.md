# omni-mdx

[![Build][build-badge]][build]
[![Coverage][coverage-badge]][coverage]
![Downloads][downloads-badge]
[![Version][version-badge]][version]
[![License][license-badge]][license]

**The high-performance MDX engine for Python, powered by a native Rust core.**

```text
pip install omni-mdx
```

## Contents

* [What is this?](#what-is-this)
* [When should I use this?](#when-should-i-use-this)
* [Install](#install)
* [Use](#use)
  * [1. Parsing to AST](#1-parsing-to-ast)
  * [2. Web Rendering (HTML)](#2-web-rendering-html)
  * [3. Native Desktop Rendering (PyQt5)](#3-native-desktop-rendering-pyqt5)
* [API](#api)
* [Architecture](#architecture)
* [Compatibility](#compatibility)
* [Security](#security)
* [Contribute](#contribute)
* [License](#license)

## What is this?

```omni-mdx``` provides a high-performance bridge between the ```pulldown-cmark``` Rust parser and native Python applications. It parses MDX (Markdown + JSX) into a deeply manipulable Abstract Syntax Tree (AST) and offers zero-dependency native rendering solutions for both the Web (HTML/KaTeX) and Desktop (PyQt5/Matplotlib).

## When should I use this?

* **Extreme Performance:** You need to parse large amounts of MDX content without blocking the Python GIL, thanks to the native Rust core.
* **Data Mining & AI:** You want to extract structured data (formulas, metadata, components) from MDX for LLM ingestion or indexing.
* **Native Desktop Apps:** You are building a PyQt5 or PySide application and need to render rich Markdown and Math equations without the overhead of a heavy WebEngine/Chromium component.
* **Scientific Publishing:** You need robust, built-in support for GFM Tables and KaTeX/LaTeX math.

## Install

The package is distributed as a **Fat Wheel**, meaning the Rust binary is bundled directly. **No Rust toolchain is required** for installation.

```bash
pip install omni-mdx

# Optional: Required for high-quality Desktop math rendering
pip install matplotlib PyQt5
```

## Use

### 1. Parsing to AST
Transform raw text into a structured, searchable tree.

```python
import omni_mdx

mdx_content = r"""
# Physics 101
The kinetic energy is defined as:
$$\zeta(s) = \sum_{n=1}^\infty \frac{1}{n^s}$$

<Note type="warning">Check your units!</Note>
"""

# Parse returns an MdxAstRoot object
ast = omni_mdx.parse(mdx_content)

# Nodes are accessible via ast.nodes
for node in ast.nodes:
    if node.node_type == "BlockMath":
        print(f"Formula found: {node.attributes.get('data-math')}")
```

### 2. Web Rendering (HTML)
Generate clean, standards-compliant HTML.

```python
from omni_mdx import render_html, parse

nodes = parse("<Speaker name='Leon'>Welcome to the show.</Speaker>").nodes

# Register custom rendering logic for JSX components
def render_speaker(node, ctx):
    name = node.attributes.get("name")
    return f'<div class="speaker-tag"><b>{name}:</b> {node.text_content()}</div>'

html_output = render_html(nodes, components={"Speaker": render_speaker})
```

### 3. Native Desktop Rendering (PyQt5)
Render MDX content directly into native Qt Widgets.

```python
from PyQt5.QtWidgets import QScrollArea
from omni_mdx.qt_renderer import QtRenderer
import omni_mdx

# 1. Parse and render
nodes = omni_mdx.parse("# Hello Native!").nodes
renderer = QtRenderer()
content_widget = renderer.render(nodes)

# 2. Add to your UI
scroll = QScrollArea()
scroll.setWidget(content_widget)
scroll.setWidgetResizable(True)
```

## API

### ```omni_mdx.parse(source: str) -> MdxAstRoot```

Parses the MDX source string into an AST root.
* **Returns**: An ```MdxAstRoot``` containing a list of ```MdxNode``` objects.

### ```MdxNode``` Properties
* ```node_type```: The type of the node (e.g., "h1", "text", "BlockMath").
* ```content```: The raw content of the node.
* ```attributes```: A dictionary of JSX props or math metadata.
* ```children```: A list of child ```MdxNode``` objects.
* ```is_component```: Boolean indicating if the node is a JSX component.

## Architecture

Omni-MDX utilizes a "Bridge" architecture. The heavy parsing is performed in **Rust**, producing a high-performance tree. This tree is then wrapped in a Python layer that provides a familiar, typed API for manipulation and rendering.

## Compatibility

* **Python**: 3.11+.
* **Operating Systems**: Linux (x86_64, aarch64), macOS (Intel, Apple Silicon), Windows (x86_64).
* **GUI Frameworks**: Optimized for PyQt5 but compatible with any logic that can ingest the AST.

## Security

The Rust core provides a safety layer against common regex-based DoS attacks found in many pure-Python parsers. For HTML rendering, ensure your custom components properly sanitize any raw user input if necessary.

## Contribute

This package is part of the **TOAQ** open-source ecosystem.
Current Status:
* **Python Layer Coverage**: 88.10%.
* **Core Rust Coverage**: 81.29%.

See the [Contribution Guide](https://github.com/TOAQ-oss/omni-mdx-core/blob/main/CONTRIBUTING.md) to get started with local development and ```maturin```.

## License

[MIT](https://github.com/TOAQ-oss/omni-mdx-core/blob/main/LICENSE) © [TOAQ-oss](https://github.com/TOAQ-oss)

[build-badge]: https://github.com/TOAQ-oss/omni-mdx-core/actions/workflows/codecov-python.yml/badge.svg?branch=main
[build]: https://github.com/TOAQ-oss/omni-mdx-core/actions

[coverage-badge]: https://img.shields.io/codecov/c/github/TOAQ-oss/omni-mdx-core/main.svg?flag=python&logo=codecov
[coverage]: https://codecov.io/github/TOAQ-oss/omni-mdx-core

[downloads-badge]: https://img.shields.io/pypi/dm/omni-mdx.svg?style=flat-square&color=blue
[version-badge]: https://img.shields.io/pypi/v/omni-mdx.svg?style=flat-square&color=blue
[version]: https://pypi.org/project/omni-mdx/

[license-badge]: https://img.shields.io/pypi/l/omni-mdx.svg?style=flat-square&color=blue
[license]: https://github.com/TOAQ-oss/omni-mdx-core/blob/main/LICENSE