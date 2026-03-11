"""
toaq_mdx.renderer — Rendu HTML depuis l'AST MDX.

Pour le rendu web (Next.js, Flask, FastAPI...).
Les formules mathématiques sont émises avec des attributs ``data-math``
prêts pour KaTeX côté client.

Usage
-----
    from toaq_mdx.renderer import HtmlRenderer

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
    """Options et registre de composants propagés dans le rendu."""
    def __init__(self, components=None, katex=True):
        self.components = components or {}
        self.katex = katex


_BUILTIN_HTML_COMPONENTS: frozenset = frozenset()  # HTML n'a pas de built-ins, tout est custom


class HtmlRenderer:
    """
    Convertit une liste d'AstNode en HTML string.

    Priorité de rendu pour un composant JSX :
    1. Composant enregistré par le développeur  (via ``components=`` ou ``register()``)
    2. Fallback générique                       (div avec data-component=...)

    Parameters
    ----------
    components : dict, optional
        Registre de composants custom.
        Signature : ``fn(node: AstNode, ctx: RenderContext) -> str``

        Exemple — définir un composant Note ::

            def render_note(node, ctx):
                kind  = node.attr_text("type") or "info"
                title = node.attr_text("title") or ""
                inner = ctx.renderer.render(node.children)
                return f'<aside class="note note--{kind}"><b>{title}</b>{inner}</aside>'

            renderer = HtmlRenderer(components={"Note": render_note})

    katex : bool
        Si True, les formules émettent ``data-math`` pour KaTeX côté client.
        Si False, elles sont dans des balises ``<code>``.
    """

    def __init__(self, components=None, katex=True):
        self._ctx = RenderContext(components=dict(components or {}), katex=katex)

    def register(self, name: str, fn) -> "HtmlRenderer":
        """Enregistre (ou remplace) un composant HTML. Retourne self."""
        self._ctx.components[name] = fn
        return self

    def unregister(self, name: str) -> "HtmlRenderer":
        """Supprime un composant enregistré. Retourne self."""
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
    """Raccourci fonctionnel pour HtmlRenderer().render(nodes)."""
    return HtmlRenderer(components=components, katex=katex).render(nodes)