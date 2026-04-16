import pytest
from unittest.mock import patch, MagicMock
import omni_mdx.core_interface as core_interface
from omni_mdx.exceptions import MDXSyntaxError, CoreNotAvailableError

def test_core_parse_value_error():
    with patch('omni_mdx.core_interface._core') as mock_core:
        mock_core.parse.side_effect = ValueError("Rust Syntax Error")
        with pytest.raises(MDXSyntaxError, match="Rust Syntax Error"):
            core_interface.CoreInterface.parse("# test")

def test_load_core_no_binary_found():
    with patch('omni_mdx.core_interface.importlib') as mock_importlib:
        mock_importlib.import_module.side_effect = ImportError
        with patch('omni_mdx.core_interface.Path.glob', return_value=[]):
            with pytest.raises(CoreNotAvailableError, match="Rust binary not found"):
                core_interface._load_core()

def test_load_core_fallback_found_but_fails():
    with patch('omni_mdx.core_interface.importlib') as mock_importlib:
        mock_importlib.import_module.side_effect = ImportError

        mock_file = MagicMock()
        mock_file.name = "omni_mdx_core.pyd"
        
        with patch('omni_mdx.core_interface.Path.glob', return_value=[mock_file]):
            mock_importlib.util.spec_from_file_location.return_value = None
            with pytest.raises(CoreNotAvailableError):
                core_interface._load_core()