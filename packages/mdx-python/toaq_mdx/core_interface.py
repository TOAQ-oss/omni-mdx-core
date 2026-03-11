"""
toaq_mdx.core_interface — Chargement du binaire Rust.

Le fichier toaq_parser_core.pyd est placé directement dans le dossier
du package, ce qui permet de le distribuer sans installation séparée.
"""

import importlib
import importlib.util
import os
import sys
from pathlib import Path

from .exceptions import CoreNotAvailableError


def _load_core():
    """
    Charge toaq_parser_core depuis le dossier du package.
    Essaie d'abord un import direct (si déjà dans sys.modules ou PATH),
    puis cherche le .pyd dans le même dossier que ce fichier.
    """
    # 1. Import direct (maturin develop, ou déjà dans sys.path)
    try:
        return importlib.import_module("toaq_parser_core")
    except ImportError:
        pass

    # 2. Chargement depuis le dossier du package (distribution embarquée)
    pkg_dir = Path(__file__).parent
    for candidate in pkg_dir.glob("toaq_parser_core*.pyd"):
        spec = importlib.util.spec_from_file_location("toaq_parser_core", candidate)
        if spec and spec.loader:
            mod = importlib.util.module_from_spec(spec)
            sys.modules["toaq_parser_core"] = mod
            spec.loader.exec_module(mod)
            return mod

    # 3. Aussi essayer .so (Linux/macOS)
    for candidate in pkg_dir.glob("toaq_parser_core*.so"):
        spec = importlib.util.spec_from_file_location("toaq_parser_core", candidate)
        if spec and spec.loader:
            mod = importlib.util.module_from_spec(spec)
            sys.modules["toaq_parser_core"] = mod
            spec.loader.exec_module(mod)
            return mod

    raise CoreNotAvailableError(
        "Binaire Rust introuvable (toaq_parser_core.pyd / .so).\n"
        "Placez le fichier dans le dossier toaq_mdx/ ou lancez :\n"
        "  cd core-parser && maturin develop --features python"
    )


# Chargement unique au démarrage du module
_core = _load_core()


class CoreInterface:
    """Interface bas niveau vers le parser Rust."""

    @staticmethod
    def parse_to_json(mdx_text: str) -> str:
        """
        Appelle le parser Rust et retourne l'AST sérialisé en JSON.

        Parameters
        ----------
        mdx_text : str
            Source MDX brut.

        Returns
        -------
        str
            JSON représentant la liste de nœuds racine.

        Raises
        ------
        MDXSyntaxError
            Si le parser Rust lève une erreur de parsing.
        """
        from .exceptions import MDXSyntaxError
        try:
            return _core.parse(mdx_text)
        except (RuntimeError, ValueError) as e:
            raise MDXSyntaxError(str(e)) from e