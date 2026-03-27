from .core_interface import CoreInterface

class OmniMDX:
    def __init__(self, components=None):
        self.components = components or {}
        self._interface = CoreInterface()

    def parse_to_ast(self, mdx_text: str):
        return self._interface.parse(mdx_text).nodes

    def render(self, mdx_text: str):
        nodes = self.parse_to_ast(mdx_text)
        return self._traverse(nodes)

    def _traverse(self, node):
        if isinstance(node, list):
            return [self._traverse(n) for n in node]
        
        if node.is_component:
            tag = node.node_type
            if tag in self.components:
                props = node.attributes or {}
                children = self._traverse(node.children)
                return self.components[tag](props, children)
        
        if node.node_type == "text":
            return node.content or ""

        children_rendered = self._traverse(node.children)
        return {"type": node.node_type, "children": children_rendered, "props": node.attributes}