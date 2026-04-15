"""
omni_mdx.math_render — LaTeX rendering without a browser.

InlineMath  → Unicode via symbol table.
BlockMath   → QPixmap via matplotlib.mathtext (two passes).
              Clean fallback if the formula is too complex for math text.
"""

from __future__ import annotations
import re

# Tables mapping common LaTeX commands to their Unicode equivalents
_GREEK = {
    r"\alpha":"α", r"\beta":"β", r"\gamma":"γ", r"\delta":"δ",
    r"\epsilon":"ε", r"\varepsilon":"ε", r"\zeta":"ζ", r"\eta":"η",
    r"\theta":"θ", r"\vartheta":"ϑ", r"\iota":"ι", r"\kappa":"κ",
    r"\lambda":"λ", r"\mu":"μ", r"\nu":"ν", r"\xi":"ξ",
    r"\pi":"π", r"\varpi":"ϖ", r"\rho":"ρ", r"\varrho":"ϱ",
    r"\sigma":"σ", r"\varsigma":"ς", r"\tau":"τ", r"\upsilon":"υ",
    r"\phi":"φ", r"\varphi":"φ", r"\chi":"χ", r"\psi":"ψ", r"\omega":"ω",
    r"\Gamma":"Γ", r"\Delta":"Δ", r"\Theta":"Θ", r"\Lambda":"Λ",
    r"\Xi":"Ξ", r"\Pi":"Π", r"\Sigma":"Σ", r"\Upsilon":"Υ",
    r"\Phi":"Φ", r"\Psi":"Ψ", r"\Omega":"Ω",
}
_OPERATORS = {
    r"\times":"×", r"\div":"÷", r"\pm":"±", r"\mp":"∓",
    r"\cdot":"·", r"\cdots":"⋯", r"\ldots":"…", r"\vdots":"⋮",
    r"\leq":"≤", r"\geq":"≥", r"\neq":"≠", r"\approx":"≈",
    r"\equiv":"≡", r"\sim":"∼", r"\simeq":"≃", r"\cong":"≅",
    r"\propto":"∝", r"\infty":"∞", r"\partial":"∂", r"\nabla":"∇",
    r"\forall":"∀", r"\exists":"∃", r"\in":"∈", r"\notin":"∉",
    r"\subset":"⊂", r"\supset":"⊃", r"\cup":"∪", r"\cap":"∩",
    r"\emptyset":"∅", r"\varnothing":"∅",
    r"\rightarrow":"→", r"\leftarrow":"←", r"\leftrightarrow":"↔",
    r"\Rightarrow":"⇒", r"\Leftarrow":"⇐", r"\Leftrightarrow":"⇔",
    r"\to":"→", r"\gets":"←",
    r"\sum":"∑", r"\prod":"∏", r"\int":"∫", r"\oint":"∮",
    r"\sqrt":"√",
}
_SUPERSCRIPTS = {
    "0":"⁰","1":"¹","2":"²","3":"³","4":"⁴","5":"⁵","6":"⁶","7":"⁷","8":"⁸","9":"⁹",
    "n":"ⁿ","i":"ⁱ","+":"⁺","-":"⁻","=":"⁼","(":"⁽",")":"⁾",
}
_SUBSCRIPTS = {
    "0":"₀","1":"₁","2":"₂","3":"₃","4":"₄","5":"₅","6":"₆","7":"₇","8":"₈","9":"₉",
    "a":"ₐ","e":"ₑ","i":"ᵢ","j":"ⱼ","k":"ₖ","m":"ₘ","n":"ₙ","o":"ₒ","p":"ₚ",
    "r":"ᵣ","s":"ₛ","t":"ₜ","u":"ᵤ","v":"ᵥ","x":"ₓ","+":"₊","-":"₋","=":"₌",
}
_ALL_SYMBOLS = {**_GREEK, **_OPERATORS}


def _convert_frac(s: str) -> str:
    """Internal helper to convert \\frac{a}{b} into (a/b)."""
    return re.sub(r'\\frac\{([^}]*)\}\{([^}]*)\}',
                  lambda m: f"({m.group(1).strip()}/{m.group(2).strip()})", s)

