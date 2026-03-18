"""
omni_mdx.parser — Parsing MDX to Python AST.
"""

from __future__ import annotations
from typing import List

from .core_interface import CoreInterface
from .ast import AstNode, parse_ast
from .exceptions import MDXSyntaxError


class MDXParser:
    """
    Parse MDX to a list of AstNode.

    Usage
    -----
        parser = MDXParser()
        nodes = parser.parse("# Titre\\n\\n<Note>contenu</Note>")
    """

    def __init__(self):
        self._interface = CoreInterface()

    def parse(self, mdx_text: str) -> List[AstNode]:
        """
        Parse MDX text and return the list of root nodes.

        Parameters
        ----------
        mdx_text : str
            Raw MDX source.

        Returns
        -------
        list[AstNode]
            Root nodes of the AST.

        Raises
        ------
        MDXSyntaxError
            If the MDX contains a syntax error.
        """
        raw_json = self._interface.parse_to_json(mdx_text)
        return parse_ast(raw_json)

    def parse_to_dict(self, mdx_text: str) -> list:
        """
        Return the AST as a list of raw Python dicts
        (without deserialization to AstNode).
        Useful for debugging or JSON serialization.
        """
        import json
        raw_json = self._interface.parse_to_json(mdx_text)
        return json.loads(raw_json)