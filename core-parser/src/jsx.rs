use crate::ast::{AstNode, AttrValue, ParseError};
use std::borrow::Cow;
use std::collections::HashMap;

/// Parses a raw JSX source block (isolated by the lexer) into an [`AstNode`].
///
/// This function acts as the primary orchestrator for converting raw JSX strings
/// (like `<Chart data={...} />`) into structured AST nodes. It delegates the parsing
/// of the opening tag (and its attributes) to `parse_open_tag`, and if the tag is paired,
/// it parses the inner content as children.
///
/// # Arguments
/// * `input` - The raw JSX string block.
/// * `block_math` - The pool of extracted display math blocks (passed down to avoid re-parsing).
/// * `inline_math` - The pool of extracted inline math blocks.
///
/// # Errors
/// Returns a [`ParseError`] if the input does not start with a valid tag or is malformed.
pub fn parse_jsx<'a>(
    input: &'a str,
    block_math: &'a [String],
    inline_math: &'a [String],
) -> Result<AstNode<'a>, ParseError> {
    let input = input.trim();
    let bytes = input.as_bytes();

    if bytes.is_empty() || bytes[0] != b'<' {
        return Err(ParseError::UnexpectedToken {
            pos: 0,
            got: bytes.first().copied().unwrap_or(b'?') as char,
        });
    }

    // 1. Parse the opening tag to extract the name, attributes, and self-closing status.
    let (tag_name, attrs, self_closing, after_open) =
        parse_open_tag(input, block_math, inline_math)?;

    let mut node = AstNode::element(tag_name.clone(), self_closing);
    if !attrs.is_empty() {
        node.attributes = Some(attrs);
    }

    // 2. If it's a self-closing tag (e.g., `<br />`), we are done.
    if self_closing {
        return Ok(node);
    }

    // 3. Otherwise, extract the inner content and parse it as children.
    let rest = &input[after_open..];
    let children_src = strip_closing_tag(rest, &tag_name)?;
    node.children = parse_children(children_src, block_math, inline_math)?;

    Ok(node)
}

/// Parses an opening JSX/HTML tag, extracting its name and attributes.
///
/// This is a complex state machine designed to handle the various ways attributes
/// can be defined in MDX, including raw strings (`prop="val"`), booleans (`disabled`),
/// and dynamic expressions (`prop={{ a: 1 }}`).
///
/// # Returns
/// A tuple containing:
/// 1. `String`: The tag name.
/// 2. `HashMap<String, AttrValue>`: The extracted attributes.
/// 3. `bool`: True if the tag is self-closing (`/>`).
/// 4. `usize`: The byte offset immediately following the closing `>`.
fn parse_open_tag<'a>(
    input: &'a str,
    block_math: &'a [String],
    inline_math: &'a [String],
) -> Result<
    (
        Cow<'a, str>,
        HashMap<Cow<'a, str>, AttrValue<'a>>,
        bool,
        usize,
    ),
    ParseError,
> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 1; // skip `<`

    let name_start = i;
    while i < len && is_tag_name_char(bytes[i]) {
        i += 1;
    }
    // Zero-Copy: we take a reference to the input
    let tag_name = Cow::Borrowed(&input[name_start..i]);

    let mut attrs: HashMap<Cow<'a, str>, AttrValue<'a>> = HashMap::new();

    loop {
        while i < len && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\n') {
            i += 1;
        }

        if i >= len {
            return Err(ParseError::UnclosedTag {
                name: tag_name.to_string(),
            });
        }

        if bytes[i] == b'>' {
            return Ok((tag_name, attrs, false, i + 1));
        }

        if bytes[i] == b'/' {
            i += 1;
            if i < len && bytes[i] == b'>' {
                return Ok((tag_name, attrs, true, i + 1));
            }
            return Err(ParseError::UnexpectedToken { pos: i, got: '/' });
        }

        let attr_start = i;
        while i < len && is_attr_name_char(bytes[i]) {
            i += 1;
        }
        if i == attr_start {
            return Err(ParseError::UnexpectedToken {
                pos: i,
                got: bytes[i] as char,
            });
        }
        // Zero-Copy: Attribute name
        let attr_name = Cow::Borrowed(&input[attr_start..i]);

        while i < len && bytes[i] == b' ' {
            i += 1;
        }

        if i >= len || bytes[i] != b'=' {
            attrs.insert(attr_name, AttrValue::Boolean);
            continue;
        }

        i += 1; // skip `=`
        while i < len && bytes[i] == b' ' {
            i += 1;
        }

        let value = match bytes[i] {
            b'"' | b'\'' => {
                let q = bytes[i] as char;
                i += 1;
                let str_start = i;
                let mut end_byte = str_start;
                for (byte_pos, ch) in input[str_start..].char_indices() {
                    if ch == q {
                        end_byte = str_start + byte_pos;
                        i = end_byte + q.len_utf8();
                        break;
                    }
                }
                // Zero-Copy: Text value
                AttrValue::Text(Cow::Borrowed(&input[str_start..end_byte]))
            }
            b'{' => {
                let brace_start_byte = i + 1;
                let mut depth = 1i32;
                let mut inner_quote: Option<char> = None;
                let mut end_byte = brace_start_byte;

                for (byte_pos, ch) in input[brace_start_byte..].char_indices() {
                    let abs = brace_start_byte + byte_pos;
                    if let Some(q) = inner_quote {
                        if ch == q {
                            inner_quote = None;
                        }
                        end_byte = abs + ch.len_utf8();
                        continue;
                    }
                    match ch {
                        '"' | '\'' | '`' => {
                            inner_quote = Some(ch);
                        }
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                end_byte = abs;
                                break;
                            }
                        }
                        _ => {}
                    }
                    end_byte = abs + ch.len_utf8();
                }

                let expr = input[brace_start_byte..end_byte].trim();
                i = end_byte + 1;

                let expr_bytes = expr.as_bytes();
                if expr_bytes.first() == Some(&b'<') {
                    // Optimized recursion
                    match parse_jsx(expr, block_math, inline_math) {
                        Ok(node) => AttrValue::Ast(vec![node]),
                        Err(_) => AttrValue::Expression(Cow::Borrowed(expr)),
                    }
                } else {
                    AttrValue::Expression(Cow::Borrowed(expr))
                }
            }
            other => {
                return Err(ParseError::UnexpectedToken {
                    pos: i,
                    got: other as char,
                })
            }
        };

        attrs.insert(attr_name, value);
    }
}

