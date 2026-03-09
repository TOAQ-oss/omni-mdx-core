use std::collections::HashMap;

use crate::ast::{AstNode, AttrValue, ParseError};

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
pub fn parse_jsx(
    input: &str,
    block_math: &[String],
    inline_math: &[String],
) -> Result<AstNode, ParseError> {
    let input = input.trim();
    let bytes = input.as_bytes();

    if bytes.is_empty() || bytes[0] != b'<' {
        return Err(ParseError::UnexpectedToken {
            pos: 0,
            got: bytes.first().copied().unwrap_or(b'?') as char,
        });
    }

    // 1. Parse the opening tag to extract the name, attributes, and self-closing status.
    let (tag_name, attrs, self_closing, after_open) = parse_open_tag(input, block_math, inline_math)?;

    let mut node = AstNode::element(&tag_name, self_closing);
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
fn parse_open_tag(
    input: &str,
    block_math: &[String],
    inline_math: &[String],
) -> Result<(String, HashMap<String, AttrValue>, bool, usize), ParseError> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 1; // skip `<`

    // Extract Tag Name: supports letters, digits, and specific symbols (like `.`, `-`, `_`)
    let name_start = i;
    while i < len && is_tag_name_char(bytes[i]) {
        i += 1;
    }
    let tag_name = input[name_start..i].to_string();

    // Attributes
    let mut attrs: HashMap<String, AttrValue> = HashMap::new();

    loop {
        // Skip leading whitespace before the next attribute or the closing bracket.
        while i < len && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\n') {
            i += 1;
        }

        if i >= len {
            return Err(ParseError::UnclosedTag { name: tag_name });
        }

        // Check for the end of the opening tag `>`
        if bytes[i] == b'>' {
            return Ok((tag_name, attrs, false, i + 1));
        }

        // Check for a self-closing tag `/>`
        if bytes[i] == b'/' {
            i += 1;
            if i < len && bytes[i] == b'>' {
                return Ok((tag_name, attrs, true, i + 1));
            }
            return Err(ParseError::UnexpectedToken { pos: i, got: '/' });
        }

        // Extract Attribute Name
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
        let attr_name = input[attr_start..i].to_string();

        // Skip whitespace around the `=` operator
        while i < len && bytes[i] == b' ' {
            i += 1;
        }

        // If there's no `=`, it's a bare boolean attribute (e.g., `<Input disabled />`)
        if i >= len || bytes[i] != b'=' {
            attrs.insert(attr_name, AttrValue::Boolean);
            continue;
        }

        i += 1; // skip `=`
        while i < len && bytes[i] == b' ' {
            i += 1;
        }

        if i >= len {
            return Err(ParseError::UnclosedTag { name: tag_name });
        }

        // Extract Attribute Value
        let value = match bytes[i] {
            // Case 1: Standard quoted string values (e.g., `class="btn"`)
            b'"' | b'\'' => {
                let q = bytes[i] as char;
                i += 1;
                let str_start = i;
                let mut end_byte = str_start;

                // Using char_indices ensures we safely step over multi-byte UTF-8 chars
                for (byte_pos, ch) in input[str_start..].char_indices() {
                    if ch == q {
                        end_byte = str_start + byte_pos;
                        i = end_byte + q.len_utf8(); // skip closing quote
                        break;
                    }
                }
                AttrValue::Text(input[str_start..end_byte].to_string())
            }
            // Case 2: JSX Expressions enclosed in braces (e.g., `data={{ id: 1 }}`)
            b'{' => {
                let brace_start_byte = i + 1; // byte offset after opening `{`
                let mut depth = 1i32;
                let mut inner_quote: Option<char> = None;
                let mut end_byte = brace_start_byte;

                // Carefully track nested braces and ignore braces inside internal strings.
                for (byte_pos, ch) in input[brace_start_byte..].char_indices() {
                    let abs = brace_start_byte + byte_pos;
                    if let Some(q) = inner_quote {
                        if ch == q { inner_quote = None; }
                        end_byte = abs + ch.len_utf8();
                        continue;
                    }
                    match ch {
                        '"' | '\'' | '`' => { inner_quote = Some(ch); }
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                end_byte = abs; // points at closing `}`
                                break;
                            }
                        }
                        _ => {}
                    }
                    end_byte = abs + ch.len_utf8();
                }

                let expr = input[brace_start_byte..end_byte].trim().to_string();
                i = end_byte + 1; // Advance past the closing `}`

                // Heuristic: If the expression starts with `<Letter`, it might be an inline JSX component.
                let expr_bytes = expr.as_bytes();
                let looks_like_jsx = expr_bytes.first() == Some(&b'<')
                    && expr_bytes.get(1).map(|b| (*b as char).is_ascii_alphabetic()).unwrap_or(false);

                if looks_like_jsx {
                    let first_letter = expr_bytes.get(1).copied().unwrap_or(0) as char;
                    if first_letter.is_ascii_uppercase() {
                        // Custom JSX Component inside an attribute (e.g., `icon={<Icon />}`)
                        match parse_jsx(&expr, block_math, inline_math) {
                            Ok(node) => AttrValue::Ast(vec![node]),
                            Err(_)   => AttrValue::Expression(expr),
                        }
                    } else {
                        // Standard HTML wrapper inside an attribute (e.g., `label={<span>Text</span>}`)
                        match parse_html_wrapper(&expr, block_math, inline_math) {
                            Ok(node) => AttrValue::Ast(vec![node]),
                            Err(_)   => AttrValue::Expression(expr),
                        }
                    }
                } else {
                    // It's just a standard JavaScript expression (e.g., a JSON object or variable)
                    AttrValue::Expression(expr)
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
fn parse_children(
    src: &str,
    block_math: &[String],
    inline_math: &[String],
) -> Result<Vec<AstNode>, ParseError> {
    let mut children = Vec::new();
    let bytes = src.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'<' && i + 1 < len && (bytes[i + 1] as char).is_ascii_uppercase() {
            // Found a nested custom JSX child (e.g., `<Child />`). Find its bounds and parse recursively.
            let start = i;
            let mut end = i;
            crate::lexer::scan_jsx_block_pub(bytes, &mut end, len)
                .map_err(|_| ParseError::UnclosedJsxBlock { pos: start })?;
            let child_src = &src[start..end];
            children.push(parse_jsx(child_src, block_math, inline_math)?);
            i = end;
        } else {
            // Accumulate plain text (or standard HTML) until the next custom JSX tag.
            let start = i;
            while i < len {
                if bytes[i] == b'<' && i + 1 < len && (bytes[i + 1] as char).is_ascii_uppercase() {
                    break;
                }
                i += 1;
            }
            let text = &src[start..i];
            if !text.trim().is_empty() {
                // The text already has math placeholders, so we bypass `parse_mdx` to avoid re-extraction.
                // We pass an empty JSX pool because any nested custom JSX was already handled above.
                let mut sub = crate::markdown::parse_markdown(
                    text, &[], block_math, inline_math,
                )?;
                // Markdown parsers often wrap loose text in a `<p>` tag. 
                // We unwrap it here if it's the only element, preventing unnecessary nesting in the final DOM.
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

/// Parses standard HTML tags that act as wrappers in MDX (e.g., `<div className="prose">...</div>`).
///
/// This is necessary because while we let the Markdown parser handle simple HTML, 
/// complex wrappers with specific attributes need to be parsed into proper AST nodes 
/// so their inner Markdown content is processed correctly.
fn parse_html_wrapper(
    input: &str,
    block_math: &[String],
    inline_math: &[String],
) -> Result<AstNode, ParseError> {
    let bytes = input.as_bytes();
    let len = bytes.len();

    // Parse tag name
    let mut i = 1; // skip `<`
    let name_start = i;
    while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_') {
        i += 1;
    }
    let tag_name = input[name_start..i].to_string();

    // Parse attributes (reuse existing logic)
    // Reuse the robust attribute parsing logic from `parse_open_tag`.
    let (_, attrs, self_closing, after_open) =
        parse_open_tag(input, block_math, inline_math)?;

    let mut node = AstNode::element(&tag_name, self_closing);
    if !attrs.is_empty() {
        node.attributes = Some(attrs);
    }

    if self_closing {
        return Ok(node);
    }

    // Parse inner content
    let rest = &input[after_open..];
    let inner = strip_closing_tag(rest, &tag_name)?;
    // Dedenting is crucial: if an HTML wrapper is indented, its inner Markdown content 
    // might accidentally trigger Markdown's 4-space `<pre>` code block rule.
    let dedented = dedent(inner);

    // Re-run the JSX cloaking pass on the inner content to catch custom components 
    // nested inside the HTML wrapper.
    let (md_text, inner_jsx_pool) = crate::lexer::extract_jsx(&dedented)
        .map_err(|e| e)?;

    let children = crate::markdown::parse_markdown(
        &md_text, &inner_jsx_pool, block_math, inline_math,
    )?;
    node.children = children;

    Ok(node)
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

/// Removes common leading indentation from a block of text.
///
/// When users format their MDX, they often indent the content inside tags for readability.
/// However, standard Markdown interprets 4 spaces of indentation as a code block. 
/// This helper calculates the minimum indentation across all non-empty lines and strips it,
/// preserving the intended Markdown structure.
fn dedent(s: &str) -> String {
    let min_indent = s
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    if min_indent == 0 {
        return s.to_string();
    }

    s.lines()
        .map(|l| {
            if l.trim().is_empty() { "" } else { &l[min_indent.min(l.len())..] }
        })
        .collect::<Vec<_>>()
        .join("\n")
}


#[cfg(test)]
mod tests {
    use super::*;

    // Helper: parse JSX with no math pools (for tests that don't involve math).
    fn p(src: &str) -> AstNode { parse_jsx(src, &[], &[]).unwrap() }

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
        let node = parse_jsx(
            "<Note>\x02MATHI0\x03</Note>",
            &[], &inline_pool
        ).unwrap();
        assert_eq!(node.children[0].node_type, "InlineMath");
        assert_eq!(node.children[0].content.as_deref(), Some("E = mc^2"));
    }
}