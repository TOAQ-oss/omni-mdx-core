import pytest
import textwrap
from PyQt5.QtWidgets import QApplication, QWidget
from PyQt5.QtCore import QRect
from omni_mdx import parse
from omni_mdx.math_render import latex_to_unicode, latex_to_pixmap, latex_to_pixmap_available
from omni_mdx.qt_renderer import QtRenderer, FlowLayout

app = QApplication.instance() or QApplication([])

def test_latex_to_unicode_conversions():
    assert "(1/2)" in latex_to_unicode(r"\frac{1}{2}")
    text = latex_to_unicode(r"\alpha \geq \beta")
    assert "α ≥ β" in text
    assert "x²" in latex_to_unicode("x^2")
    assert "H₂O" in latex_to_unicode("H_2O")

def test_latex_to_pixmap():
    if not latex_to_pixmap_available():
        pytest.skip("matplotlib non disponible")
    pixmap = latex_to_pixmap(r"E=mc^2")
    assert pixmap is not None
    assert not pixmap.isNull()

class TestQtRendererMath:
    def test_flow_layout_deep(self):
        parent = QWidget()
        layout = FlowLayout(parent)
        for _ in range(5):
            w = QWidget()
            w.setFixedSize(50, 20)
            layout.addWidget(w)
        
        layout.setGeometry(QRect(0, 0, 100, 500))
        h = layout.heightForWidth(100)
        assert h > 0

    def test_full_document_render(self):
        mdx = textwrap.dedent("""
            # Title
            > Citation
            * List
            1. Ordered

            | Tab | Header |
            |---|---|
            | Cell | Data |

            ---
            Link [TOAQ](https://toaq.fr) and `code`
        """).strip()
        nodes = parse(mdx).nodes
        renderer = QtRenderer()
        widget = renderer.render(nodes)
        assert widget.layout().count() >= 5