// Parses the inner content of a paired JSX tag into a list of child nodes.
///
/// This function acts as a mini-router:
/// - If it spots a nested custom JSX component, it isolates it via the lexer and recursively calls `parse_jsx`.
/// - Otherwise, it hands the text block over to the standard Markdown parser.
fn parse_children<'a>(
    src: &'a str,
    block_math: &'a [String],
    inline_math: &'a [String],
) -> Result<Vec<AstNode<'a>>, ParseError> {
    let mut children = Vec::new();
    let bytes = src.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'<' && i + 1 < len && (bytes[i + 1] as char).is_ascii_uppercase() {
            let start = i;
            let mut end = i;
            crate::lexer::scan_jsx_block_pub(bytes, &mut end, len)
                .map_err(|_| ParseError::UnclosedJsxBlock { pos: start })?;
            children.push(parse_jsx(&src[start..end], block_math, inline_math)?);
            i = end;
        } else {
            let start = i;
            while i < len {
                if bytes[i] == b'<' && i + 1 < len && (bytes[i + 1] as char).is_ascii_uppercase() {
                    break;
                }
                i += 1;
            }
            let text = &src[start..i];
            if !text.trim().is_empty() {
                let mut sub = crate::markdown::parse_markdown(text, &[], block_math, inline_math)?;
                if sub.len() == 1 && sub[0].node_type == "p" {
                    children.extend(sub.remove(0).children);
                } else {
                    children.extend(sub);
                }
            }
        }
    }
    Ok(children)
}

/// Locates the corresponding closing tag for a paired element and returns the inner content.
///
/// # Errors
/// Returns a [`ParseError::UnclosedTag`] if the closing tag cannot be found in the remaining string.
fn strip_closing_tag<'a>(rest: &'a str, tag_name: &str) -> Result<&'a str, ParseError> {
    let close = format!("</{tag_name}>");
    match rest.rfind(&close) {
        Some(pos) => Ok(&rest[..pos]),
        None => Err(ParseError::UnclosedTag {
            name: tag_name.to_string(),
        }),
    }
}

/// Determines if a byte is a valid character for an HTML/JSX tag name.
fn is_tag_name_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b'_'
}

/// Determines if a byte is a valid character for an HTML/JSX attribute name.
fn is_attr_name_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b':'
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: parse JSX with no math pools (for tests that don't involve math).
    fn p<'a>(src: &'a str) -> AstNode<'a> {
        parse_jsx(src, &[], &[]).unwrap()
    }

    #[test]
    fn self_closing_no_attrs() {
        let node = p("<Foo />");
        assert_eq!(node.node_type, "Foo");
        assert!(node.self_closing);
        assert!(node.attributes.is_none());
    }

    #[test]
    fn self_closing_with_attrs() {
        let node = p(r#"<Alert type="error" count={42} disabled />"#);
        assert_eq!(node.node_type, "Alert");
        let attrs = node.attributes.unwrap();
        assert_eq!(attrs["type"], AttrValue::Text("error".into()));
        assert_eq!(attrs["count"], AttrValue::Expression("42".into()));
        assert_eq!(attrs["disabled"], AttrValue::Boolean);
    }

    #[test]
    fn paired_with_text_child() {
        let node = p("<Box>hello world</Box>");
        assert!(!node.self_closing);
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].content.as_deref(), Some("hello world"));
    }

    #[test]
    fn nested_jsx_children() {
        let node = p("<Outer><Inner /></Outer>");
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].node_type, "Inner");
    }

    #[test]
    fn dotted_component_name() {
        let node = p("<A.B.C />");
        assert_eq!(node.node_type, "A.B.C");
    }

    #[test]
    fn math_placeholder_in_children() {
        // Simulate what parse_mdx passes: placeholder already substituted.
        let inline_pool = vec!["E = mc^2".to_string()];
        let node = parse_jsx("<Note>\x02MATHI0\x03</Note>", &[], &inline_pool).unwrap();
        assert_eq!(node.children[0].node_type, "InlineMath");
        assert_eq!(node.children[0].content.as_deref(), Some("E = mc^2"));
    }
}
