use crate::ast::{AstNode, AttrValue};
use crate::parser::parse_mdx;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Arc;

#[pyclass(name = "MdxNode", module = "omni_mdx_core")]
pub struct PyMdxNode {
    ast: Arc<Vec<AstNode<'static>>>,
    path: Vec<usize>,
}

impl PyMdxNode {
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

    fn child_node(&self, child_index: usize) -> PyMdxNode {
        let mut child_path = self.path.clone();
        child_path.push(child_index);
        PyMdxNode {
            ast: Arc::clone(&self.ast),
            path: child_path,
        }
    }

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
    #[getter]
    pub fn node_type(&self) -> String {
        self.resolve().map(|n| n.node_type.to_string()).unwrap_or_default()
    }

    #[getter]
    pub fn content(&self) -> Option<String> {
        self.resolve().and_then(|n| n.content.as_ref().map(|c| c.to_string()))
    }

    #[getter]
    pub fn self_closing(&self) -> bool {
        self.resolve().map(|n| n.self_closing).unwrap_or(false)
    }

    #[getter]
    pub fn children(&self) -> Vec<PyMdxNode> {
        let count = self.resolve().map(|n| n.children.len()).unwrap_or(0);
        (0..count).map(|i| self.child_node(i)).collect()
    }

    #[getter]
    pub fn is_component(&self) -> bool {
        self.node_type().chars().next().map_or(false, |c| c.is_ascii_uppercase())
    }

    pub fn text_content(&self) -> String {
        let mut buf = String::new();
        self.extract_text_recursive(&mut buf);
        buf
    }

    pub fn attr_text(&self, name: &str) -> Option<String> {
        let attrs = self.resolve()?.attributes.as_ref()?;
        match attrs.get(name)? {
            AttrValue::Text(s) => Some(s.to_string()),
            AttrValue::Expression(s) => Some(s.to_string()),
            _ => None,
        }
    }

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

fn child_node_clone(node: &PyMdxNode) -> PyMdxNode {
    PyMdxNode {
        ast: Arc::clone(&node.ast),
        path: node.path.clone(),
    }
}

#[pyclass(name = "MdxAst", module = "omni_mdx_core")]
pub struct PyMdxAst {
    inner: Arc<Vec<AstNode<'static>>>,
}

#[pymethods]
impl PyMdxAst {
    #[getter]
    pub fn length(&self) -> usize {
        self.inner.len()
    }

    #[getter]
    pub fn nodes(&self) -> Vec<PyMdxNode> {
        (0..self.inner.len())
            .map(|i| PyMdxNode { ast: Arc::clone(&self.inner), path: vec![i] })
            .collect()
    }
}

#[pyfunction]
#[pyo3(name = "parse")]
fn py_parse_mdx(input: String) -> PyResult<PyMdxAst> {
    let ast = parse_mdx(&input)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    
    let static_ast: Vec<AstNode<'static>> = ast.into_iter().map(|n| n.into_static()).collect();
    Ok(PyMdxAst { inner: Arc::new(static_ast) })
}

#[pymodule]
#[pyo3(name = "omni_mdx_core")]
fn omni_mdx_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMdxNode>()?;
    m.add_class::<PyMdxAst>()?;
    m.add_function(wrap_pyfunction!(py_parse_mdx, m)?)?;
    Ok(())
}