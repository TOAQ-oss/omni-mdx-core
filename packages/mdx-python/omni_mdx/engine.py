from .parser import MDXParser

class OmniMDX:
    """
    The primary API class for integrating Omni-MDX into Python applications.
    
    It coordinates the native parsing engine and provides a recursive traversal 
    mechanism to transform AST nodes into Python-native structures or 
    custom component outputs.
    """
    def __init__(self, components=None):
        """
        Initializes the Omni-MDX engine.
        
        Args:
            components: A dictionary mapping MDX component names (e.g., "MyCard") 
                        to Python functions that handle their rendering.
        """
        self.components = components or {}
        self._parser = MDXParser()

    def parse_to_ast(self, mdx_text: str):
        """
        Parses raw MDX text and returns the raw Abstract Syntax Tree nodes.
        
        Args:
            mdx_text: The MDX content to parse.
            
        Returns:
            A list of MdxNode objects representing the document structure.
        """
        return self._parser.parse(mdx_text).nodes

    def render(self, mdx_text: str):
        """
        Parses and renders MDX text using the registered Python components.
        
        This method performs a full pass: text -> AST -> Traversal -> Output.
        
        Args:
            mdx_text: The MDX content to process.
            
        Returns:
            The result of the recursive traversal (can be strings, dicts, or objects 
            depending on component implementation).
        """
        nodes = self.parse_to_ast(mdx_text)
        return self._traverse(nodes)

    def _traverse(self, node):
        """
        Internal recursive helper to walk the AST and apply rendering logic.
        
        Logic priority:
        1. If it's a list, process each element.
        2. If it's a registered custom component, call its Python function.
        3. If it's a text node, return its content.
        4. Otherwise, return a generic dictionary representation.
        """
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