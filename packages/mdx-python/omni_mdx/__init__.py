"""
omni_mdx — High-performance Python + Rust MDX Engine.

The native Rust binary (omni_mdx_core) is embedded directly within this 
package. A single 'pip install' is all you need—no separate Rust installation 
or toolchain required on the user's side.

Quick Start
-----------
    import omni_mdx

    # Parse MDX text into a list of AST nodes
    nodes = omni_mdx.parse("# Hello\\n\\n$E = mc^2$")

    # Option 1: Render to HTML (for Web environments)
    html = omni_mdx.render_html(nodes)

    # Option 2: Render to Native Qt Widgets (for Desktop apps, 0 HTML)
    from omni_mdx.qt_renderer import QtRenderer
    renderer = QtRenderer()
    widget = renderer.render(nodes, parent=my_parent)
"""

from .parser import MDXParser
from .engine import OmniMDX
from .renderer import HtmlRenderer, render_html
from .exceptions import OmniMDXError, MDXSyntaxError, MDXRenderError

# Shared singleton instance for convenient functional-style parsing
_parser = MDXParser()
parse = _parser.parse

__all__ = [
    # Core Functions
    "parse",
    "render_html",
    # Primary Classes
    "MDXParser",
    "OmniMDX",
    "HtmlRenderer",
    # Exception Hierarchy
    "OmniMDXError",
    "MDXSyntaxError",
    "MDXRenderError",
]

# Synchronized with the official stable v1 release
__version__ = "0.1.18"