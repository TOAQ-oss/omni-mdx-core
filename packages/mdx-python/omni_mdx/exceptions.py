"""toaq_mdx.exceptions — Module exceptions."""


class OmniMDXError(Exception):
    """Base exception for toaq_mdx."""
    pass


class MDXSyntaxError(OmniMDXError):
    """MDX parsing error (unclosed tag, invalid token, etc.)."""
    pass


class MDXRenderError(OmniMDXError):
    """Error during AST node rendering."""
    pass


class CoreNotAvailableError(OmniMDXError):
    """The Rust binary (.pyd) is not available."""
    pass