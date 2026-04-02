//! Omni-Core MDX Engine
//!
//! A high-performance, security-first MDX parser and binary protocol encoder.
//!
//! Omni-Core is designed to parse complex Markdown, LaTeX, and JSX structures into a safe
//! Abstract Syntax Tree (AST). It features built-in algorithmic complexity shields to prevent
//! $O(n^2)$ Denial-of-Service (DoS) attacks, making it safe for both server-side (Node.js/Python)
//! and client-side (WebAssembly) execution.
//!
//! # Features
//! * `wasm`: Enables WebAssembly bindings for browser and Node.js execution.
//! * `python`: Enables PyO3 bindings for native Python execution.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Core Abstract Syntax Tree (AST) structures.
pub mod ast;
/// Omni-Core Protocol (OCP) binary encoder and decoder.
pub mod binary;
/// AST compilation and transformation utilities.
pub mod compiler;
/// Foreign Function Interface (FFI) bindings for C/C++ integration.
pub mod ffi;
/// JSX component lexer and validator.
pub mod jsx;
/// Core lexical analysis tools.
pub mod lexer;
/// Markdown structure and LaTeX math processing.
pub mod markdown;
/// The main parser pipeline and DoS protection shields.
pub mod parser;

// ============================================================================
// WebAssembly Bindings
// ============================================================================

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

/// Parses raw MDX bytes and encodes the resulting AST into the Omni-Core Protocol (OCP) binary format.
///
/// This is the most efficient way to transmit parsed MDX structures over the network.
/// It accepts a raw byte slice (`Uint8Array` in JavaScript) to bypass the overhead of
/// JS-to-Rust string conversion when reading directly from network streams or files.
///
/// # Returns
/// A compact `Uint8Array` representing the OCP binary payload.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn parse_to_binary(mdx_input: &[u8]) -> Result<Vec<u8>, JsValue> {
    let mdx_str = std::str::from_utf8(mdx_input)
        .map_err(|e| JsValue::from_str(&format!("Invalid UTF-8: {}", e)))?;

    let ast = crate::parser::parse_mdx(mdx_str).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let binary_data = crate::binary::encode_ast(&ast);

    Ok(binary_data)
}

/// Utility function exposed to JavaScript to fetch the compiled parser's version.
///
/// Allows frontend packages to verify WASM binary compatibility with their wrapper scripts.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn parse_mdx_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ============================================================================
// Python Bindings
// ============================================================================

/// PyO3 Python bindings for the Omni-Core engine.
#[cfg(feature = "python")]
pub mod python;
