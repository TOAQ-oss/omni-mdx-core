"""
omni_mdx.renderer — HTML rendering from AST MDX.

For web rendering (Next.js, Flask, FastAPI, etc.).
Mathematical formulas are output with ``data-math`` attributes
ready for KaTeX on the client side.

Usage
-----
    from omni_mdx.renderer import HtmlRenderer

    renderer = HtmlRenderer(components={"Note": my_note_fn})
    html = renderer.render(ast_nodes)
"""

from __future__ import annotations
import html as _html
from typing import Callable, Dict, List, Optional

from .ast import AstNode, AttrValue
from .exceptions import MDXRenderError


ComponentFn = Callable[["AstNode", "RenderContext"], str]


class RenderContext:
    """Options and component registry propagated in the rendering."""
    def __init__(self, components=None, katex=True):
        self.components = components or {}
        self.katex = katex


_BUILTIN_HTML_COMPONENTS: frozenset = frozenset()  # HTML has no built-ins, everything is custom


class HtmlRenderer:
    """
    Converts a list of AstNode to an HTML string.

    Rendering priority for a JSX component:
    1. Component registered by the developer  (via ``components=`` or ``register()``)
    2. Generic fallback                       (div with data-component=...)

    Parameters
    ----------
    components: dict, optional
        Custom component registry.
        Signature: ``fn(node: AstNode, ctx: RenderContext) -> str``

        Example — defining a Note component ::

            def render_note(node, ctx):
                kind  = node.attr_text(“type”) or “info”
                title = node.attr_text(“title”) or “”
                inner = ctx.renderer.render(node.children)
                return f'<aside class="note note--{kind}"><b>{title}</b>{inner}</aside>'

            renderer = HtmlRenderer(components={“Note”: render_note})

    katex: bool
        If True, formulas emit ``data-math`` for KaTeX on the client side.
        If False, they are enclosed in ``<code>`` tags.
    """

    def __init__(self, components=None, katex=True):
        self._ctx = RenderContext(components=dict(components or {}), katex=katex)

    def register(self, name: str, fn) -> "HtmlRenderer":
        """Register (or replace) an HTML component. Returns self."""
        self._ctx.components[name] = fn
        return self

    def unregister(self, name: str) -> "HtmlRenderer":
        """Unregister a registered component. Returns self."""
        self._ctx.components.pop(name, None)
        return self

    def render(self, nodes: List[AstNode]) -> str:
        return "".join(self._node(n) for n in nodes)

    _VOID = frozenset({"area","base","br","col","embed","hr","img",
                       "input","link","meta","param","source","track","wbr"})

    def _node(self, node: AstNode) -> str:
        t = node.node_type
        if t == "InlineMath":  return self._inline_math(node)
        if t == "BlockMath":   return self._block_math(node)
        if t == "text":        return _html.escape(node.content or "")
        if node.is_component:
            fn = self._ctx.components.get(t)
            return fn(node, self._ctx) if fn else self._component_fallback(node)
        return self._element(node)

    def _element(self, node: AstNode) -> str:
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

    def _attrs(self, node: AstNode) -> str:
        if not node.attributes:
            return ""
        parts = []
        for name, val in node.attributes.items():
            html_name = "class" if name == "className" else name
            if val.is_boolean:
                parts.append(f" {html_name}")
            elif val.is_text:
                parts.append(f' {html_name}="{_html.escape(val.text or "")}"')
        return "".join(parts)

    def _img(self, node: AstNode) -> str:
        src   = _html.escape(node.attr_text("src") or "")
        alt   = _html.escape(node.attr_text("alt") or "")
        title = node.attr_text("title")
        t     = f' title="{_html.escape(title)}"' if title else ""
        return f'<img src="{src}" alt="{alt}"{t}>'

    def _link(self, node: AstNode) -> str:
        href  = _html.escape(node.attr_text("href") or "#")
        title = node.attr_text("title")
        t     = f' title="{_html.escape(title)}"' if title else ""
        inner = "".join(self._node(c) for c in node.children)
        return f'<a href="{href}"{t}>{inner}</a>'

    def _code_block(self, node: AstNode) -> str:
        lang = node.attr_text("lang") or ""
        la   = f' class="language-{_html.escape(lang)}"' if lang else ""
        code = _html.escape(node.content or node.text_content())
        return f"<pre><code{la}>{code}</code></pre>"

    def _inline_math(self, node: AstNode) -> str:
        src = _html.escape(node.content or "")
        if self._ctx.katex:
            return f'<span class="math-inline" data-math="{src}"></span>'
        return f"<code>{src}</code>"

    def _block_math(self, node: AstNode) -> str:
        src = _html.escape(node.content or "")
        if self._ctx.katex:
            return f'<div class="math-block" data-math="{src}"></div>'
        return f"<pre><code>{src}</code></pre>"

    def _component_fallback(self, node: AstNode) -> str:
        name  = node.node_type
        parts = [f'data-component="{name}"']
        for k, v in (node.attributes or {}).items():
            if v.is_text:
                parts.append(f'data-{k}="{_html.escape(v.text or "")}"')
            elif v.is_boolean:
                parts.append(f"data-{k}")
        attrs = " ".join(parts)
        if node.self_closing:
            return f"<div {attrs}></div>"
        inner = "".join(self._node(c) for c in node.children)
        return f"<div {attrs}>{inner}</div>"


def render_html(nodes, *, components=None, katex=True) -> str:
    """Functional shortcut for HtmlRenderer().render(nodes)."""
    return HtmlRenderer(components=components, katex=katex).render(nodes)