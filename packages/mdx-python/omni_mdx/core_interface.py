import importlib
import importlib.util
import sys
from pathlib import Path
from .exceptions import CoreNotAvailableError

def _load_core():
    """
    Internal helper to dynamically load the native Rust module.
    
    It first attempts a standard import. If that fails (e.g., during development 
    or custom installs), it scans the package directory for shared library 
    files and loads them manually into sys.modules.
    
    Returns:
        The loaded native module.
        
    Raises:
        CoreNotAvailableError: If no compatible Rust binary is found.
    """
    # 1. Try standard import (standard installation)
    try:
        return importlib.import_module("omni_mdx_core")
    except ImportError:
        pass

    # 2. Search for compiled binaries in the current directory
    pkg_dir = Path(__file__).parent

    # Check for Windows binaries (.pyd)
    for candidate in pkg_dir.glob("omni_mdx_core*.pyd"):
        spec = importlib.util.spec_from_file_location("omni_mdx_core", candidate)
        if spec and spec.loader:
            mod = importlib.util.module_from_spec(spec)
            sys.modules["omni_mdx_core"] = mod
            spec.loader.exec_module(mod)
            return mod

    # Check for Unix/macOS binaries (.so)
    for candidate in pkg_dir.glob("omni_mdx_core*.so"):
        spec = importlib.util.spec_from_file_location("omni_mdx_core", candidate)
        if spec and spec.loader:
            mod = importlib.util.module_from_spec(spec)
            sys.modules["omni_mdx_core"] = mod
            spec.loader.exec_module(mod)
            return mod

    # 3. Fail if no binary is found
    raise CoreNotAvailableError(
        "Rust binary not found. Place the file in the omni_mdx/ folder or run 'maturin develop'"
    )

# Singleton instance of the loaded native core
_core = _load_core()

class CoreInterface:
    """
    Low-level wrapper for the native Rust core.
    
    Provides a clean bridge for data transfer between Python strings 
    and the native parsing logic.
    """
    @staticmethod
    def parse(mdx_text: str):
        """
        Calls the native Rust parser.
        
        Args:
            mdx_text (str): The raw MDX content to parse.
            
        Returns:
            The native AST structure (usually a list of nodes).
            
        Raises:
            MDXSyntaxError: If the Rust engine detects invalid MDX/JSX syntax.
        """
        from .exceptions import MDXSyntaxError
        try:
            return _core.parse(mdx_text)
        except ValueError as e:
            # Map native ValueErrors (syntax) to our specific exception
            raise MDXSyntaxError(str(e)) from e