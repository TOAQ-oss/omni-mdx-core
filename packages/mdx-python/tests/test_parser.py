import pytest
from unittest.mock import patch
import omni_mdx
from omni_mdx.parser import MDXParser, MdxNode
from omni_mdx.exceptions import MDXSyntaxError

class TestParser:
    def test_empty_string(self):
        result = omni_mdx.parse("")
        assert result.length == 0

    def test_valid_markdown_returns_ast(self):
        nodes = omni_mdx.parse("# Principal Title\nParagraph.")
        assert nodes.length == 2
        assert nodes.nodes[0].node_type == "h1"
        assert nodes.nodes[0].text_content() == "Principal Title"

    def test_invalid_type_raises(self):
        with pytest.raises(TypeError, match="Input must be a string"):
            omni_mdx.parse(None)

    def test_jsx_component_parsing(self):
        mdx = '<Note type="warning" alert>Warning</Note>'
        nodes = omni_mdx.parse(mdx)
        note = nodes.nodes[0]
        assert note.node_type == "Note"
        assert note.attributes.get("alert") is True
        assert note.attr_text("type") == "warning"

    def test_ast_search_misses(self):
        nodes = omni_mdx.parse("<Box>Text</Box>")
        box = nodes.nodes[0]
        assert box.find("Missing") is None
        assert box.find_all("Missing") == []

    def test_json_error_handling(self):
        with patch('omni_mdx.core_interface.CoreInterface.parse') as mock_parse:
            mock_parse.return_value = "invalid { json"
            parser = MDXParser()
            with pytest.raises(MDXSyntaxError, match="Invalid JSON from Rust core"):
                parser.parse("# test")

    def test_malformed_attributes_json(self):
        node = MdxNode({"node_type": "div", "attributes": "{broken}"})
        assert node.attributes == {}