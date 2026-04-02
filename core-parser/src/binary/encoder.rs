//! Omni-Core Protocol (OCP) Binary Encoder
//!
//! This module serializes the validated Abstract Syntax Tree (AST) into a highly
//! optimized, compact binary format (OCP). This format is designed to be quickly
//! decoded by WebAssembly (WASM) clients or native frontends, completely bypassing
//! the CPU overhead of parsing JSON.
//!
//! # Architecture
//! * **Little-Endian:** All integers are serialized in Little-Endian format to match
//!   native WASM and modern CPU architectures, allowing zero-cost deserialization.
//! * **Contiguous Memory:** Uses `extend_from_slice` to perform fast, bulk memory writes.

use crate::ast::{AstNode, AttrValue};
use crate::binary::opcodes::*;

/// Encodes a complete AST into a binary byte stream.
///
/// # Format
/// The stream begins with a 32-bit integer indicating the number of root nodes,
/// followed by the recursive binary representation of each node.
///
/// # Arguments
/// * `nodes` - A slice of root `AstNode`s to encode.
///
/// # Returns
/// A compact `Vec<u8>` containing the OCP binary payload.
pub fn encode_ast(nodes: &[AstNode]) -> Vec<u8> {
    // We pre-allocate a generous amount of memory to avoid costly reallocations
    // during the recursive traversal.
    let mut buffer = Vec::with_capacity(nodes.len() * 64);

    // We encode the total number of root nodes (as a 4-byte value / u32)
    write_u32(&mut buffer, nodes.len() as u32);

    for node in nodes {
        encode_node(node, &mut buffer);
    }

    buffer
}

/// Recursively encodes a single AST node and its descendants into the buffer.
fn encode_node(node: &AstNode, buffer: &mut Vec<u8>) {
    if node.node_type == "text" {
        buffer.push(NODE_TEXT);
        let content = node.content.as_deref().unwrap_or("");
        write_string_u32(buffer, content);
    } else {
        buffer.push(NODE_ELEMENT);

        // 1. Tag name (ex: "h1", "Note")
        write_string_u16(buffer, &node.node_type);

        let has_injected_content = node.content.is_some();

        // 2. Self closing flag (1 octet: 0 or 1)
        // A tag cannot be self-closing if it carries injected text content.
        let is_self_closing = node.self_closing && !has_injected_content;
        buffer.push(if is_self_closing { 1 } else { 0 });

        // 3. Attributes
        if let Some(attrs) = &node.attributes {
            write_u16(buffer, attrs.len() as u16);
            for (key, val) in attrs {
                write_string_u16(buffer, key);
                match val {
                    AttrValue::Text(s) => {
                        buffer.push(ATTR_TEXT);
                        write_string_u32(buffer, s);
                    }
                    AttrValue::Expression(s) => {
                        buffer.push(ATTR_EXPRESSION);
                        write_string_u32(buffer, s);
                    }
                    AttrValue::Boolean => {
                        buffer.push(ATTR_BOOLEAN);
                    }
                    AttrValue::Ast(sub_nodes) => {
                        buffer.push(ATTR_AST);
                        write_u32(buffer, sub_nodes.len() as u32);
                        for sub_node in sub_nodes {
                            encode_node(sub_node, buffer);
                        }
                    }
                }
            }
        } else {
            write_u16(buffer, 0); // 0 attributes
        }

        // 4. Children
        // If the element has direct text content, we synthesize it as an extra child node.
        let total_children = node.children.len() as u32 + if has_injected_content { 1 } else { 0 };
        write_u32(buffer, total_children);

        if let Some(content_str) = &node.content {
            buffer.push(NODE_TEXT);
            write_string_u32(buffer, content_str);
        }

        for child in &node.children {
            encode_node(child, buffer);
        }
    }
}

// ============================================================================
// Utility functions for fast binary writing
// ============================================================================

/// Writes a 16-bit unsigned integer to the buffer in Little-Endian format.
#[inline]
fn write_u16(buffer: &mut Vec<u8>, val: u16) {
    buffer.extend_from_slice(&val.to_le_bytes());
}

/// Writes a 32-bit unsigned integer to the buffer in Little-Endian format.
#[inline]
fn write_u32(buffer: &mut Vec<u8>, val: u32) {
    buffer.extend_from_slice(&val.to_le_bytes());
}

/// Writes a UTF-8 string prefixed by its length as a `u16`.
#[inline]
fn write_string_u16(buffer: &mut Vec<u8>, val: &str) {
    let bytes = val.as_bytes();
    write_u16(buffer, bytes.len() as u16);
    buffer.extend_from_slice(bytes);
}

/// Writes a UTF-8 string prefixed by its length as a `u32`.
#[inline]
fn write_string_u32(buffer: &mut Vec<u8>, val: &str) {
    let bytes = val.as_bytes();
    write_u32(buffer, bytes.len() as u32);
    buffer.extend_from_slice(bytes);
}
