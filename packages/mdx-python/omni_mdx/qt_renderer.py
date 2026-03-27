"""
omni_mdx.qt_renderer — AST rendered in native PyQt5 widgets.

No HTML produced. Each Any becomes a dedicated Qt widget.
Mathematical formulas are rendered:
  - InlineMath → Unicode text via math_render.latex_to_unicode
  - BlockMath → PNG image via matplotlib.mathtext (if available)
                 fallback to Unicode text otherwise

Usage
-----
    from omni_mdx.qt_renderer import QtRenderer

    renderer = QtRenderer(components={"Note": my_note_widget_fn})
    widget = renderer.render(ast_nodes, parent=my_parent)
"""

from __future__ import annotations
from typing import Callable, Dict, List, Optional

from PyQt5.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QFrame,
    QPushButton, QSizePolicy, QLayout,
)
from PyQt5.QtCore import Qt, QRect, QSize, QPoint
from PyQt5.QtGui import QFont, QPixmap

from .math_render import latex_to_unicode, latex_to_pixmap, latex_to_pixmap_available

class FlowLayout(QLayout):
    """Dispose the widgets in lines with automatic line break (word-wrap)."""

    def __init__(self, parent=None, h_spacing=2, v_spacing=4):
        super().__init__(parent)
        self._items: list = []
        self._h = h_spacing
        self._v = v_spacing

    def addItem(self, item):       self._items.append(item)
    def count(self):               return len(self._items)
    def itemAt(self, i):           return self._items[i] if 0 <= i < len(self._items) else None
    def takeAt(self, i):           return self._items.pop(i) if 0 <= i < len(self._items) else None
    def hasHeightForWidth(self):   return True
    def sizeHint(self):            return self.minimumSize()

    def minimumSize(self):
        s = QSize()
        for item in self._items:
            s = s.expandedTo(item.minimumSize())
        return s

    def heightForWidth(self, width):
        return self._layout(QRect(0, 0, width, 0), test=True)

    def setGeometry(self, rect):
        super().setGeometry(rect)
        self._layout(rect, test=False)

    def _layout(self, rect, test):
        x, y, line_h = rect.x(), rect.y(), 0
        for item in self._items:
            w = item.widget()
            if w is None:
                continue
            iw = item.sizeHint().width()
            ih = item.sizeHint().height()
            if x + iw > rect.right() and x > rect.x():
                x, y, line_h = rect.x(), y + line_h + self._v, 0
            if not test:
                item.setGeometry(QRect(QPoint(x, y), QSize(iw, ih)))
            x += iw + self._h
            line_h = max(line_h, ih)
        return y + line_h - rect.y()

class InlineLabel(QLabel):
    def __init__(self, text: str, bold=False, italic=False,
                 strike=False, code=False, math_inline=False, parent=None):
        super().__init__(text, parent)
        self.setTextInteractionFlags(Qt.TextSelectableByMouse)
        self.setWordWrap(False)

        if code or math_inline:
            font = QFont("JetBrains Mono", 10)
        else:
            font = QFont("Segoe UI", 10)
        font.setBold(bold)
        font.setItalic(italic or math_inline)
        font.setStrikeOut(strike)
        self.setFont(font)

        if code:
            self.setStyleSheet(
                "background:#f3f4f6;color:#be185d;padding:1px 5px;"
                "border-radius:3px;border:1px solid #e5e7eb;"
            )
        elif math_inline:
            self.setStyleSheet(
                "background:#faf5ff;color:#5b21b6;padding:1px 5px;"
                "border-radius:3px;"
            )
        else:
            self.setStyleSheet("background:transparent;padding:0;color:#1a202c;")


_BUILTIN_COMPONENTS = {"Note", "Details"}

