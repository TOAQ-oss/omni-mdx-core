from __future__ import annotations
import json
from typing import List, Dict, Any, Optional
from .core_interface import CoreInterface
from .exceptions import MDXSyntaxError

class MdxNode:
    """Represents a single node in the MDX Abstract Syntax Tree."""
    def __init__(self, data: Dict[str, Any]):
        self.node_type = data.get("node_type", "")
        self.content = data.get("content")
        self.children = [MdxNode(c) for c in data.get("children", [])]
        self.self_closing = data.get("self_closing", False)
        self.is_component = bool(self.node_type and self.node_type[0].isupper())
        self.attributes = {}
        
        raw_attrs = data.get("attributes", {})
        
        if isinstance(raw_attrs, str):
            try:
                raw_attrs = json.loads(raw_attrs)
            except json.JSONDecodeError:
                raw_attrs = {}

        if isinstance(raw_attrs, dict):
            for k, v in raw_attrs.items():
                if isinstance(v, dict) and "kind" in v:
                    if v["kind"] == "boolean":
                        self.attributes[k] = True
                    else:
                        self.attributes[k] = v.get("value")
                else:
                    self.attributes[k] = v

    def text_content(self) -> str:
        """Recursively extracts plain text from this node and all its children."""
        if self.content is not None:
            return str(self.content)
        return "".join(c.text_content() for c in self.children)

    def attr_text(self, key: str) -> Optional[str]:
        """Returns the value of an attribute as a string."""
        val = self.attributes.get(key)
        return str(val) if val is not None else None

    def find(self, tag: str) -> Optional['MdxNode']:
        """Finds the first descendant node matching the given tag."""
        if self.node_type == tag:
            return self
        for child in self.children:
            found = child.find(tag)
            if found:
                return found
        return None

    def find_all(self, tag: str) -> List['MdxNode']:
        """Finds all descendant nodes matching the given tag."""
        results = []
        if self.node_type == tag:
            results.append(self)
        for child in self.children:
            results.extend(child.find_all(tag))
        return results

class MdxAstRoot:
    """Wrapper for the root of the parsed AST."""
    def __init__(self, nodes: List[MdxNode]):
        self.nodes = nodes
        self.length = len(nodes)

class MDXParser:
    """
    The main coordinator for MDX parsing operations.
    Transforms raw MDX text into navigable Python objects.
    """
    def __init__(self):
        self._interface = CoreInterface()

    def parse(self, mdx_text: str) -> MdxAstRoot:
        if not isinstance(mdx_text, str):
            raise TypeError("Input must be a string")
            
        raw_result = self._interface.parse(mdx_text)

        if isinstance(raw_result, str):
            if not raw_result.strip():
                parsed_data = []
            else:
                try:
                    parsed_data = json.loads(raw_result)
                except json.JSONDecodeError as e:
                    raise MDXSyntaxError(f"Invalid JSON from Rust core: {e}")
        else:
            parsed_data = raw_result or []

        if not isinstance(parsed_data, list):
            parsed_data = [parsed_data]

        nodes = [MdxNode(n) for n in parsed_data]
        return MdxAstRoot(nodes)