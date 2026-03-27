"""
omni_mdx — Moteur MDX Python + Rust.

Le binaire Rust (omni_mdx_core.pyd) est embarqué directement dans ce
package — un seul pip install suffit, aucune installation séparée.

Quick start
-----------
    import omni_mdx

    # Parse
    nodes = omni_mdx.parse("# Hello\\n\\n$E = mc^2$")

    # Rendu HTML (pour web)
    html = omni_mdx.render_html(nodes)

    # Rendu Qt natif (pour desktop, 0 HTML)
    from omni_mdx import OmniMDX
    engine = OmniMDX()
    widget = engine.render_qt(nodes, parent=my_parent)
"""

from .parser import MDXParser
from .engine import OmniMDX
from .renderer import HtmlRenderer, render_html
from .exceptions import OmniMDXError, MDXSyntaxError, MDXRenderError

# Instance partagée pour l'usage fonctionnel
_parser = MDXParser()
parse = _parser.parse

__all__ = [
    # Fonctions
    "parse",
    "render_html",
    # Classes
    "MDXParser",
    "OmniMDX",
    "HtmlRenderer",
    # Exceptions
    "OmniMDXError",
    "MDXSyntaxError",
    "MDXRenderError",
]

__version__ = "0.1.10"