class QtRenderer:
    """
    Converts a list of Anys into native Qt widgets.

    Rendering priority for a JSX component:
    1. Component registered by the developer  (via ``components=`` or ``register()``)
    2. Built-in component                       (Note, Details)
    3. Generic fallback                       (purple box with the name)

    Parameters
    ----------
    components: dict, optional
        Initial registry of custom components.
        Signature: ``fn(node: Any, renderer: QtRenderer) -> QWidget``

        Example — override Note with a custom style ::

            from PyQt5.QtWidgets import QFrame, QVBoxLayout, QLabel

            def my_note(node, renderer):
                frame = QFrame()
                frame.setStyleSheet(“border: 2px solid red; border-radius: 8px;”)
                layout = QVBoxLayout(frame)
                title = node.attr_text(“title”) or ‘’
                layout.addWidget(QLabel(f“⚠ {title}”))
                for w in renderer._render_mixed_children(node.children):
                    layout.addWidget(w)
                return frame

            renderer = QtRenderer(components={“Note”: my_note})

        The component receives:
        - ``node``     : the complete Any with ``node.attributes``, ``node.children``
        - ``renderer``: the current QtRenderer — use ``renderer._render_mixed_children()``
          to render children, or ``renderer.render()`` for a complete subtree.

    math_font_size: int
        Font size (pt) for image rendering of block math formulas.
    """

    def __init__(self, components: Optional[Dict[str, Callable]] = None,
                 math_font_size: int = 14):
        self.components:     Dict[str, Callable] = dict(components or {})
        self.math_font_size: int                 = math_font_size

    def register(self, name: str, fn: Callable) -> "QtRenderer":
        """
        Registers (or replaces) a component.

        Can override a built-in (Note, Details) or define a new one.
        Returns ``self`` for chaining.

        Example
        -------
        >>> renderer.register("Note", my_note_fn).register("Alert", my_alert_fn)
        """
        self.components[name] = fn
        return self

    def unregister(self, name: str) -> "QtRenderer":
        """
        Deletes a registered component.

        If ``name`` is a built-in (Note, Details), the built-in takes precedence.
        Returns ``self`` for chaining.
        """
        self.components.pop(name, None)
        return self

    def registered(self) -> Dict[str, Callable]:
        """Returns a copy of the current registry (user components only)."""
        return dict(self.components)

    @property
    def builtin_components(self) -> frozenset:
        """Names of built-in components (always available even without registration)."""
        return frozenset(_BUILTIN_COMPONENTS)

    def render(self, nodes: List[Any], parent: Optional[QWidget] = None) -> QWidget:
        """Returns a QWidget containing the rendering of all nodes."""
        container = QWidget(parent)
        container.setStyleSheet("background:transparent;")
        layout = QVBoxLayout(container)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(8)
        for node in nodes:
            w = self._node(node)
            if w:
                layout.addWidget(w)
        layout.addStretch()
        return container

    def _node(self, node: Any) -> Optional[QWidget]:
        t = node.node_type
        dispatch = {
            "h1": self._heading, "h2": self._heading,
            "h3": self._heading, "h4": self._heading,
            "h5": self._heading, "h6": self._heading,
            "p":           self._paragraph,
            "BlockMath":   self._block_math,
            "ul":          lambda n: self._list(n, ordered=False),
            "ol":          lambda n: self._list(n, ordered=True),
            "blockquote":  self._blockquote,
            "pre":         self._code_block,
            "hr":          self._separator,
            "table":       self._table,
        }
        fn = dispatch.get(t)
        if fn:
            return fn(node)
        if t == "InlineMath":
            return self._inline_math_as_block(node)
        if node.is_component:
            return self._component(node)
        if node.children:
            return self.render(node.children)
        return None

    def _heading(self, node: Any) -> QLabel:
        level = int(node.node_type[1])
        lbl   = QLabel(node.text_content())
        lbl.setTextInteractionFlags(Qt.TextSelectableByMouse)
        lbl.setWordWrap(True)
        font = QFont("Segoe UI", {1:22,2:18,3:15,4:13,5:12,6:11}.get(level, 12))
        font.setBold(True)
        lbl.setFont(font)
        color = {1:"#111827", 2:"#1f2937", 3:"#374151"}.get(level, "#4b5563")
        border = "border-bottom:2px solid #e5e7eb;padding-bottom:6px;" if level == 1 else ""
        margin = {1:"margin-top:8px;", 2:"margin-top:16px;", 3:"margin-top:12px;"}.get(level, "")
        lbl.setStyleSheet(f"color:{color};background:transparent;{border}{margin}")
        return lbl

    def _paragraph(self, node: Any) -> QWidget:
        w = QWidget()
        w.setStyleSheet("background:transparent;")
        layout = FlowLayout(w, h_spacing=2, v_spacing=2)
        w.setLayout(layout)
        self._inline_into(node.children, layout)
        return w

    def _inline_into(self, children: List[Any], layout: FlowLayout,
                     bold=False, italic=False, strike=False):
        """Populate a FlowLayout with inline nodes."""
        for child in children:
            t = child.node_type
            if t == "text":
                lbl = InlineLabel(child.content or "", bold=bold,
                                  italic=italic, strike=strike)
                layout.addWidget(lbl)
            elif t == "strong":
                self._inline_into(child.children, layout, bold=True,
                                  italic=italic, strike=strike)
            elif t == "em":
                self._inline_into(child.children, layout, bold=bold,
                                  italic=True, strike=strike)
            elif t == "del":
                self._inline_into(child.children, layout, bold=bold,
                                  italic=italic, strike=True)
            elif t == "code":
                layout.addWidget(InlineLabel(child.content or "", code=True))
            elif t == "InlineMath":
                self._inline_math_into(child, layout)
            elif t == "a":
                href = child.attr_text("href") or "#"
                text = child.text_content()
                lbl  = QLabel(f'<a href="{href}">{text}</a>')
                lbl.setOpenExternalLinks(True)
                lbl.setStyleSheet("color:#7c3aed;background:transparent;")
                layout.addWidget(lbl)
            elif t == "br":
                spacer = QWidget()
                spacer.setFixedSize(9999, 0)
                layout.addWidget(spacer)
            elif child.children:
                self._inline_into(child.children, layout, bold, italic, strike)

    def _inline_math_into(self, node: Any, layout: FlowLayout):
        """
        Inline math rendering → always Unicode.
        matplotlib is not suited to inline rendering: images do not align
        correctly in a FlowLayout and break the text flow.
        """
        latex = node.content or ""
        lbl = InlineLabel(latex_to_unicode(latex), math_inline=True)
        lbl.setToolTip(f"LaTeX : {latex}")
        layout.addWidget(lbl)

    def _block_math(self, node: Any) -> QFrame:
        frame = QFrame()
        frame.setStyleSheet(
            "QFrame{background:#faf5ff;border:1px solid #ddd6fe;"
            "border-radius:8px;margin:4px 0;}"
        )
        layout = QVBoxLayout(frame)
        layout.setContentsMargins(16, 12, 16, 12)
        layout.setAlignment(Qt.AlignCenter)

        latex = node.text_content() or ""

        pixmap: Optional[QPixmap] = None
        if latex_to_pixmap_available():
            pixmap = latex_to_pixmap(
                latex,
                font_size=self.math_font_size,
                dpi=150,
                bg="#faf5ff",
                fg="#3b0764",
            )

        if pixmap and not pixmap.isNull():
            lbl = QLabel()
            lbl.setPixmap(pixmap)
            lbl.setFixedSize(pixmap.width(), pixmap.height())
            lbl.setAlignment(Qt.AlignCenter)
            lbl.setStyleSheet("background:transparent;border:none;")
            lbl.setToolTip(f"LaTeX : {latex}")
            layout.addWidget(lbl)
            frame.setMinimumHeight(pixmap.height() + 24)
        else:
            unicode_text = latex_to_unicode(latex)

            lbl_unicode = QLabel(unicode_text)
            font_u = QFont("Segoe UI", self.math_font_size)
            font_u.setItalic(True)
            lbl_unicode.setFont(font_u)
            lbl_unicode.setAlignment(Qt.AlignCenter)
            lbl_unicode.setWordWrap(True)
            lbl_unicode.setStyleSheet("color:#5b21b6;background:transparent;border:none;")
            lbl_unicode.setToolTip(f"LaTeX : {latex}")
            layout.addWidget(lbl_unicode)

            lbl_src = QLabel(latex)
            font_s = QFont("JetBrains Mono", 9)
            lbl_src.setFont(font_s)
            lbl_src.setAlignment(Qt.AlignCenter)
            lbl_src.setWordWrap(True)
            lbl_src.setStyleSheet(
                "color:#a78bfa;background:transparent;border:none;"
                "font-size:9pt;margin-top:2px;"
            )
            layout.addWidget(lbl_src)

        return frame

    def _inline_math_as_block(self, node: Any) -> QWidget:
        """InlineMath encountered at block level (outside <p>) — centered."""
        w = QWidget()
        w.setStyleSheet("background:transparent;")
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 4, 0, 4)
        layout.setAlignment(Qt.AlignCenter)
        lbl = InlineLabel(latex_to_unicode(node.text_content() or ""), math_inline=True)
        lbl.setAlignment(Qt.AlignCenter)
        lbl.setToolTip(f"LaTeX : {node.text_content()}")
        layout.addWidget(lbl)
        return w

    def _list(self, node: Any, ordered: bool) -> QWidget:
        w = QWidget()
        w.setStyleSheet("background:transparent;")
        layout = QVBoxLayout(w)
        layout.setContentsMargins(16, 0, 0, 0)
        layout.setSpacing(3)
        for i, item in enumerate(n for n in node.children if n.node_type == "li"):
            row = QWidget()
            row.setStyleSheet("background:transparent;")
            rl  = QHBoxLayout(row)
            rl.setContentsMargins(0, 0, 0, 0)
            rl.setSpacing(6)
            rl.setAlignment(Qt.AlignTop)

            bullet = QLabel(f"{i+1}." if ordered else "•")
            bullet.setFixedWidth(20)
            bullet.setAlignment(Qt.AlignTop | Qt.AlignRight)
            bullet.setStyleSheet("color:#9ca3af;background:transparent;font-size:11px;")

            content = QWidget()
            content.setStyleSheet("background:transparent;")
            cl = FlowLayout(content, h_spacing=2, v_spacing=2)
            content.setLayout(cl)
            inline_children = [c for c in item.children
                                if c.node_type not in ("ul", "ol")]
            self._inline_into(inline_children, cl)

            rl.addWidget(bullet)
            rl.addWidget(content, 1)
            layout.addWidget(row)

            for child in item.children:
                if child.node_type in ("ul", "ol"):
                    sub = self._list(child, child.node_type == "ol")
                    wrapper = QWidget()
                    wrapper.setStyleSheet("background:transparent;")
                    wl = QHBoxLayout(wrapper)
                    wl.setContentsMargins(24, 0, 0, 0)
                    wl.addWidget(sub)
                    layout.addWidget(wrapper)
        return w

    def _blockquote(self, node: Any) -> QFrame:
        frame = QFrame()
        frame.setStyleSheet(
            "QFrame{background:#f9fafb;border-left:4px solid #d1d5db;"
            "border-radius:0 4px 4px 0;margin:2px 0;}"
        )
        layout = QVBoxLayout(frame)
        layout.setContentsMargins(12, 8, 12, 8)
        layout.setSpacing(4)
        for child in node.children:
            w = self._node(child)
            if w:
                layout.addWidget(w)
        return frame

    def _code_block(self, node: Any) -> QFrame:
        frame = QFrame()
        frame.setStyleSheet(
            "QFrame{background:#1e293b;border-radius:6px;margin:2px 0;}"
        )
        layout = QVBoxLayout(frame)
        layout.setContentsMargins(16, 12, 16, 12)
        lbl = QLabel(node.content or node.text_content())
        lbl.setFont(QFont("JetBrains Mono", 10))
        lbl.setStyleSheet("color:#e2e8f0;background:transparent;")
        lbl.setTextInteractionFlags(Qt.TextSelectableByMouse)
        lbl.setWordWrap(True)
        layout.addWidget(lbl)
        return frame

    def _separator(self, node=None) -> QFrame:
        line = QFrame()
        line.setFrameShape(QFrame.HLine)
        line.setFixedHeight(1)
        line.setStyleSheet("background:#e5e7eb;border:none;margin:4px 0;")
        return line

    def _table(self, node: Any) -> QFrame:
        from PyQt5.QtWidgets import QGridLayout
        frame = QFrame()
        frame.setStyleSheet(
            "QFrame{border:1px solid #e5e7eb;border-radius:6px;"
            "background:white;margin:2px 0;}"
        )
        grid = QGridLayout(frame)
        grid.setSpacing(0)
        grid.setContentsMargins(0, 0, 0, 0)
        row_idx = 0
        for section in node.children:
            rows = section.children if section.node_type in ("thead","tbody") else [section]
            for tr in rows:
                if tr.node_type != "tr":
                    continue
                is_header = section.node_type == "thead"
                for col_idx, td in enumerate(tr.children):
                    cell = QLabel(td.text_content())
                    cell.setContentsMargins(8, 6, 8, 6)
                    cell.setTextInteractionFlags(Qt.TextSelectableByMouse)
                    if is_header:
                        font = QFont("Segoe UI", 10); font.setBold(True)
                        cell.setFont(font)
                        cell.setStyleSheet(
                            "background:#f9fafb;color:#111827;"
                            "border-bottom:1px solid #e5e7eb;"
                        )
                    else:
                        cell.setStyleSheet(
                            "background:white;color:#374151;"
                            "border-bottom:1px solid #f3f4f6;"
                        )
                    grid.addWidget(cell, row_idx, col_idx)
                row_idx += 1
        return frame

    def _component(self, node: Any) -> QFrame:
        fn = self.components.get(node.node_type)
        if fn:
            return fn(node, self)

        if node.node_type == "Note":
            return self._note(node)
        if node.node_type == "Details":
            return self._details(node)
        
        frame = QFrame()
        frame.setStyleSheet(
            "QFrame{background:#f5f3ff;border:1px solid #ddd6fe;"
            "border-radius:6px;margin:2px 0;}"
        )
        layout = QVBoxLayout(frame)
        layout.setContentsMargins(12, 8, 12, 8)
        layout.setSpacing(4)
        header = QLabel(f"⬡ {node.node_type}")
        font = QFont("Segoe UI", 10); font.setBold(True)
        header.setFont(font)
        header.setStyleSheet("color:#6d28d9;background:transparent;")
        layout.addWidget(header)
        for w in self._render_mixed_children(node.children):
            layout.addWidget(w)
        return frame

    def _render_mixed_children(self, children: List[Any]) -> List[QWidget]:
        """
        Returns a list of children that can mix block and inline nodes.

        Consecutive inline nodes (text, InlineMath, strong, em, code, a)
        are grouped together in a common FlowLayout (like an implicit <p>).
        Block nodes (p, h1-h6, BlockMath, ul, ol, ...) are rendered
        individually via _node().
        """
        _INLINE_TYPES = {"text", "InlineMath", "strong", "em", "del",
                         "code", "a", "br"}

        widgets: List[QWidget] = []
        inline_buf: List[Any] = []

        def flush_inline():
            if not inline_buf:
                return
            w = QWidget()
            w.setStyleSheet("background:transparent;")
            fl = FlowLayout(w, h_spacing=2, v_spacing=2)
            w.setLayout(fl)
            self._inline_into(inline_buf, fl)
            widgets.append(w)
            inline_buf.clear()

        for child in children:
            if child.node_type in _INLINE_TYPES:
                inline_buf.append(child)
            else:
                flush_inline()
                w = self._node(child)
                if w:
                    widgets.append(w)

        flush_inline()
        return widgets

    def _note(self, node: Any) -> QFrame:
        kind  = node.attr_text("type") or "info"
        title = node.attr_text("title") or ""
        styles = {
            "info":    ("#eff6ff", "#3b82f6", "#1d4ed8"),
            "warning": ("#fffbeb", "#f59e0b", "#92400e"),
            "error":   ("#fef2f2", "#ef4444", "#991b1b"),
            "success": ("#f0fdf4", "#22c55e", "#14532d"),
        }
        icons = {"info":"ℹ️","warning":"⚠️","error":"🚨","success":"✅"}
        bg, border, color = styles.get(kind, styles["info"])
        frame = QFrame()
        frame.setStyleSheet(
            f"QFrame{{background:{bg};border-left:4px solid {border};"
            f"border-radius:0 6px 6px 0;margin:2px 0;}}"
        )
        layout = QVBoxLayout(frame)
        layout.setContentsMargins(12, 10, 12, 10)
        layout.setSpacing(4)
        if title:
            lbl  = QLabel(f"{icons.get(kind,'ℹ️')}  {title}")
            font = QFont("Segoe UI", 10); font.setBold(True)
            lbl.setFont(font)
            lbl.setStyleSheet(f"color:{color};background:transparent;")
            layout.addWidget(lbl)
        for w in self._render_mixed_children(node.children):
            layout.addWidget(w)
        return frame

    def _details(self, node: Any) -> QFrame:
        title = node.attr_text("title") or "Détails"
        outer = QFrame()
        outer.setStyleSheet(
            "QFrame{background:white;border:1px solid #e5e7eb;"
            "border-radius:6px;margin:2px 0;}"
        )
        outer_layout = QVBoxLayout(outer)
        outer_layout.setContentsMargins(0, 0, 0, 0)
        outer_layout.setSpacing(0)

        btn = QPushButton(f"▶  {title}")
        btn.setCheckable(True)
        btn.setCursor(Qt.PointingHandCursor)
        btn.setStyleSheet("""
            QPushButton{background:#f9fafb;border:none;border-radius:6px 6px 0 0;
                        padding:10px 16px;text-align:left;font-size:10pt;
                        font-weight:600;color:#374151;}
            QPushButton:hover{background:#f3f4f6;}
            QPushButton:checked{background:#f3f4f6;
                                border-bottom:1px solid #e5e7eb;}
        """)

        body = QWidget()
        body.setStyleSheet("background:white;")
        bl = QVBoxLayout(body)
        bl.setContentsMargins(16, 10, 16, 10)
        bl.setSpacing(4)
        for w in self._render_mixed_children(node.children):
            bl.addWidget(w)
        body.setVisible(False)

        def toggle(checked):
            body.setVisible(checked)
            btn.setText(f"{'▼' if checked else '▶'}  {title}")

        btn.toggled.connect(toggle)
        outer_layout.addWidget(btn)
        outer_layout.addWidget(body)
        return outer