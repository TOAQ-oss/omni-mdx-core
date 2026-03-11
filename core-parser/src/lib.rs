#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

pub mod ast;
pub mod ffi;
pub mod jsx;
pub mod lexer;
pub mod markdown;
pub mod parser;

// WebAssembly Bindings

/// Compiles the raw MDX input into a minified JSON string representing the AST.
///
/// This function is exposed directly to JavaScript via `wasm-bindgen`. 
/// It returns a standard `JsError` if the parsing fails, ensuring that Node.js 
/// or the browser can `try/catch` the error cleanly instead of encountering a WASM panic.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn parse_mdx_to_json(input: &str) -> Result<String, JsError> {
    let ast = parser::parse_mdx(input).map_err(|e| JsError::new(&e.to_string()))?;
    serde_json::to_string(&ast).map_err(|e| JsError::new(&e.to_string()))
}

/// Compiles the raw MDX input into a pretty-printed JSON string.
///
/// Useful for debugging, logging, or local development environments where 
/// human-readable AST introspection is required.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn parse_mdx_to_json_pretty(input: &str) -> Result<String, JsError> {
    let ast = parser::parse_mdx(input).map_err(|e| JsError::new(&e.to_string()))?;
    serde_json::to_string_pretty(&ast).map_err(|e| JsError::new(&e.to_string()))
}

/// Utility function exposed to JavaScript to fetch the compiled parser's version,
/// allowing the frontend package to verify WASM binary compatibility.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn parse_mdx_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(feature = "python")]
pub mod python;