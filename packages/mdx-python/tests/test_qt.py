import textwrap
from PyQt5.QtWidgets import QApplication, QLabel, QPushButton
from omni_mdx import parse
from omni_mdx.qt_renderer import QtRenderer, InlineLabel

app = QApplication.instance() or QApplication([])

class TestQtRendererFull:
    def test_renderer_full_dispatch(self):
        mdx = textwrap.dedent("""
            # H1
            ## H2
            ### H3
            #### H4
            ##### H5
            ###### H6

            Text **bold** *italic* ~~strikethrough~~ `code` [link](https://x.com)

            ---

            * Item A
            * Item B

            1. One
            2. Two

            > Citation

            ```python
            print(1)
            ```

            | H | B |
            |---|---|
            | 1 | 2 |

            $inline$
            
            $$
            block
            $$
        """).strip()
        nodes = parse(mdx).nodes
        renderer = QtRenderer()
        widget = renderer.render(nodes)
        
        assert widget.layout().count() > 8

    def test_unregistered_component_fallback(self):
        nodes = parse("<UnknownComp />").nodes
        renderer = QtRenderer()
        widget = renderer.render(nodes)
        
        labels = widget.findChildren(QLabel)
        assert any("UnknownComp" in l.text() for l in labels)

    def test_details_toggle(self):
        nodes = parse("<Details title='Info'>Secret</Details>").nodes
        renderer = QtRenderer()
        widget = renderer.render(nodes)
        
        btn = widget.findChild(QPushButton)
        assert btn is not None
        assert "▶" in btn.text()
        
        btn.click()
        assert "▼" in btn.text()

    def test_note_variants(self):
        mdx = textwrap.dedent("""
            <Note type="warning">W</Note>
            
            <Note type="error">E</Note>
            
            <Note type="success">S</Note>
            
            <Note type="info">I</Note>
        """).strip()
        nodes = parse(mdx).nodes
        renderer = QtRenderer()
        widget = renderer.render(nodes)
        assert widget.layout().count() >= 4