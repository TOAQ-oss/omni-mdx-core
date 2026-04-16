from omni_mdx.renderer import HtmlRenderer, render_html
from omni_mdx.parser import MdxNode

class TestHtmlRenderer:
    def test_register_unregister(self):
        renderer = HtmlRenderer()
        renderer.register("Alert", lambda n, ctx: "<div>alert</div>")
        assert "Alert" in renderer._ctx.components
        renderer.unregister("Alert")
        assert "Alert" not in renderer._ctx.components

    def test_void_elements(self):
        node_br = MdxNode({"node_type": "br"})
        assert render_html([node_br]) == "<br>"

        node_hr = MdxNode({"node_type": "hr"})
        assert render_html([node_hr]) == "<hr>"

        node_input = MdxNode({"node_type": "input", "attributes": {"type": "text"}})
        assert render_html([node_input]) == '<input type="text" >'

    def test_links_and_images(self):
        node_img = MdxNode({
            "node_type": "img", 
            "attributes": {"src": "a.png", "alt": "desc", "title": "title"}
        })
        assert render_html([node_img]) == '<img src="a.png" alt="desc" title="title">'

        node_link = MdxNode({
            "node_type": "a",
            "attributes": {"href": "https://x.com", "title": "website"},
            "children": [{"node_type": "text", "content": "click"}]
        })
        assert render_html([node_link]) == '<a href="https://x.com" title="website">click</a>'

    def test_attributes_mapping_and_escaping(self):
        node = MdxNode({
            "node_type": "div",
            "attributes": {
                "className": "btn",
                "hidden": {"kind": "boolean"},
                "data-info": "one & two"
            },
            "children": [{"node_type": "text", "content": "<b>"}]
        })
        rendered = render_html([node])
        assert 'class="btn"' in rendered
        assert ' hidden' in rendered
        assert 'data-info="one &amp; two"' in rendered
        assert '&lt;b&gt;' in rendered

    def test_math_rendering_logic(self):
        node_math = MdxNode({
            "node_type": "InlineMath",
            "attributes": {"data-math": "E=mc^2"}
        })
    
        assert 'data-math="E=mc^2"' in render_html([node_math])

        renderer = HtmlRenderer(katex=False)
        assert renderer.render([node_math]) == "<code>E=mc^2</code>"

        node_block = MdxNode({"node_type": "BlockMath", "content": "x=1"})
        assert renderer.render([node_block]) == "<pre><code>x=1</code></pre>"

    def test_component_fallback(self):
        node = MdxNode({
            "node_type": "CustomComp",
            "attributes": {"prop": "val"},
            "self_closing": True
        })

        assert 'data-component="CustomComp"' in render_html([node])
        assert 'data-prop="val"' in render_html([node])

    def test_raw_html_injection(self):
        node = MdxNode({"node_type": "html", "content": "<div>safe</div>"})
        assert render_html([node]) == "<div>safe</div>"

        node_code = MdxNode({
            "node_type": "pre",
            "attributes": {"lang": "py"},
            "content": "print(1)"
        })
        assert '<code class="language-py">print(1)</code>' in render_html([node_code])

    def test_renderer_edge_cases(self):
        node_attr = MdxNode({
            "node_type": "div",
            "attributes": {"invalid": 123} 
        })
        assert 'invalid' not in render_html([node_attr])

        node_code = MdxNode({"node_type": "pre", "content": "simple code"})
        assert '<code>simple code</code>' in render_html([node_code])
        assert 'class="language-' not in render_html([node_code])