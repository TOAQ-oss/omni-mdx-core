"""
toaq_mdx.ast — Python representation of the MDX AST.

AstNode and AttrValue mirror the Rust types exactly so that the JSON
produced by `omni_mdx_core.parse()` deserialises without surprises.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Union
import json



@dataclass
class AttrValue:
    """
    The value of a JSX attribute, tagged by kind:

    - ``text``       → prop="hello"
    - ``expression`` → prop={someExpr}   (raw JS string)
    - ``boolean``    → bare prop          (implicitly true)
    - ``ast``        → prop={<Component/>} (sub-tree)
    """
    kind: str                              # "text" | "expression" | "boolean" | "ast"
    value: Union[str, bool, List["AstNode"], None] = None

    @property
    def is_text(self) -> bool:
        return self.kind == "text"

    @property
    def is_expression(self) -> bool:
        return self.kind == "expression"

    @property
    def is_boolean(self) -> bool:
        return self.kind == "boolean"

    @property
    def is_ast(self) -> bool:
        return self.kind == "ast"

    @property
    def text(self) -> Optional[str]:
        """Return the string value if kind is 'text', else None."""
        return self.value if self.is_text else None

    @property
    def expression(self) -> Optional[str]:
        """Return the raw JS expression string if kind is 'expression', else None."""
        return self.value if self.is_expression else None

    @property
    def nodes(self) -> List["AstNode"]:
        """Return the sub-tree if kind is 'ast', else []."""
        return self.value if self.is_ast else []

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "AttrValue":
        kind = data.get("kind", "boolean")
        if kind == "ast":
            return cls(kind=kind, value=[AstNode.from_dict(n) for n in (data.get("value") or [])])
        return cls(kind=kind, value=data.get("value"))

    def __repr__(self) -> str:
        if self.is_boolean:
            return "AttrValue(boolean)"
        return f"AttrValue({self.kind}={self.value!r})"


@dataclass
class AstNode:
    """
    A single node in the MDX AST.

    Attributes
    ----------
    node_type : str
        Tag name or pseudo-type, e.g. ``"p"``, ``"h1"``, ``"text"``,
        ``"InlineMath"``, ``"BlockMath"``, ``"Note"``.
    content : str or None
        Raw text content for leaf text nodes.
    attributes : dict
        JSX / HTML attributes as ``{name: AttrValue}`` mapping.
    children : list[AstNode]
        Child nodes.
    self_closing : bool
        True when the tag was self-closing (``<Comp />``).
    """
    node_type: str
    content: Optional[str] = None
    attributes: Dict[str, AttrValue] = field(default_factory=dict)
    children: List["AstNode"] = field(default_factory=list)
    self_closing: bool = False

    @property
    def is_text(self) -> bool:
        return self.node_type == "text"

    @property
    def is_inline_math(self) -> bool:
        return self.node_type == "InlineMath"

    @property
    def is_block_math(self) -> bool:
        return self.node_type == "BlockMath"

    @property
    def is_component(self) -> bool:
        """True for custom JSX components (uppercase first letter)."""
        return bool(self.node_type) and self.node_type[0].isupper()

    @property
    def is_heading(self) -> bool:
        return self.node_type in ("h1", "h2", "h3", "h4", "h5", "h6")

    def attr(self, name: str) -> Optional[AttrValue]:
        """Return the AttrValue for *name*, or None if absent."""
        return self.attributes.get(name)

    def attr_text(self, name: str) -> Optional[str]:
        """Return the plain-text value of attribute *name*, or None."""
        v = self.attr(name)
        return v.text if v else None

    def find(self, node_type: str) -> Optional["AstNode"]:
        """BFS search — return the first descendant with *node_type*."""
        queue = list(self.children)
        while queue:
            node = queue.pop(0)
            if node.node_type == node_type:
                return node
            queue.extend(node.children)
        return None

    def find_all(self, node_type: str) -> List["AstNode"]:
        """Return all descendants with *node_type* (depth-first)."""
        results: List[AstNode] = []
        stack = list(self.children)
        while stack:
            node = stack.pop()
            if node.node_type == node_type:
                results.append(node)
            stack.extend(reversed(node.children))
        return results

    def text_content(self) -> str:
        """Recursively collect all text content as a single string."""
        if self.is_text:
            return self.content or ""
        return "".join(c.text_content() for c in self.children)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "AstNode":
        attrs_raw = data.get("attributes") or {}
        return cls(
            node_type=data["node_type"],
            content=data.get("content"),
            attributes={k: AttrValue.from_dict(v) for k, v in attrs_raw.items()},
            children=[cls.from_dict(c) for c in (data.get("children") or [])],
            self_closing=data.get("self_closing", False),
        )

    def to_dict(self) -> Dict[str, Any]:
        d: Dict[str, Any] = {"node_type": self.node_type}
        if self.content is not None:
            d["content"] = self.content
        if self.attributes:
            d["attributes"] = {
                k: {"kind": v.kind, **({"value": v.value} if not v.is_boolean else {})}
                for k, v in self.attributes.items()
            }
        if self.children:
            d["children"] = [c.to_dict() for c in self.children]
        if self.self_closing:
            d["self_closing"] = True
        return d

    def __repr__(self) -> str:
        parts = [f"node_type={self.node_type!r}"]
        if self.content:
            parts.append(f"content={self.content!r}")
        if self.attributes:
            parts.append(f"attributes={list(self.attributes)}")
        if self.children:
            parts.append(f"children=[{len(self.children)}]")
        return f"AstNode({', '.join(parts)})"


def parse_ast(json_str: str) -> List[AstNode]:
    """Deserialise the raw JSON string from ``omni_mdx_core.parse()``."""
    data = json.loads(json_str)
    return [AstNode.from_dict(node) for node in data]