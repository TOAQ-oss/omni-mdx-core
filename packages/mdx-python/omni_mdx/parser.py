from __future__ import annotations
import json
from typing import List, Dict, Any, Optional
from .core_interface import CoreInterface
from .exceptions import MDXSyntaxError

class MdxNode:
    """Universal wrapper for nodes (Dict or Native Rust Object)."""
    def __init__(self, data: Any):
        def _get(key, default=None):
            if isinstance(data, dict):
                return data.get(key, default)
            return getattr(data, key, default)

        self.node_type = _get("node_type", "")
        self.content = _get("content")
        self.self_closing = _get("self_closing", False)
        
        raw_children = _get("children", [])
        self.children = [MdxNode(c) for c in raw_children]

        self.is_component = bool(self.node_type and self.node_type[0].isupper())

        self.attributes = {}
        raw_attrs = _get("attributes", {})
        
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
        elif raw_attrs is not None:
            try:
                self.attributes = dict(raw_attrs)
            except:
                pass

    def text_content(self) -> str:
        """Extracts the text content recursively."""
        if self.content is not None:
            return str(self.content)
        return "".join(c.text_content() for c in self.children)

    def attr_text(self, key: str) -> Optional[str]:
        """Retrieves an attribute as text."""
        val = self.attributes.get(key)
        return str(val) if val is not None else None

    def find(self, tag: str) -> Optional['MdxNode']:
        if self.node_type == tag:
            return self
        for child in self.children:
            found = child.find(tag)
            if found:
                return found
        return None

    def find_all(self, tag: str) -> List['MdxNode']:
        results = []
        if self.node_type == tag:
            results.append(self)
        for child in self.children:
            results.extend(child.find_all(tag))
        return results

class MdxAstRoot:
    """Wrapper for the AST root."""
    def __init__(self, nodes: List[MdxNode]):
        self.nodes = nodes
        self.length = len(nodes)

class MDXParser:
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

        if not isinstance(parsed_data, list) and not hasattr(parsed_data, "__iter__"):
            parsed_data = [parsed_data]

        nodes = [MdxNode(n) for n in parsed_data]
        return MdxAstRoot(nodes)