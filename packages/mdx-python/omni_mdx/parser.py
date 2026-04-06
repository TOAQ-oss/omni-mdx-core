from __future__ import annotations
from typing import List
from .core_interface import CoreInterface

class MDXParser:
    """
    The main coordinator for MDX parsing operations.
    
    This class manages the lifecycle of the underlying native interface and 
    provides high-level methods to transform raw text into navigable node structures.
    """
    def __init__(self):
        """
        Initializes the parser by establishing a connection to the native 
        CoreInterface (compiled Rust library).
        """
        self._interface = CoreInterface()

    def parse(self, mdx_text: str) -> List:
        """
        Parses raw MDX text and returns a list of root AST nodes.
        
        This operation uses a 'Zero-Copy' approach where possible, minimizing 
        memory allocations by referencing the original buffer or using highly 
        optimized binary transfers between Rust and Python.

        Args:
            mdx_text (str): The raw Markdown/MDX string to be processed.

        Returns:
            List[MdxNode]: A list of objects representing the root level of 
                           the Abstract Syntax Tree.
        """
        return self._interface.parse(mdx_text)