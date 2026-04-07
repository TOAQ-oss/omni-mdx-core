"""
omni_mdx.renderer — HTML rendering from AST MDX.
"""

from __future__ import annotations
import html as _html
from typing import Callable, Dict, List, Optional

from .exceptions import MDXRenderError

# Type hint simplifié
ComponentFn = Callable[["MdxNode", "RenderContext"], str]

class RenderContext:
    """
    Configuration and state container for the rendering pipeline.
    
    Attributes:
        components (Dict[str, ComponentFn]): Map of custom MDX component names to rendering functions.
        katex (bool): Whether to enable specific data-attributes for KaTeX math rendering.
    """
    def __init__(self, components=None, katex=True):
        self.components = components or {}
        self.katex = katex

_BUILTIN_HTML_COMPONENTS: frozenset = frozenset()

class HtmlRenderer:
    """
    Core HTML renderer for Omni-MDX AST nodes.
    
    Handles standard HTML elements, Markdown-specific constructs (links, images, code),
    Math expressions (LaTeX), and custom JSX-like components.
    """
    def __init__(self, components=None, katex=True):
        """
        Initializes the renderer with optional custom components.
        
        Args:
            components: Initial map of custom component names to functions.
            katex: If True, math nodes are rendered with KaTeX-compatible wrappers.
        """
        self._ctx = RenderContext(components=dict(components or {}), katex=katex)

    def register(self, name: str, fn) -> "HtmlRenderer":
        """Registers a custom rendering function for a specific MDX component name."""
        self._ctx.components[name] = fn
        return self

    def unregister(self, name: str) -> "HtmlRenderer":
        """Removes a custom component registration."""
        self._ctx.components.pop(name, None)
        return self

    def render(self, nodes: List) -> str:
        """
        Transforms a list of AST nodes into a single HTML string.
        
        Args:
            nodes: A list of nodes (usually obtained from `parse_mdx`).
        """
        return "".join(self._node(n) for n in nodes)

    # HTML5 void elements that do not require a closing tag
    _VOID = frozenset({"area","base","br","col","embed","hr","img",
                       "input","link","meta","param","source","track","wbr"})

    def _node(self, node) -> str:
        """Internal router that dispatches rendering based on node type."""
        t = node.node_type
        if t == "InlineMath":  return self._inline_math(node)
        if t == "BlockMath":   return self._block_math(node)
        if t == "text":        return _html.escape(node.content or "")
        if node.is_component:
            fn = self._ctx.components.get(t)
            return fn(node, self._ctx) if fn else self._component_fallback(node)
        return self._element(node)

    def _element(self, node) -> str:
        """Renders standard HTML elements, handling content and children recursion."""
        t = node.node_type
        if t == "br":   return "<br>"
        if t == "hr":   return "<hr>"
        if t == "html": return node.content or ""
        if t == "img":  return self._img(node)
        if t == "a":    return self._link(node)
        if t == "pre":  return self._code_block(node)
        attrs = self._attrs(node)
        inner = "".join(self._node(c) for c in node.children)
        if t in self._VOID or node.self_closing:
            return f"<{t}{attrs}>"
        return f"<{t}{attrs}>{inner}</{t}>"

    def _attrs(self, node) -> str:
        """
        Serializes node attributes into HTML attribute strings.
        Maps React-style 'className' to standard HTML 'class'.
        Supports boolean attributes and escaped string values.
        """
        if not node.attributes:
            return ""
        parts = []
        for name, val in node.attributes.items():
            html_name = "class" if name == "className" else name

            # Values are now native Python booleans or strings
            if isinstance(val, bool) and val:
                parts.append(f" {html_name}")
            elif isinstance(val, str):
                parts.append(f' {html_name}="{_html.escape(val)}" ')
        return "".join(parts)

    def _img(self, node) -> str:
        """Renders an <img> tag with escaped src, alt, and title."""
        src   = _html.escape(node.attr_text("src") or "")
        alt   = _html.escape(node.attr_text("alt") or "")
        title = node.attr_text("title")
        t     = f' title="{_html.escape(title)}"' if title else ""
        return f'<img src="{src}" alt="{alt}"{t}>'

    def _link(self, node) -> str:
        """Renders an <a> tag with escaped href and recursive inner content."""
        href  = _html.escape(node.attr_text("href") or "#")
        title = node.attr_text("title")
        t     = f' title="{_html.escape(title)}"' if title else ""
        inner = "".join(self._node(c) for c in node.children)
        return f'<a href="{href}"{t}>{inner}</a>'

    def _code_block(self, node) -> str:
        """Renders a code block using <pre> and <code> with optional language class."""
        lang = node.attr_text("lang") or ""
        la   = f' class="language-{_html.escape(lang)}"' if lang else ""
        code = _html.escape(node.content or node.text_content())
        return f"<pre><code{la}>{code}</code></pre>"

    def _inline_math(self, node) -> str:
        """Renders inline LaTeX. Uses a data-attribute if KaTeX is enabled."""
        src = _html.escape(node.attr_text("data-math") or "")
        if self._ctx.katex:
            return f'<span class="math-inline" data-math="{src}"></span>'
        return f"<code>{src}</code>"

    def _block_math(self, node) -> str:
        """Renders block LaTeX. Uses a div with data-attribute if KaTeX is enabled."""
        src = _html.escape(node.attr_text("data-math") or "")
        if self._ctx.katex:
            return f'<div class="math-block" data-math="{src}"></div>'
        return f"<pre><code>{src}</code></pre>"

    def _component_fallback(self, node) -> str:
        """
        Fallback renderer for custom components not found in RenderContext.
        Renders as a <div> with data-attributes to preserve props for external processing.
        """
        name  = node.node_type
        parts = [f'data-component="{name}"']
        for k, v in (node.attributes or {}).items():
            if isinstance(v, str):
                parts.append(f'data-{k}="{_html.escape(v)}"')
            elif isinstance(v, bool) and v:
                parts.append(f"data-{k}")
        attrs = " ".join(parts)
        if node.self_closing:
            return f"<div {attrs}></div>"
        inner = "".join(self._node(c) for c in node.children)
        return f"<div {attrs}>{inner}</div>"


def render_html(nodes, *, components=None, katex=True) -> str:
    """
    High-level utility to render a list of MDX nodes to HTML.
    
    Args:
        nodes: List of AST nodes.
        components: Dictionary of custom component names to rendering functions.
        katex: Whether to use KaTeX-compatible data attributes for math.
        
    Returns:
        A safe HTML string.
    """
    return HtmlRenderer(components=components, katex=katex).render(nodes)