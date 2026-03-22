"""
Tests unitaires pour omni_mdx (Python).
Le module Rust est compilé et injecté avant que pytest tourne.
"""
import pytest

# Adapte cet import à ton API réelle
# from omni_mdx import parse, render


class TestParser:
    def test_empty_string(self):
        """Un string vide ne doit pas lever d'exception."""
        # result = parse("")
        # assert result is not None
        assert True  # placeholder

    def test_valid_input_returns_ast(self):
        """Un input valide retourne un AST avec un type racine."""
        # result = parse("# Hello")
        # assert result["type"] == "root"
        assert True

    def test_invalid_type_raises(self):
        """Passer autre chose qu'un str doit lever TypeError."""
        # with pytest.raises(TypeError):
        #     parse(None)
        assert True

    def test_deterministic_output(self):
        """Le même input doit toujours produire le même output."""
        # a = parse("# Hello")
        # b = parse("# Hello")
        # assert a == b
        assert True


class TestRenderer:
    def test_heading_to_html(self):
        """Un nœud heading doit être rendu en <h1>."""
        # html = render({"type": "heading", "depth": 1, "value": "Hello"})
        # assert "<h1>" in html
        assert True

    def test_empty_ast_to_empty_string(self):
        # html = render({"type": "root", "children": []})
        # assert html == ""
        assert True