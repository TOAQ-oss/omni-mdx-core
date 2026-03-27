"""
Integration tests for omni_mdx (Python).
Verifies HTML rendering from the AST, including KaTeX and JSX support.
"""
import omni_mdx
from omni_mdx.renderer import HtmlRenderer

class TestRendererIntegration:
    def test_render_basic_html(self):
        """A simple AST must generate valid HTML."""
        ast = omni_mdx.parse("## Subtitle\nText.")
        renderer = HtmlRenderer()
        html = renderer.render(ast.nodes)
        
        assert "<h2>Subtitle</h2>" in html
        assert "<p>Text.</p>" in html

    def test_render_math_nodes_katex(self):
        """
        Crucial test: Verifies that mathematical formulas are properly
        extracted from virtual children and placed in data-math attributes.
        """
        ast = omni_mdx.parse("Inline: $a^2+b^2=c^2$\n\nBlock:\n$$E=mc^2$$")
        
        renderer = HtmlRenderer(katex=True)
        html = renderer.render(ast.nodes)
        
        assert '<span class="math-inline" data-math="a^2+b^2=c^2"></span>' in html
        assert '<div class="math-block" data-math="E=mc^2"></div>' in html

    def test_render_custom_jsx_component(self):
        """JSX components must be able to use a custom rendering function."""
        ast = omni_mdx.parse('<Alert level="danger">System halt</Alert>')
        
        def render_alert(node, ctx):
            level = node.attr_text("level") or "info"
            text = node.text_content()
            return f'<div class="custom-alert alert-{level}">{text}</div>'
            
        renderer = HtmlRenderer(components={"Alert": render_alert})
        html = renderer.render(ast.nodes)
        
        assert '<div class="custom-alert alert-danger">System halt</div>' in html

    def test_component_fallback(self):
        """An unregistered JSX component must use the fallback rendering."""
        ast = omni_mdx.parse('<Unknown prop="test">Content</Unknown>')
        
        renderer = HtmlRenderer()
        html = renderer.render(ast.nodes)
        
        assert '<div data-component="Unknown" data-prop="test">Content</div>' in html