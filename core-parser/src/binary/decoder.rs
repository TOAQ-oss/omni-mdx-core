use crate::ast::{AstNode, AttrValue};
use crate::binary::opcodes::*;
use std::borrow::Cow;

/// Errors that can occur during OCP binary decoding.
#[derive(Debug)]
pub enum DecodeError {
    /// The byte stream ended before the structure was complete.
    UnexpectedEof,
    /// An opcode byte was not recognised.
    UnknownOpcode(u8),
    /// A length field pointed past the end of the buffer.
    InvalidLength {
        offset: usize,
        claimed: usize,
        remaining: usize,
    },
    /// A string slice was not valid UTF-8.
    InvalidUtf8 { offset: usize },
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::UnexpectedEof => write!(f, "OCP decode error: unexpected end of buffer"),
            DecodeError::UnknownOpcode(op) => {
                write!(f, "OCP decode error: unknown opcode 0x{:02X}", op)
            }
            DecodeError::InvalidLength {
                offset,
                claimed,
                remaining,
            } => write!(
                f,
                "OCP decode error: at offset {} claimed length {} > {} remaining bytes",
                offset, claimed, remaining
            ),
            DecodeError::InvalidUtf8 { offset } => {
                write!(f, "OCP decode error: invalid UTF-8 at offset {}", offset)
            }
        }
    }
}

impl std::error::Error for DecodeError {}

/// Decodes a complete OCP binary stream into a Vec of root AstNodes.
///
/// This is the exact mirror of `encode_ast`. The stream format is:
///
/// ```text
/// [root_count: u32]
/// [node] × root_count
/// ```
pub fn decode_ast(data: &[u8]) -> Result<Vec<AstNode<'static>>, DecodeError> {
    let mut cursor = Cursor::new(data);
    let root_count = cursor.read_u32()?;

    // Guard against absurd root_count values before allocating
    // (e.g. u32::MAX from a fuzzed payload).
    // A sane document never has more than 1M root nodes.
    const MAX_ROOT_NODES: u32 = 1_000_000;
    if root_count > MAX_ROOT_NODES {
        return Err(DecodeError::InvalidLength {
            offset: 0,
            claimed: root_count as usize,
            remaining: cursor.remaining(),
        });
    }

    let mut nodes = Vec::with_capacity(root_count as usize);
    for _ in 0..root_count {
        nodes.push(decode_node(&mut cursor)?);
    }
    Ok(nodes)
}

