use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::parser::parse_mdx;

/// Parses a raw MDX string from the host environment (e.g., JavaScript/WASM or C) 
/// and returns a serialized JSON representation of the AST.
///
/// This function acts as the bridge between Rust's safe memory management and 
/// the host's raw memory space. It takes ownership of the data translation, 
/// converting the input pointer into a Rust string, running the parser, and 
/// converting the output back into a raw pointer.
///
/// # Safety
/// * `input` must be a valid, non-null, null-terminated UTF-8 C string.
/// * The returned pointer is heap-allocated by Rust. To prevent memory leaks in 
///   the host environment (like WASM linear memory), the host **must** free this 
///   pointer by explicitly calling [`mdx_free`] after consuming the string.
#[no_mangle]
pub unsafe extern "C" fn mdx_parse(input: *const c_char) -> *mut c_char {
    // 1. Null check: Prevent segmentation faults from bad host calls.
    if input.is_null() {
        return std::ptr::null_mut();
    }

    // SAFETY: The caller guarantees a valid, null-terminated string.
    // We borrow it as a CStr to safely calculate its length.
    let c_str = unsafe { CStr::from_ptr(input) };

    // 2. Encoding check: Ensure the byte array is valid UTF-8 before Rust processes it.
    let s = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return error_json("Invalid UTF-8 input"),
    };

    // 3. Parsing & Serialization: Convert MDX to AST, then AST to JSON string.
    match parse_mdx(s) {
        Ok(ast) => match serde_json::to_string(&ast) {
            Ok(json) => match CString::new(json) {
                // `into_raw` transfers ownership of the memory from Rust to the host.
                // Rust will NOT drop this memory until it is explicitly handed back.
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
/// Because WebAssembly (and C FFI) does not share a garbage collector with Rust,
/// any memory allocated in Rust (`into_raw`) must be explicitly dropped in Rust.
///
/// # Safety
/// * `ptr` must be either null or a pointer strictly returned by [`mdx_parse`].
/// * It must not have been freed already (to avoid a double-free memory corruption).
#[no_mangle]
pub unsafe extern "C" fn mdx_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    // SAFETY: We retake ownership of the allocation created by `CString::into_raw`
    // by turning it back into a `CString`. As soon as the scope ends, Rust's
    // standard `Drop` trait will cleanly deallocate the memory.
    unsafe {
        drop(CString::from_raw(ptr));
    }
}

/// Formats a Rust error into a standardized JSON payload: `{"error": "<msg>"}`.
///
/// This ensures that when the parser fails, the host environment (like Node.js) 
/// receives a cleanly formatted JSON string that it can `JSON.parse()` and throw 
/// as a native JavaScript Error, rather than receiving a hard Rust panic that 
/// would crash the entire WebAssembly module.
///
/// Returns a heap-allocated C string (which must also be freed with [`mdx_free`]).
fn error_json(msg: &str) -> *mut c_char {
    let payload = format!(r#"{{"error":{}}}"#, serde_json::json!(msg));
    match CString::new(payload) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(), // Failsafe fallback
    }
}