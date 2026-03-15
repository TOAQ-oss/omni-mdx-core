use pyo3::prelude::*;
use crate::parser::parse_mdx;

#[pyfunction]
#[pyo3(name = "parse")]
fn py_parse_mdx(input: String) -> PyResult<String> {
    match parse_mdx(&input) {
        Ok(ast) => serde_json::to_string(&ast)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string())),
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())),
    }
}

#[pymodule]
fn omni_mdx_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_parse_mdx, m)?)?;
    Ok(())
}