use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Arc;
use crate::ast::{AstNode, AttrValue};
use crate::parser::parse_mdx;

/// Parses a raw MDX string from the host environment (e.g., JavaScript/WASM or C) 
/// and returns a serialized JSON representation of the AST.
///
/// # Safety
/// * `input` must be a valid, non-null, null-terminated UTF-8 C string.
/// * The returned pointer is heap-allocated by Rust. The host **must** free this 
///   pointer by calling [`mdx_free`] after consuming the string.
#[no_mangle]
pub unsafe extern "C" fn mdx_parse(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(input) };

    let s = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return error_json("Invalid UTF-8 input"),
    };

    match parse_mdx(s) {
        Ok(ast) => match serde_json::to_string(&ast) {
            Ok(json) => match CString::new(json) {
                Ok(cs) => cs.into_raw(),
                Err(_) => error_json("JSON contained interior null byte"),
            },
            Err(e) => error_json(&e.to_string()),
        },
        Err(e) => error_json(&e.to_string()),
    }
}

/// Frees a string pointer previously allocated and returned by [`mdx_parse`].
///
/// # Safety
/// * `ptr` must be either null or a pointer strictly returned by [`mdx_parse`].
/// * It must not have been freed already (to avoid a double-free memory corruption).
#[no_mangle]
pub unsafe extern "C" fn mdx_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(ptr));
    }
}

