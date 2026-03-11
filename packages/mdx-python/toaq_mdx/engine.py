from .core_interface import CoreInterface

class OmniMDX:
    def __init__(self, components=None):
        self.components = components or {}
        self._interface = CoreInterface()

    def parse_to_ast(self, mdx_text: str):
        return self._interface.call_rust_parser(mdx_text)

    def render(self, mdx_text: str):
        ast = self.parse_to_ast(mdx_text)
        return self._traverse(ast)

    def _traverse(self, node):
        if isinstance(node, list):
            return [self._traverse(n) for n in node]
        
        node_type = node.get("node_type")
        
        if node_type == "Jsx":
            tag = node.get("name")
            if tag in self.components:
                props = node.get("props", {})
                children = self._traverse(node.get("children", []))
                return self.components[tag](props, children)
        
        if node_type == "text":
            return node.get("content", "")

        if "children" in node:
            node["children"] = self._traverse(node["children"])
        return node