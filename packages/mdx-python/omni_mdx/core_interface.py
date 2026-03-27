import importlib
import importlib.util
import sys
from pathlib import Path
from .exceptions import CoreNotAvailableError

def _load_core():
    try:
        return importlib.import_module("omni_mdx_core")
    except ImportError:
        pass

    pkg_dir = Path(__file__).parent
    for candidate in pkg_dir.glob("omni_mdx_core*.pyd"):
        spec = importlib.util.spec_from_file_location("omni_mdx_core", candidate)
        if spec and spec.loader:
            mod = importlib.util.module_from_spec(spec)
            sys.modules["omni_mdx_core"] = mod
            spec.loader.exec_module(mod)
            return mod

    for candidate in pkg_dir.glob("omni_mdx_core*.so"):
        spec = importlib.util.spec_from_file_location("omni_mdx_core", candidate)
        if spec and spec.loader:
            mod = importlib.util.module_from_spec(spec)
            sys.modules["omni_mdx_core"] = mod
            spec.loader.exec_module(mod)
            return mod

    raise CoreNotAvailableError(
        "Rust binary not found. Place the file in the omni_mdx/ folder or run 'maturin develop'"
    )

_core = _load_core()

class CoreInterface:
    @staticmethod
    def parse(mdx_text: str):
        from .exceptions import MDXSyntaxError
        try:
            return _core.parse(mdx_text)
        except ValueError as e:
            raise MDXSyntaxError(str(e)) from e