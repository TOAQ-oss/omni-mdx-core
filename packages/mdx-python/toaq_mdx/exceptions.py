"""toaq_mdx.exceptions — Exceptions du module."""


class OmniMDXError(Exception):
    """Base exception pour toaq_mdx."""
    pass


class MDXSyntaxError(OmniMDXError):
    """Erreur de parsing MDX (tag non fermé, token invalide, etc.)."""
    pass


class MDXRenderError(OmniMDXError):
    """Erreur lors du rendu d'un nœud AST."""
    pass


class CoreNotAvailableError(OmniMDXError):
    """Le binaire Rust (.pyd) n'est pas disponible."""
    pass