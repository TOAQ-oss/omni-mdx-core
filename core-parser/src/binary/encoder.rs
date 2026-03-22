use crate::ast::{AstNode, AttrValue};
use crate::binary::opcodes::*;

/// Encodes a complete AST into a binary byte stream.
pub fn encode_ast(nodes: &[AstNode]) -> Vec<u8> {
    // We pre-allocate a small amount of memory to avoid costly reallocations
    let mut buffer = Vec::with_capacity(nodes.len() * 64);
    
    // We encode the total number of root nodes (as a 4-byte value / u32)
    write_u32(&mut buffer, nodes.len() as u32);
    
    for node in nodes {
        encode_node(node, &mut buffer);
    }
    
    buffer
}

/// Recursively encodes a single node and its children.
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

// Utility functions for binary writing

fn write_u16(buffer: &mut Vec<u8>, val: u16) {
    buffer.extend_from_slice(&val.to_le_bytes());
}

fn write_u32(buffer: &mut Vec<u8>, val: u32) {
    buffer.extend_from_slice(&val.to_le_bytes());
}

fn write_string_u16(buffer: &mut Vec<u8>, val: &str) {
    let bytes = val.as_bytes();
    write_u16(buffer, bytes.len() as u16);
    buffer.extend_from_slice(bytes);
}

fn write_string_u32(buffer: &mut Vec<u8>, val: &str) {
    let bytes = val.as_bytes();
    write_u32(buffer, bytes.len() as u32);
    buffer.extend_from_slice(bytes);
}