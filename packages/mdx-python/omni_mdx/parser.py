from __future__ import annotations
import json
from typing import List, Dict, Any, Optional
from .core_interface import CoreInterface
from .exceptions import MDXSyntaxError

class MdxNode:
    """Wrapper universel pour les nœuds AST (Dictionnaires ou Objets natifs Rust)."""
    def __init__(self, data: Any):
        # Helper pour lire une propriété (attribut d'objet ou clé de dictionnaire)
        def _get_val(key: str, default: Any = None):
            if isinstance(data, dict):
                return data.get(key, default)
            return getattr(data, key, default)

        self.node_type = _get_val("node_type", "")
        self.content = _get_val("content")
        self.self_closing = _get_val("self_closing", False)
        
        # Récursion sur les enfants
        raw_children = _get_val("children", [])
        self.children = [MdxNode(c) for c in raw_children]
        
        # Un composant commence par une Majuscule
        self.is_component = bool(self.node_type and self.node_type[0].isupper())

        # Traitement des attributs
        self.attributes = {}
        raw_attrs = _get_val("attributes", {})
        
        if isinstance(raw_attrs, str):
            try:
                raw_attrs = json.loads(raw_attrs)
            except json.JSONDecodeError:
                raw_attrs = {}

        if isinstance(raw_attrs, dict):
            for k, v in raw_attrs.items():
                if isinstance(v, dict) and "kind" in v:
                    self.attributes[k] = True if v["kind"] == "boolean" else v.get("value")
                else:
                    self.attributes[k] = v
        elif raw_attrs is not None:
            try:
                # Tentative de conversion si c'est une Map native Rust
                self.attributes = dict(raw_attrs)
            except (TypeError, ValueError):
                pass

    def text_content(self) -> str:
        """Extrait récursivement le texte brut de ce nœud et de ses enfants."""
        if self.content is not None:
            return str(self.content)
        return "".join(c.text_content() for c in self.children)

    def attr_text(self, key: str) -> Optional[str]:
        """Récupère la valeur d'un attribut sous forme de chaîne."""
        val = self.attributes.get(key)
        return str(val) if val is not None else None

    def find(self, tag: str) -> Optional['MdxNode']:
        if self.node_type == tag: return self
        for child in self.children:
            found = child.find(tag)
            if found: return found
        return None

    def find_all(self, tag: str) -> List['MdxNode']:
        results = []
        if self.node_type == tag: results.append(self)
        for child in self.children:
            results.extend(child.find_all(tag))
        return results

class MdxAstRoot:
    """Conteneur pour la racine de l'AST, tel qu'attendu par les tests."""
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
        
        # Détection du format renvoyé par le noyau Rust
        if isinstance(raw_result, str):
            if not raw_result.strip():
                parsed_data = []
            else:
                try:
                    parsed_data = json.loads(raw_result)
                except json.JSONDecodeError as e:
                    # Fix pour test_json_error_handling
                    raise MDXSyntaxError(f"Invalid JSON from Rust core: {e}")
        else:
            parsed_data = raw_result or []

        # Normalisation : Unpacking du Root/Fragment natif Rust vers une liste Python
        nodes = []
        # On vérifie si l'objet natif a une propriété .nodes ou .children
        source = getattr(parsed_data, "nodes", getattr(parsed_data, "children", None))
        
        if source is not None:
            nodes = [MdxNode(n) for n in source]
        elif isinstance(parsed_data, list):
            nodes = [MdxNode(n) for n in parsed_data]
        elif parsed_data:
            nodes = [MdxNode(parsed_data)]

        # Cas spécifique : si Rust renvoie un nœud racine vide
        if len(nodes) == 1 and not nodes[0].node_type and not nodes[0].children and not nodes[0].content:
            nodes = []

        return MdxAstRoot(nodes)