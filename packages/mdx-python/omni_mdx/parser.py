from __future__ import annotations
import json
from typing import List, Dict, Any, Optional
from .core_interface import CoreInterface
from .exceptions import MDXSyntaxError

class MdxNode:
    """Représente un nœud AST, compatible avec les dictionnaires et les objets natifs Rust."""
    def __init__(self, data: Any):
        # Helper pour lire une propriété de manière sécurisée (Dict ou Objet)
        def _safe_get(key: str, default: Any = None):
            if isinstance(data, dict):
                return data.get(key, default)
            return getattr(data, key, default)

        self.node_type = _safe_get("node_type", "")
        self.content = _safe_get("content")
        self.self_closing = _safe_get("self_closing", False)
        
        # Récursion sur les enfants
        raw_children = _safe_get("children", [])
        self.children = [MdxNode(c) for c in raw_children]
        
        # Identification des composants (commencent par une Majuscule)
        self.is_component = bool(self.node_type and self.node_type[0].isupper())

        # Traitement des attributs
        self.attributes = {}
        raw_attrs = _safe_get("attributes", {})
        
        if isinstance(raw_attrs, str):
            try:
                raw_attrs = json.loads(raw_attrs)
            except json.JSONDecodeError:
                raw_attrs = {}

        # Normalisation des attributs (kind/value -> valeur simple)
        if isinstance(raw_attrs, dict):
            for k, v in raw_attrs.items():
                if isinstance(v, dict) and "kind" in v:
                    self.attributes[k] = True if v["kind"] == "boolean" else v.get("value")
                else:
                    self.attributes[k] = v
        elif raw_attrs is not None:
            try:
                self.attributes = dict(raw_attrs)
            except (TypeError, ValueError):
                pass

    def text_content(self) -> str:
        """Extrait récursivement le contenu textuel."""
        if self.content is not None:
            return str(self.content)
        return "".join(c.text_content() for c in self.children)

    def attr_text(self, key: str) -> Optional[str]:
        """Retourne un attribut sous forme de chaîne."""
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
    """Conteneur racine de l'AST."""
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
        
        # Normalisation sécurisée du format de sortie
        if isinstance(raw_result, str):
            if not raw_result.strip():
                parsed_data = []
            else:
                try:
                    parsed_data = json.loads(raw_result)
                except json.JSONDecodeError as e:
                    # Correction ici : on lève MDXSyntaxError pour passer le test
                    raise MDXSyntaxError(f"Invalid JSON from Rust core: {e}")
        else:
            parsed_data = raw_result or []

        # Déballage automatique du nœud fragment/root si nécessaire pour la CI
        if not isinstance(parsed_data, list) and hasattr(parsed_data, 'children'):
            nodes = [MdxNode(c) for c in parsed_data.children]
        elif isinstance(parsed_data, list):
            nodes = [MdxNode(n) for n in parsed_data]
        else:
            nodes = [MdxNode(parsed_data)] if parsed_data else []

        # Gestion du cas vide (fragment sans contenu)
        if len(nodes) == 1 and not nodes[0].node_type and not nodes[0].children and not nodes[0].content:
            nodes = []

        return MdxAstRoot(nodes)