/// A minimal cursor that tracks position inside a byte slice.
/// Using a cursor instead of passing `&mut usize` everywhere keeps the
/// borrow checker happy and makes the read helpers easy to reason about.
struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Cursor { data, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    /// Read exactly `n` bytes and advance the cursor.
    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        let end = self.pos + n;
        if end > self.data.len() {
            // No bytes at all left → UnexpectedEof.
            // Some bytes left but not enough → InvalidLength.
            return if self.remaining() == 0 {
                Err(DecodeError::UnexpectedEof)
            } else {
                Err(DecodeError::InvalidLength {
                    offset: self.pos,
                    claimed: n,
                    remaining: self.remaining(),
                })
            };
        }
        let slice = &self.data[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    fn read_u8(&mut self) -> Result<u8, DecodeError> {
        if self.pos >= self.data.len() {
            return Err(DecodeError::UnexpectedEof);
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    fn read_u16(&mut self) -> Result<u16, DecodeError> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32(&mut self) -> Result<u32, DecodeError> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read a UTF-8 string whose length is prefixed as a u16.
    fn read_string_u16(&mut self) -> Result<String, DecodeError> {
        let len = self.read_u16()? as usize;
        let start = self.pos;
        let bytes = self.read_bytes(len)?;
        std::str::from_utf8(bytes)
            .map(|s| s.to_owned())
            .map_err(|_| DecodeError::InvalidUtf8 { offset: start })
    }

    /// Read a UTF-8 string whose length is prefixed as a u32.
    fn read_string_u32(&mut self) -> Result<String, DecodeError> {
        let len_raw = self.read_u32()? as usize;

        // Guard: a single string cannot be longer than the entire remaining buffer.
        if len_raw > self.remaining() {
            return Err(DecodeError::InvalidLength {
                offset: self.pos,
                claimed: len_raw,
                remaining: self.remaining(),
            });
        }

        let start = self.pos;
        let bytes = self.read_bytes(len_raw)?;
        std::str::from_utf8(bytes)
            .map(|s| s.to_owned())
            .map_err(|_| DecodeError::InvalidUtf8 { offset: start })
    }
}

/// Maximum nesting depth to prevent stack overflow on pathological inputs.
const MAX_DEPTH: usize = 512;

fn decode_node(cursor: &mut Cursor<'_>) -> Result<AstNode<'static>, DecodeError> {
    decode_node_inner(cursor, 0)
}

fn decode_node_inner(
    cursor: &mut Cursor<'_>,
    depth: usize,
) -> Result<AstNode<'static>, DecodeError> {
    if depth > MAX_DEPTH {
        // Treat runaway nesting as a malformed payload.
        return Err(DecodeError::InvalidLength {
            offset: cursor.pos,
            claimed: depth,
            remaining: cursor.remaining(),
        });
    }

    let opcode = cursor.read_u8()?;

    match opcode {
        NODE_TEXT => {
            let content = cursor.read_string_u32()?;
            Ok(AstNode {
                node_type: Cow::Owned("text".to_owned()),
                content: Some(Cow::Owned(content)),
                self_closing: false,
                attributes: None,
                children: vec![],
            })
        }

        NODE_ELEMENT => {
            // 1. Tag name
            let tag_name = cursor.read_string_u16()?;

            // 2. Self-closing flag
            let self_closing_byte = cursor.read_u8()?;
            let self_closing = self_closing_byte != 0;

            // 3. Attributes
            let attr_count = cursor.read_u16()? as usize;

            // Guard: refuse absurd attribute counts before allocating.
            const MAX_ATTRS: usize = 4096;
            if attr_count > MAX_ATTRS {
                return Err(DecodeError::InvalidLength {
                    offset: cursor.pos,
                    claimed: attr_count,
                    remaining: cursor.remaining(),
                });
            }

            let mut attributes = if attr_count > 0 {
                Some(std::collections::HashMap::with_capacity(attr_count))
            } else {
                None
            };

            for _ in 0..attr_count {
                let key = cursor.read_string_u16()?;
                let attr_opcode = cursor.read_u8()?;

                let value = match attr_opcode {
                    ATTR_TEXT => {
                        let s = cursor.read_string_u32()?;
                        AttrValue::Text(Cow::Owned(s))
                    }
                    ATTR_EXPRESSION => {
                        let s = cursor.read_string_u32()?;
                        AttrValue::Expression(Cow::Owned(s))
                    }
                    ATTR_BOOLEAN => AttrValue::Boolean,
                    ATTR_AST => {
                        let sub_count = cursor.read_u32()? as usize;
                        const MAX_SUB_NODES: usize = 65536;
                        if sub_count > MAX_SUB_NODES {
                            return Err(DecodeError::InvalidLength {
                                offset: cursor.pos,
                                claimed: sub_count,
                                remaining: cursor.remaining(),
                            });
                        }
                        let mut sub_nodes = Vec::with_capacity(sub_count);
                        for _ in 0..sub_count {
                            sub_nodes.push(decode_node_inner(cursor, depth + 1)?);
                        }
                        AttrValue::Ast(sub_nodes)
                    }
                    other => return Err(DecodeError::UnknownOpcode(other)),
                };

                attributes
                    .as_mut()
                    .expect("attributes must be initialized when attr_count > 0")
                    .insert(Cow::Owned(key), value);
            }

            // 4. Children
            let child_count = cursor.read_u32()? as usize;

            const MAX_CHILDREN: usize = 1_000_000;
            if child_count > MAX_CHILDREN {
                return Err(DecodeError::InvalidLength {
                    offset: cursor.pos,
                    claimed: child_count,
                    remaining: cursor.remaining(),
                });
            }

            // The encoder may inject a text node as the first child when
            // the AstNode had a `content` field (has_injected_content).
            // We decode all children uniformly — the injected text node
            // is just another NODE_TEXT in the stream.
            let mut children = Vec::with_capacity(child_count);
            let mut injected_content: Option<Cow<'static, str>> = None;

            for i in 0..child_count {
                let child = decode_node_inner(cursor, depth + 1)?;

                // The encoder always places the injected content node first.
                if i == 0 && child.node_type == "text" && !self_closing {
                    // Heuristic: if the element was encoded with injected content,
                    // the first child is a synthetic text node — restore it as content.
                    // We keep it as a child too so roundtrip is lossless at the
                    // byte level; callers can normalise if needed.
                    injected_content = child.content.clone();
                }

                children.push(child);
            }

            Ok(AstNode {
                node_type: Cow::Owned(tag_name),
                content: injected_content,
                self_closing,
                attributes,
                children,
            })
        }

        other => Err(DecodeError::UnknownOpcode(other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::encode_ast;
    use crate::parser::parse_mdx;

    /// Round-trip: parse MDX → encode → decode → same structure.
    fn roundtrip(mdx: &str) -> Vec<AstNode<'static>> {
        let ast = parse_mdx(mdx).expect("parse_mdx failed");
        let bytes = encode_ast(&ast);
        decode_ast(&bytes).expect("decode_ast failed")
    }

    #[test]
    fn roundtrip_empty() {
        let result = roundtrip("");
        assert!(result.is_empty());
    }

    #[test]
    fn roundtrip_simple_paragraph() {
        let result = roundtrip("Hello world.");
        assert!(!result.is_empty());
    }

    #[test]
    fn roundtrip_heading() {
        let result = roundtrip("# Title");
        assert!(result.iter().any(|n| n.node_type == "h1"));
    }

    #[test]
    fn roundtrip_jsx_with_attrs() {
        let result = roundtrip(r#"<Note title="test">content</Note>"#);
        let note = result.iter().find(|n| n.node_type == "Note");
        assert!(note.is_some());
    }

    #[test]
    fn roundtrip_nested() {
        let mdx = "# H1\n\nParagraph with **bold** and *italic*.\n\n<Box>\n\n## Nested\n\n</Box>";
        let result = roundtrip(mdx);
        assert!(!result.is_empty());
    }

    #[test]
    fn decode_empty_buffer() {
        assert!(matches!(decode_ast(&[]), Err(DecodeError::UnexpectedEof)));
    }

    #[test]
    fn decode_overflow_root_count() {
        let mut buf = vec![];
        buf.extend_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode_ast(&buf).is_err());
    }

    #[test]
    fn decode_overflow_string_length() {
        let mut buf = vec![];
        buf.extend_from_slice(&1u32.to_le_bytes()); // root_count = 1
        buf.push(NODE_TEXT);
        buf.extend_from_slice(&u32::MAX.to_le_bytes()); // string len = u32::MAX
        assert!(decode_ast(&buf).is_err());
    }

    #[test]
    fn decode_unknown_opcode() {
        let mut buf = vec![];
        buf.extend_from_slice(&1u32.to_le_bytes());
        buf.push(0xFF); // unknown opcode
        assert!(matches!(
            decode_ast(&buf),
            Err(DecodeError::UnknownOpcode(0xFF))
        ));
    }

    #[test]
    fn decode_overflow_attr_count() {
        let mut buf = vec![];
        buf.extend_from_slice(&1u32.to_le_bytes());
        buf.push(NODE_ELEMENT);
        buf.extend_from_slice(&3u16.to_le_bytes()); // tag "Box"
        buf.extend_from_slice(b"Box");
        buf.push(0); // self_closing = false
        buf.extend_from_slice(&u16::MAX.to_le_bytes()); // attr_count = 65535
        assert!(decode_ast(&buf).is_err());
    }
}