/// Formats a Rust error into a standardized JSON payload: `{"error": "<msg>"}`.
///
/// Returns a heap-allocated C string (which must also be freed with [`mdx_free`]).
fn error_json(msg: &str) -> *mut c_char {
    let payload = format!(r#"{{"error":{}}}"#, serde_json::json!(msg));
    match CString::new(payload) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

// Node.js native bindings (N-API)
//
// The architecture below eliminates the FFI serialization bottleneck.
// Instead of converting the entire AST to JSON on every call, the AST is stored
// in Rust memory (behind an Arc) and accessed lazily from JavaScript via
// lightweight napi method calls.
//
// Memory management is handled automatically:
//   - `MdxAst` wraps the AST in an `Arc<Vec<AstNode>>`.
//   - Each `MdxNode` holds a clone of that `Arc`, keeping the allocation alive
//     as long as any node is referenced from JavaScript.
//   - When all JS references (MdxAst + all MdxNodes) are garbage collected,
//     the Arc refcount drops to zero and Rust frees the memory.
//   - No manual `free()` call is needed from userland.

#[cfg(feature = "node")]
mod node {
    use super::*;
    use napi::bindgen_prelude::*;
    use napi_derive::napi;
    use std::collections::HashMap;

    // MdxNode
    //
    // Represents a single AST node. Holds a reference to the shared Arc so the
    // underlying allocation stays alive regardless of the MdxAst lifetime.
    // The `path` field encodes the position of this node within the tree as a
    // sequence of child indices, avoiding any copy of the node data itself.

    #[napi]
    pub struct MdxNode {
        /// Shared ownership of the root AST — keeps memory alive.
        ast:  Arc<Vec<AstNode<'static>>>,
        /// Path from root to this node, e.g. [2, 0, 1] means
        /// root[2].children[0].children[1].
        path: Vec<usize>,
    }

    impl MdxNode {
        /// Constructs a new MdxNode from a shared Arc and a path.
        fn new(ast: Arc<Vec<AstNode<'static>>>, path: Vec<usize>) -> Self {
            MdxNode { ast, path }
        }

        /// Resolves the path through the AST tree and returns a reference to
        /// the underlying AstNode, or None if the path is invalid.
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

        /// Constructs a child MdxNode by appending a child index to the current path.
        fn child_node(&self, child_index: usize) -> MdxNode {
            let mut child_path = self.path.clone();
            child_path.push(child_index);
            MdxNode::new(Arc::clone(&self.ast), child_path)
        }
    }

    #[napi]
    impl MdxNode {
        /// The tag name or element type (e.g., `"p"`, `"h1"`, `"Note"`, `"text"`).
        #[napi(getter)]
        pub fn node_type(&self) -> String {
            self.resolve()
                .map(|n| n.node_type.to_string())
                .unwrap_or_default()
        }

        /// The raw text content. Only populated for leaf nodes of type `"text"`.
        #[napi(getter)]
        pub fn content(&self) -> Option<String> {
            self.resolve().and_then(|n| n.content.as_ref().map(|c| c.to_string()))
        }

        /// Whether the tag was self-closing in the source MDX (e.g., `<Chart />`).
        #[napi(getter)]
        pub fn self_closing(&self) -> bool {
            self.resolve().map(|n| n.self_closing).unwrap_or(false)
        }

        /// The number of direct children of this node.
        #[napi(getter)]
        pub fn child_count(&self) -> u32 {
            self.resolve()
                .map(|n| n.children.len() as u32)
                .unwrap_or(0)
        }

        /// Returns all direct children as MdxNode instances.
        /// Each child shares the same Arc, so no AST data is copied.
        #[napi(getter)]
        pub fn children(&self) -> Vec<MdxNode> {
            let count = self.child_count() as usize;
            (0..count).map(|i| self.child_node(i)).collect()
        }

        /// Returns a single child by index, or None if out of bounds.
        #[napi]
        pub fn get_child(&self, index: u32) -> Option<MdxNode> {
            let node = self.resolve()?;
            if (index as usize) < node.children.len() {
                Some(self.child_node(index as usize))
            } else {
                None
            }
        }

        /// Returns the raw JSX/HTML attributes as a plain JavaScript object.
        /// Expression values are returned as strings; AST values are serialized to JSON.
        #[napi(getter)]
        pub fn attributes(&self) -> Option<String> {
            let attrs = self.resolve()?.attributes.as_ref()?;
            let mut map = HashMap::with_capacity(attrs.len());
            for (key, val) in attrs {
                let js_val = match val {
                    AttrValue::Text(s)       => serde_json::Value::String(s.to_string()),
                    AttrValue::Expression(s) => serde_json::Value::String(s.to_string()),
                    AttrValue::Boolean       => serde_json::Value::Bool(true),
                    AttrValue::Ast(nodes)    => {
                        // Nested AST attribute — serialize to JSON for JS consumption.
                        serde_json::to_value(nodes).unwrap_or(serde_json::Value::Null)
                    }
                };
                map.insert(key.to_string(), js_val);
            }
            serde_json::to_string(&map).ok()
        }

        /// Escape hatch: serializes this node and all its descendants to a JSON string.
        /// Useful for AI processing pipelines or debug tooling.
        #[napi]
        pub fn to_json(&self) -> napi::Result<String> {
            let node = self.resolve()
                .ok_or_else(|| napi::Error::from_reason("Invalid node path"))?;
            serde_json::to_string(node)
                .map_err(|e| napi::Error::from_reason(e.to_string()))
        }
    }

    // MdxAst
    //
    // The root container returned by `parse()`. Wraps the AST in an Arc so that
    // MdxNode instances can safely outlive the MdxAst object itself.
    // When both MdxAst and all derived MdxNodes are garbage collected by the JS
    // runtime, the Arc refcount drops to zero and Rust reclaims the memory —
    // no manual `free()` call is needed from userland.

    #[napi]
    pub struct MdxAst {
        /// Shared ownership of the parsed AST.
        inner: Arc<Vec<AstNode<'static>>>,
    }

    #[napi]
    impl MdxAst {
        /// The number of top-level nodes in the AST.
        #[napi(getter)]
        pub fn length(&self) -> u32 {
            self.inner.len() as u32
        }

        /// Returns the top-level node at `index`, or None if out of bounds.
        #[napi]
        pub fn get(&self, index: u32) -> Option<MdxNode> {
            if (index as usize) < self.inner.len() {
                Some(MdxNode::new(Arc::clone(&self.inner), vec![index as usize]))
            } else {
                None
            }
        }

        /// Iterates all top-level nodes and returns them as MdxNode instances.
        #[napi(getter)]
        pub fn nodes(&self) -> Vec<MdxNode> {
            (0..self.inner.len())
                .map(|i| MdxNode::new(Arc::clone(&self.inner), vec![i]))
                .collect()
        }

        /// Escape hatch: serializes the entire AST to a JSON string.
        /// Useful for AI processing, caching, or interop with the legacy JSON API.
        #[napi]
        pub fn to_json(&self) -> napi::Result<String> {
            serde_json::to_string(self.inner.as_ref())
                .map_err(|e| napi::Error::from_reason(e.to_string()))
        }

        /// Pretty-prints the entire AST as JSON.
        /// Intended for development and debugging.
        #[napi]
        pub fn to_json_pretty(&self) -> napi::Result<String> {
            serde_json::to_string_pretty(self.inner.as_ref())
                .map_err(|e| napi::Error::from_reason(e.to_string()))
        }
    }

    // Entry point

    /// Parses an MDX string and returns an `MdxAst` handle backed by Rust memory.
    ///
    /// The returned object exposes the AST lazily via method calls, avoiding the
    /// cost of serializing the entire tree to JSON across the FFI boundary.
    /// Memory is managed automatically via Arc reference counting — when all
    /// JavaScript references are garbage collected, Rust frees the allocation.
    #[napi]
    pub fn parse(mdx: String) -> napi::Result<MdxAst> {
        let ast = parse_mdx(&mdx)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        Ok(MdxAst { inner: Arc::new(ast) })
    }

    /// Legacy compatibility: parses MDX and returns the full AST as a JSON string.
    ///
    /// Prefer `parse()` for performance-sensitive paths. Use this function when
    /// interoperating with code that expects the raw JSON format.
    #[napi]
    pub fn parse_to_json(mdx: String) -> napi::Result<String> {
        parse_mdx(&mdx)
            .map(|ast| serde_json::to_string(&ast).unwrap())
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Parses MDX and returns the AST serialized as a highly optimized binary buffer.
    /// This bypasses V8's JSON parser entirely, returning a Uint8Array to JavaScript.
    #[napi]
    pub fn parse_to_binary(mdx_buffer: Buffer) -> napi::Result<Buffer> {
        let raw_bytes: &[u8] = mdx_buffer.as_ref();

        let mdx_str = std::str::from_utf8(raw_bytes)
            .map_err(|e| napi::Error::from_reason(format!("Invalid UTF-8: {}", e)))?;

        let ast = crate::parser::parse_mdx(mdx_str)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        
        let binary_data = crate::binary::encode_ast(&ast);
        Ok(binary_data.into())
    }

    /// Parses MDX and compiles it directly to a JSX string in Rust.
    /// This is the absolute fastest path for React/Next.js web rendering.
    #[napi]
    pub fn compile_to_jsx(mdx_buffer: Buffer) -> napi::Result<String> {
        let raw_bytes: &[u8] = mdx_buffer.as_ref();
        let mdx_str = std::str::from_utf8(raw_bytes)
            .map_err(|e| napi::Error::from_reason(format!("Invalid UTF-8: {}", e)))?;

        let ast = crate::parser::parse_mdx(mdx_str)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        
        let jsx_string = crate::compiler::compile_to_jsx(&ast);
        Ok(jsx_string)
    }
}