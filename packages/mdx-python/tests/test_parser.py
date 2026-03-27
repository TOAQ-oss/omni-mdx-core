"""
Tests unitaires pour le parser omni_mdx (Python).
Vérifie la communication avec le moteur Rust et la construction de l'AST.
"""
import pytest
import omni_mdx

class TestParser:
    def test_empty_string(self):
        """Un string vide doit renvoyer une liste vide, sans planter."""
        result = omni_mdx.parse("")
        assert result.length == 0

    def test_valid_markdown_returns_ast(self):
        """Un input valide doit retourner l'arbre AST correct."""
        nodes = omni_mdx.parse("# Titre Principal\nUn paragraphe.")
        
        assert nodes.length == 2
        assert nodes.nodes[0].node_type == "h1"
        assert nodes.nodes[0].text_content() == "Titre Principal"
        assert nodes.nodes[1].node_type == "p"
        assert nodes.nodes[1].text_content() == "Un paragraphe."

    def test_invalid_type_raises(self):
        """Passer autre chose qu'une chaîne de caractères doit lever une erreur."""
        with pytest.raises(TypeError):
            omni_mdx.parse(None)
        with pytest.raises(TypeError):
            omni_mdx.parse(123)

    def test_jsx_component_parsing(self):
        """Le parser doit extraire correctement les composants JSX et leurs attributs."""
        mdx = '<Note type="warning" alert>Attention</Note>'
        nodes = omni_mdx.parse(mdx)
        
        assert nodes.length == 1

        note = nodes.nodes[0]
        
        assert note.node_type == "Note"
        assert note.is_component is True
        assert note.attr_text("type") == "warning"
        assert note.attributes.get("alert") is True
        assert note.text_content() == "Attention"

    def test_ast_find_methods(self):
        """Les méthodes find() et find_all() doivent parcourir l'arbre correctement."""
        nodes = omni_mdx.parse("<Box><Item>A</Item><Badge>B</Badge><Item>C</Item></Box>")
        box = nodes.nodes[0]
        
        assert box.find("Item").text_content() == "A"
        assert len(box.find_all("Item")) == 2
        assert len(box.find_all("Badge")) == 1