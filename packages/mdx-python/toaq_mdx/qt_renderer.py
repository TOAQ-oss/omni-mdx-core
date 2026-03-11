"""
toaq_mdx.qt_renderer — Rendu AST en widgets PyQt5 natifs.

Zéro HTML produit. Chaque AstNode devient un widget Qt dédié.
Les formules mathématiques sont rendues :
  - InlineMath → texte Unicode via math_render.latex_to_unicode
  - BlockMath  → image PNG via matplotlib.mathtext (si disponible)
                 fallback sur texte Unicode sinon

Usage
-----
    from toaq_mdx.qt_renderer import QtRenderer

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

from .ast import AstNode
from .math_render import latex_to_unicode, latex_to_pixmap, latex_to_pixmap_available


# ══════════════════════════════════════════════════════════════════════════════
#  FlowLayout — word-wrap de widgets inline
# ══════════════════════════════════════════════════════════════════════════════

class FlowLayout(QLayout):
    """Dispose les widgets en lignes avec retour automatique (word-wrap)."""

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


# ══════════════════════════════════════════════════════════════════════════════
#  InlineLabel — widget texte inline stylisé
# ══════════════════════════════════════════════════════════════════════════════

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


# ══════════════════════════════════════════════════════════════════════════════
#  QtRenderer
# ══════════════════════════════════════════════════════════════════════════════

# Composants built-in — peuvent être surchargés par le développeur
_BUILTIN_COMPONENTS = {"Note", "Details"}


class QtRenderer:
    """
    Convertit une liste d'AstNode en widgets Qt natifs.

    Priorité de rendu pour un composant JSX :
    1. Composant enregistré par le développeur  (via ``components=`` ou ``register()``)
    2. Composant built-in                       (Note, Details)
    3. Fallback générique                       (encadré violet avec le nom)

    Parameters
    ----------
    components : dict, optional
        Registre initial de composants custom.
        Signature : ``fn(node: AstNode, renderer: QtRenderer) -> QWidget``

        Exemple — surcharger Note avec un style custom ::

            from PyQt5.QtWidgets import QFrame, QVBoxLayout, QLabel

            def my_note(node, renderer):
                frame = QFrame()
                frame.setStyleSheet("border: 2px solid red; border-radius: 8px;")
                layout = QVBoxLayout(frame)
                title = node.attr_text("title") or ""
                layout.addWidget(QLabel(f"⚠ {title}"))
                for w in renderer._render_mixed_children(node.children):
                    layout.addWidget(w)
                return frame

            renderer = QtRenderer(components={"Note": my_note})

        Le composant reçoit :
        - ``node``     : l'AstNode complet avec ``node.attributes``, ``node.children``
        - ``renderer`` : le QtRenderer courant — utilisez ``renderer._render_mixed_children()``
          pour rendre les enfants, ou ``renderer.render()`` pour un sous-arbre complet.

    math_font_size : int
        Taille de police (pt) pour le rendu image des formules block math.
    """

    def __init__(self, components: Optional[Dict[str, Callable]] = None,
                 math_font_size: int = 14):
        # Copie défensive — on ne modifie pas le dict passé par l'appelant
        self.components:     Dict[str, Callable] = dict(components or {})
        self.math_font_size: int                 = math_font_size

    # ── Registre de composants ────────────────────────────────────────────────

    def register(self, name: str, fn: Callable) -> "QtRenderer":
        """
        Enregistre (ou remplace) un composant.

        Peut surcharger un built-in (Note, Details) ou définir un nouveau.
        Retourne ``self`` pour le chaînage.

        Example
        -------
        >>> renderer.register("Note", my_note_fn).register("Alert", my_alert_fn)
        """
        self.components[name] = fn
        return self

    def unregister(self, name: str) -> "QtRenderer":
        """
        Supprime un composant enregistré.

        Si ``name`` est un built-in (Note, Details), le built-in reprend le dessus.
        Retourne ``self`` pour le chaînage.
        """
        self.components.pop(name, None)
        return self

    def registered(self) -> Dict[str, Callable]:
        """Retourne une copie du registre courant (user components uniquement)."""
        return dict(self.components)

    @property
    def builtin_components(self) -> frozenset:
        """Noms des composants built-in (toujours disponibles même sans enregistrement)."""
        return frozenset(_BUILTIN_COMPONENTS)

    # ── Entrée publique ───────────────────────────────────────────────────────

    def render(self, nodes: List[AstNode], parent: Optional[QWidget] = None) -> QWidget:
        """Retourne un QWidget contenant le rendu de tous les nœuds."""
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

    # ── Dispatch ──────────────────────────────────────────────────────────────

    def _node(self, node: AstNode) -> Optional[QWidget]:
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
        # InlineMath au niveau bloc (rare mais possible) → widget centré
        if t == "InlineMath":
            return self._inline_math_as_block(node)
        if node.is_component:
            return self._component(node)
        if node.children:
            return self.render(node.children)
        return None

    # ── Headings ──────────────────────────────────────────────────────────────

    def _heading(self, node: AstNode) -> QLabel:
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

    # ── Paragraphe (flow inline) ───────────────────────────────────────────────

    def _paragraph(self, node: AstNode) -> QWidget:
        w = QWidget()
        w.setStyleSheet("background:transparent;")
        layout = FlowLayout(w, h_spacing=2, v_spacing=2)
        w.setLayout(layout)
        self._inline_into(node.children, layout)
        return w

    def _inline_into(self, children: List[AstNode], layout: FlowLayout,
                     bold=False, italic=False, strike=False):
        """Peuple un FlowLayout avec les nœuds inline."""
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

    def _inline_math_into(self, node: AstNode, layout: FlowLayout):
        """
        Rendu inline math → toujours Unicode.
        matplotlib n'est pas adapté à l'inline : les images ne s'alignent
        pas correctement dans un FlowLayout et cassent le flow de texte.
        """
        latex = node.content or ""
        lbl = InlineLabel(latex_to_unicode(latex), math_inline=True)
        lbl.setToolTip(f"LaTeX : {latex}")
        layout.addWidget(lbl)

    # ── BlockMath — image ou fallback unicode ─────────────────────────────────

    def _block_math(self, node: AstNode) -> QFrame:
        frame = QFrame()
        frame.setStyleSheet(
            "QFrame{background:#faf5ff;border:1px solid #ddd6fe;"
            "border-radius:8px;margin:4px 0;}"
        )
        layout = QVBoxLayout(frame)
        layout.setContentsMargins(16, 12, 16, 12)
        layout.setAlignment(Qt.AlignCenter)

        latex = node.content or ""

        # Tentative rendu image (matplotlib)
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
            # Fallback : unicode centré, avec le LaTeX source en monospace
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

            # Source LaTeX en petit monospace pour référence
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

    def _inline_math_as_block(self, node: AstNode) -> QWidget:
        """InlineMath rencontré au niveau bloc (hors <p>) — centré."""
        w = QWidget()
        w.setStyleSheet("background:transparent;")
        layout = QVBoxLayout(w)
        layout.setContentsMargins(0, 4, 0, 4)
        layout.setAlignment(Qt.AlignCenter)
        lbl = InlineLabel(latex_to_unicode(node.content or ""), math_inline=True)
        lbl.setAlignment(Qt.AlignCenter)
        lbl.setToolTip(f"LaTeX : {node.content}")
        layout.addWidget(lbl)
        return w

    # ── Listes ────────────────────────────────────────────────────────────────

    def _list(self, node: AstNode, ordered: bool) -> QWidget:
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
            # Filtrer les sous-listes pour l'inline
            inline_children = [c for c in item.children
                                if c.node_type not in ("ul", "ol")]
            self._inline_into(inline_children, cl)

            rl.addWidget(bullet)
            rl.addWidget(content, 1)
            layout.addWidget(row)

            # Sous-listes
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

    # ── Blockquote ────────────────────────────────────────────────────────────

    def _blockquote(self, node: AstNode) -> QFrame:
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

    # ── Code block ────────────────────────────────────────────────────────────

    def _code_block(self, node: AstNode) -> QFrame:
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

    # ── Séparateur ────────────────────────────────────────────────────────────

    def _separator(self, node=None) -> QFrame:
        line = QFrame()
        line.setFrameShape(QFrame.HLine)
        line.setFixedHeight(1)
        line.setStyleSheet("background:#e5e7eb;border:none;margin:4px 0;")
        return line

    # ── Table ─────────────────────────────────────────────────────────────────

    def _table(self, node: AstNode) -> QFrame:
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

    # ── Composants JSX custom ─────────────────────────────────────────────────

    def _component(self, node: AstNode) -> QFrame:
        # Composant enregistré
        fn = self.components.get(node.node_type)
        if fn:
            return fn(node, self)

        # Composants built-in
        if node.node_type == "Note":
            return self._note(node)
        if node.node_type == "Details":
            return self._details(node)

        # Fallback générique
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

    def _render_mixed_children(self, children: List[AstNode]) -> List[QWidget]:
        """
        Rend une liste d'enfants qui peut mélanger nœuds block et inline.

        Les nœuds inline consécutifs (text, InlineMath, strong, em, code, a)
        sont regroupés dans un FlowLayout commun (comme un <p> implicite).
        Les nœuds block (p, h1-h6, BlockMath, ul, ol, ...) sont rendus
        individuellement via _node().
        """
        _INLINE_TYPES = {"text", "InlineMath", "strong", "em", "del",
                         "code", "a", "br"}

        widgets: List[QWidget] = []
        inline_buf: List[AstNode] = []

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

    def _note(self, node: AstNode) -> QFrame:
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

    def _details(self, node: AstNode) -> QFrame:
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