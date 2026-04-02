use crate::ast::{AstNode, AttrValue};
use crate::parser::parse_mdx;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Arc;

/// Represents a single MDX Abstract Syntax Tree (AST) node exposed to Python.
///
/// To ensure memory safety and avoid deep-copying the AST when traversing,
/// nodes are resolved lazily using an `Arc` reference to the root and a navigational path.
#[pyclass(name = "MdxNode", module = "omni_mdx_core")]
pub struct PyMdxNode {
    ast: Arc<Vec<AstNode<'static>>>,
    path: Vec<usize>,
}

impl PyMdxNode {
    /// Lazily resolves the internal Rust `AstNode` based on the stored path.
    fn resolve(&self) -> Option<&AstNode<'static>> {
        if self.path.is_empty() {
            return None;
        }
        let mut nodes: &[AstNode<'static>] = &self.ast;
        let last = self.path.len() - 1;
        for &idx in &self.path[..last] {
            nodes = &nodes.get(idx)?.children;
        }
        nodes.get(self.path[last])
    }

    /// Creates a new `PyMdxNode` pointing to a specific child index.
    fn child_node(&self, child_index: usize) -> PyMdxNode {
        let mut child_path = self.path.clone();
        child_path.push(child_index);
        PyMdxNode {
            ast: Arc::clone(&self.ast),
            path: child_path,
        }
    }

    /// Internal recursive helper to extract text content.
    fn extract_text_recursive(&self, buf: &mut String) {
        if let Some(n) = self.resolve() {
            if n.node_type == "text" {
                if let Some(c) = &n.content {
                    buf.push_str(c);
                }
            }
            for i in 0..n.children.len() {
                self.child_node(i).extract_text_recursive(buf);
            }
        }
    }
}

#[pymethods]
impl PyMdxNode {
    /// The tag name or type of the node (e.g., 'p', 'div', 'text', 'Box').
    #[getter]
    pub fn node_type(&self) -> String {
        self.resolve()
            .map(|n| n.node_type.to_string())
            .unwrap_or_default()
    }

    /// The raw text content of the node, if applicable.
    /// Returns `None` if the node does not contain direct text.
    #[getter]
    pub fn content(&self) -> Option<String> {
        self.resolve()
            .and_then(|n| n.content.as_ref().map(|c| c.to_string()))
    }

    /// Returns `True` if the node is self-closing (e.g., `<img />`).
    #[getter]
    pub fn self_closing(&self) -> bool {
        self.resolve().map(|n| n.self_closing).unwrap_or(false)
    }

    /// Returns a list of the node's children as `MdxNode` objects.
    #[getter]
    pub fn children(&self) -> Vec<PyMdxNode> {
        let count = self.resolve().map(|n| n.children.len()).unwrap_or(0);
        (0..count).map(|i| self.child_node(i)).collect()
    }

    /// Returns `True` if the node represents a JSX/MDX Component (Starts with a capital letter).
    #[getter]
    pub fn is_component(&self) -> bool {
        self.node_type()
            .chars()
            .next()
            .map_or(false, |c| c.is_ascii_uppercase())
    }

    /// Recursively extracts and concatenates all text content from this node and its descendants.
    pub fn text_content(&self) -> String {
        let mut buf = String::new();
        self.extract_text_recursive(&mut buf);
        buf
    }

    /// Retrieves the string value of a specific attribute.
    /// Returns `None` if the attribute does not exist or is not a text/expression type.
    pub fn attr_text(&self, name: &str) -> Option<String> {
        let attrs = self.resolve()?.attributes.as_ref()?;
        match attrs.get(name)? {
            AttrValue::Text(s) => Some(s.to_string()),
            AttrValue::Expression(s) => Some(s.to_string()),
            _ => None,
        }
    }

    /// Returns a Python dictionary containing all attributes of the node.
    /// Expressions and text are returned as strings, booleans as bools.
    #[getter]
    pub fn attributes(&self, py: Python) -> PyResult<Option<PyObject>> {
        let attrs = match self.resolve().and_then(|n| n.attributes.as_ref()) {
            Some(a) => a,
            None => return Ok(None),
        };

        let dict = PyDict::new_bound(py);
        for (key, val) in attrs {
            let py_val = match val {
                AttrValue::Text(s) => s.to_string().into_py(py),
                AttrValue::Expression(s) => s.to_string().into_py(py),
                AttrValue::Boolean => true.into_py(py),
                AttrValue::Ast(_) => "Ast".into_py(py),
            };
            dict.set_item(key.to_string(), py_val)?;
        }
        Ok(Some(dict.into()))
    }

    /// Performs a Depth-First Search (DFS) to find the first child node matching the given type.
    pub fn find(&self, node_type: &str) -> Option<PyMdxNode> {
        for child in self.children() {
            if child.node_type() == node_type {
                return Some(child);
            }
            if let Some(found) = child.find(node_type) {
                return Some(found);
            }
        }
        None
    }

    /// Performs a Depth-First Search (DFS) and returns a list of all nodes matching the given type.
    pub fn find_all(&self, node_type: &str) -> Vec<PyMdxNode> {
        let mut results = Vec::new();
        for child in self.children() {
            if child.node_type() == node_type {
                results.push(child_node_clone(&child));
            }
            results.extend(child.find_all(node_type));
        }
        results
    }
}

/// Helper function to clone a PyMdxNode cheaply.
fn child_node_clone(node: &PyMdxNode) -> PyMdxNode {
    PyMdxNode {
        ast: Arc::clone(&node.ast),
        path: node.path.clone(),
    }
}

/// Represents the root Abstract Syntax Tree (AST) returned by the parser.
#[pyclass(name = "MdxAst", module = "omni_mdx_core")]
pub struct PyMdxAst {
    inner: Arc<Vec<AstNode<'static>>>,
}

#[pymethods]
impl PyMdxAst {
    /// The number of root nodes in the AST.
    #[getter]
    pub fn length(&self) -> usize {
        self.inner.len()
    }

    /// Returns a list of all root nodes in the AST.
    #[getter]
    pub fn nodes(&self) -> Vec<PyMdxNode> {
        (0..self.inner.len())
            .map(|i| PyMdxNode {
                ast: Arc::clone(&self.inner),
                path: vec![i],
            })
            .collect()
    }
}

/// Parses an MDX string and returns the root Abstract Syntax Tree (`MdxAst`).
///
/// Raises a `ValueError` if the MDX content is malformed.
#[pyfunction]
#[pyo3(name = "parse")]
fn py_parse_mdx(input: String) -> PyResult<PyMdxAst> {
    let ast = parse_mdx(&input)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    let static_ast: Vec<AstNode<'static>> = ast.into_iter().map(|n| n.into_static()).collect();
    Ok(PyMdxAst {
        inner: Arc::new(static_ast),
    })
}

/// The Omni-Core MDX parser module for Python.
///
/// Provides blazing fast MDX parsing with a safe, lazy-evaluated DOM API.
#[pymodule]
#[pyo3(name = "omni_mdx_core")]
fn omni_mdx_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMdxNode>()?;
    m.add_class::<PyMdxAst>()?;
    m.add_function(wrap_pyfunction!(py_parse_mdx, m)?)?;
    Ok(())
}
