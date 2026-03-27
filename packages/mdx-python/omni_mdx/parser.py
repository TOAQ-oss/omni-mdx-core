from __future__ import annotations
from typing import List
from .core_interface import CoreInterface

class MDXParser:
    def __init__(self):
        self._interface = CoreInterface()

    def parse(self, mdx_text: str) -> List:
        """
        Parse MDX text and return the list of root MdxNode (Zero-Copy).
        """
        return self._interface.parse(mdx_text)