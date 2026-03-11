"""
toaq_mdx.parser — Parsing MDX vers AST Python.
"""

from __future__ import annotations
from typing import List

from .core_interface import CoreInterface
from .ast import AstNode, parse_ast
from .exceptions import MDXSyntaxError


class MDXParser:
    """
    Parse du MDX en liste d'AstNode.

    Usage
    -----
        parser = MDXParser()
        nodes = parser.parse("# Titre\\n\\n<Note>contenu</Note>")
    """

    def __init__(self):
        self._interface = CoreInterface()

    def parse(self, mdx_text: str) -> List[AstNode]:
        """
        Parse un string MDX et retourne la liste des nœuds racine.

        Parameters
        ----------
        mdx_text : str
            Source MDX brut.

        Returns
        -------
        list[AstNode]
            Nœuds racine de l'AST.

        Raises
        ------
        MDXSyntaxError
            Si le MDX contient une erreur de syntaxe.
        """
        raw_json = self._interface.parse_to_json(mdx_text)
        return parse_ast(raw_json)

    def parse_to_dict(self, mdx_text: str) -> list:
        """
        Retourne l'AST sous forme de liste de dicts Python bruts
        (sans désérialisation en AstNode).
        Utile pour du debug ou de la sérialisation JSON.
        """
        import json
        raw_json = self._interface.parse_to_json(mdx_text)
        return json.loads(raw_json)