def _convert_scripts(s: str) -> str:
    """Internal helper to replace ^ and _ patterns with Unicode super/subscript chars."""
    s = re.sub(r'\^\{([^}]*)\}', lambda m: "".join(_SUPERSCRIPTS.get(c,c) for c in m.group(1)), s)
    s = re.sub(r'_\{([^}]*)\}',  lambda m: "".join(_SUBSCRIPTS.get(c,c)  for c in m.group(1)), s)
    s = re.sub(r'\^(.)', lambda m: _SUPERSCRIPTS.get(m.group(1), f"^{m.group(1)}"), s)
    s = re.sub(r'_(.)',  lambda m: _SUBSCRIPTS.get(m.group(1),   f"_{m.group(1)}"), s)
    return s

def _convert_commands(s: str) -> str:
    """Internal helper to replace LaTeX commands (like \\alpha) with their Unicode symbols."""
    for cmd, sym in sorted(_ALL_SYMBOLS.items(), key=lambda x: -len(x[0])):
        s = s.replace(cmd, sym)
    return s

def latex_to_unicode(latex: str) -> str:
    """
    Main utility to convert a LaTeX string into readable Unicode text.
    Handles fractions, Greek letters, operators, and scripts.
    """
    s = latex.strip()
    s = _convert_frac(s)
    s = _convert_commands(s)
    s = _convert_scripts(s)
    s = s.replace("{", "").replace("}", "")
    return re.sub(r' {2,}', ' ', s).strip()

def latex_to_pixmap_available() -> bool:
    """Checks if the matplotlib library is available in the current environment."""
    try:
        import matplotlib  # noqa
        return True
    except ImportError:
        return False


def latex_to_pixmap(latex: str, font_size: int = 14, dpi: int = 150,
                    bg: str = "#faf5ff", fg: str = "#3b0764"):
    """
    Renders a LaTeX formula as a high-quality QPixmap using matplotlib.mathtext.

    This function performs a two-pass render:
    1. A 'probe' pass to determine the exact bounding box of the rendered text.
    2. A 'final' pass on a precisely sized canvas to avoid excessive whitespace.

    Returns:
        QPixmap: The rendered image if successful.
        None: If matplotlib is missing or the formula is syntactically invalid.
    """
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    latex_clean = latex.strip()
    if not latex_clean.startswith("$"):
        latex_clean = f"${latex_clean}$"

    fig_probe = None
    fig       = None

    try:
        import io
        from PyQt5.QtGui import QPixmap, QImage

        fig_probe, ax_probe = plt.subplots(figsize=(10, 2), dpi=dpi)
        ax_probe.set_axis_off()
        txt = ax_probe.text(
            0.5, 0.5, latex_clean,
            fontsize=font_size, color=fg,
            ha="center", va="center",
            transform=ax_probe.transAxes,
            usetex=False,
        )

        fig_probe.canvas.draw()
        bbox_px = txt.get_window_extent(renderer=fig_probe.canvas.get_renderer())

        pad_px = 20
        fig_w  = (bbox_px.width  + pad_px * 2) / dpi
        fig_h  = (bbox_px.height + pad_px * 2) / dpi

        fig, ax = plt.subplots(figsize=(max(fig_w, 0.5), max(fig_h, 0.3)), dpi=dpi)
        ax.set_axis_off()
        fig.patch.set_facecolor(bg)
        ax.set_facecolor(bg)
        ax.text(
            0.5, 0.5, latex_clean,
            fontsize=font_size, color=fg,
            ha="center", va="center",
            transform=ax.transAxes,
            usetex=False,
        )

        buf = io.BytesIO()
        fig.savefig(buf, format="png", dpi=dpi,
                    bbox_inches="tight", pad_inches=0.05,
                    facecolor=bg)

        buf.seek(0)
        qimage = QImage.fromData(buf.read())
        return QPixmap.fromImage(qimage) if not qimage.isNull() else None

    except Exception:
        return None

    finally:
        if fig_probe is not None:
            plt.close(fig_probe)
        if fig is not None:
            plt.close(fig)