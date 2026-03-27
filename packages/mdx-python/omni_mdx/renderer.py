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
    def __init__(self, components=None, katex=True):
        self.components = components or {}
        self.katex = katex

_BUILTIN_HTML_COMPONENTS: frozenset = frozenset()

class HtmlRenderer:
    def __init__(self, components=None, katex=True):
        self._ctx = RenderContext(components=dict(components or {}), katex=katex)

    def register(self, name: str, fn) -> "HtmlRenderer":
        self._ctx.components[name] = fn
        return self

    def unregister(self, name: str) -> "HtmlRenderer":
        self._ctx.components.pop(name, None)
        return self

    def render(self, nodes: List) -> str:
        return "".join(self._node(n) for n in nodes)

    _VOID = frozenset({"area","base","br","col","embed","hr","img",
                       "input","link","meta","param","source","track","wbr"})

    def _node(self, node) -> str:
        t = node.node_type
        if t == "InlineMath":  return self._inline_math(node)
        if t == "BlockMath":   return self._block_math(node)
        if t == "text":        return _html.escape(node.content or "")
        if node.is_component:
            fn = self._ctx.components.get(t)
            return fn(node, self._ctx) if fn else self._component_fallback(node)
        return self._element(node)

    def _element(self, node) -> str:
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
        if not node.attributes:
            return ""
        parts = []
        for name, val in node.attributes.items():
            html_name = "class" if name == "className" else name
            # 🚀 Changement ici : val est maintenant un booléen ou une string natif Python !
            if isinstance(val, bool) and val:
                parts.append(f" {html_name}")
            elif isinstance(val, str):
                parts.append(f' {html_name}="{_html.escape(val)}" ')
        return "".join(parts)

    def _img(self, node) -> str:
        src   = _html.escape(node.attr_text("src") or "")
        alt   = _html.escape(node.attr_text("alt") or "")
        title = node.attr_text("title")
        t     = f' title="{_html.escape(title)}"' if title else ""
        return f'<img src="{src}" alt="{alt}"{t}>'

    def _link(self, node) -> str:
        href  = _html.escape(node.attr_text("href") or "#")
        title = node.attr_text("title")
        t     = f' title="{_html.escape(title)}"' if title else ""
        inner = "".join(self._node(c) for c in node.children)
        return f'<a href="{href}"{t}>{inner}</a>'

    def _code_block(self, node) -> str:
        lang = node.attr_text("lang") or ""
        la   = f' class="language-{_html.escape(lang)}"' if lang else ""
        code = _html.escape(node.content or node.text_content())
        return f"<pre><code{la}>{code}</code></pre>"

    def _inline_math(self, node) -> str:
        src = _html.escape(node.content or "")
        if self._ctx.katex:
            return f'<span class="math-inline" data-math="{src}"></span>'
        return f"<code>{src}</code>"

    def _block_math(self, node) -> str:
        src = _html.escape(node.content or "")
        if self._ctx.katex:
            return f'<div class="math-block" data-math="{src}"></div>'
        return f"<pre><code>{src}</code></pre>"

    def _component_fallback(self, node) -> str:
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
    return HtmlRenderer(components=components, katex=katex).render(nodes)