from omni_mdx.engine import OmniMDX

class TestOmniMDXEngine:
    def test_engine_init(self):
        engine_empty = OmniMDX()
        assert engine_empty.components == {}

        engine_with = OmniMDX(components={"Test": lambda p, c: "ok"})
        assert "Test" in engine_with.components

    def test_parse_to_ast(self):
        engine = OmniMDX()
        nodes = engine.parse_to_ast("# Heading")
        
        assert len(nodes) == 1
        assert nodes[0].node_type == "h1"
        assert nodes[0].text_content() == "Heading"

    def test_render_standard_nodes(self):
        engine = OmniMDX()
        result = engine.render("Hello **World**")
        
        assert isinstance(result, list)
        assert len(result) == 1
        
        p_node = result[0]
        assert p_node["type"] == "p"
        
        children = p_node["children"]
        assert children[0] == "Hello "
        assert children[1]["type"] == "strong"
        assert children[1]["children"][0] == "World"

    def test_render_custom_components(self):
        def mock_card(props, children):
            return {
                "widget_type": "QCard",
                "title": props.get("title", "No title"),
                "is_active": props.get("active", False),
                "inner": children
            }

        engine = OmniMDX(components={"Card": mock_card})
        
        result = engine.render('<Card title="Test" active>Content</Card>')
        
        assert len(result) == 1
        
        card = result[0]
        assert card["widget_type"] == "QCard"
        assert card["title"] == "Test"
        assert card["is_active"] is True
        assert card["inner"] == ["Content"]