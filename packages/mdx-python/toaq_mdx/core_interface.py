"""
toaq_mdx.core_interface — Loading the Rust binary.

The toaq_parser_core.pyd file is placed directly in the package folder,
which allows it to be distributed without separate installation.
"""

import importlib
import importlib.util
import os
import sys
from pathlib import Path

from .exceptions import CoreNotAvailableError


def _load_core():
    """
    Load toaq_parser_core from the package folder.
    First try a direct import (if already in sys.modules or PATH),
    then search for the .pyd in the same folder as this file.
    """
    try:
        return importlib.import_module("toaq_parser_core")
    except ImportError:
        pass

    pkg_dir = Path(__file__).parent
    for candidate in pkg_dir.glob("toaq_parser_core*.pyd"):
        spec = importlib.util.spec_from_file_location("toaq_parser_core", candidate)
        if spec and spec.loader:
            mod = importlib.util.module_from_spec(spec)
            sys.modules["toaq_parser_core"] = mod
            spec.loader.exec_module(mod)
            return mod

    for candidate in pkg_dir.glob("toaq_parser_core*.so"):
        spec = importlib.util.spec_from_file_location("toaq_parser_core", candidate)
        if spec and spec.loader:
            mod = importlib.util.module_from_spec(spec)
            sys.modules["toaq_parser_core"] = mod
            spec.loader.exec_module(mod)
            return mod

    raise CoreNotAvailableError(
        "Rust binary not found (toaq_parser_core.pyd / .so).\n"
        "Place the file in the toaq_mdx/ folder or run:\n"
        "  cd core-parser && maturin develop --features python"
    )


_core = _load_core()


class CoreInterface:
    """Low-level interface to the Rust parser."""

    @staticmethod
    def parse_to_json(mdx_text: str) -> str:
        """
        Call the Rust parser and return the serialized AST in JSON.

        Parameters
        ----------
        mdx_text : str
            Raw MDX source.

        Returns
        -------
        str
            JSON representing the list of root nodes.

        Raises
        ------
        MDXSyntaxError
            If the Rust parser raises a parsing error.
        """
        from .exceptions import MDXSyntaxError
        try:
            return _core.parse(mdx_text)
        except (RuntimeError, ValueError) as e:
            raise MDXSyntaxError(str(e)) from e