use pulldown_cmark::{Event, Options, Parser, Tag};
use regex::Regex;

use crate::ast::{AstNode, ParseError};
use crate::jsx::parse_jsx;

// To prevent the standard Markdown parser (`pulldown-cmark`) from destroying or 
// misinterpreting our custom JSX and LaTeX math blocks, we temporarily replace them 
// with placeholders.
//
// We use ASCII control characters STX (\x02) and ETX (\x03) as delimiters.
// Unlike `_` or `$`, these control characters trigger absolutely no Markdown formatting 
// rules (like emphasis or code blocks) and are safely passed through as raw text.
//
// Namespaces:
//   \x02JSXn\x03   — Represents a JSX component block (where n is the pool index).
//   \x02MATHBn\x03 — Represents a block math equation ($$…$$).
//   \x02MATHIn\x03 — Represents an inline math equation ($…$).

const PFX_JSX:    &str = "\x02JSX";
const PFX_MATHB:  &str = "\x02MATHB";
const PFX_MATHI:  &str = "\x02MATHI";
const SFX:        &str = "\x03";

pub(crate) fn make_placeholder(n: usize) -> String {
    format!("{}{}{}", PFX_JSX, n, SFX)
}
fn make_mathb_placeholder(n: usize) -> String {
    format!("{}{}{}", PFX_MATHB, n, SFX)
}
fn make_mathi_placeholder(n: usize) -> String {
    format!("{}{}{}", PFX_MATHI, n, SFX)
}

/// Masque les `<`, `>` et `$` à l'intérieur des blocs de code fencés et inline
/// en les remplaçant par \x01 avant que extract_math et extract_jsx ne tournent.
/// Les positions sont préservées — pulldown-cmark re-parsera le vrai input.
pub fn mask_code_blocks(input: &str) -> String {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut out = bytes.to_vec();
    let mut i = 0;
 
    while i < len {
        // Bloc fencé ```...```
        if i + 2 < len && bytes[i] == b'`' && bytes[i+1] == b'`' && bytes[i+2] == b'`' {
            i += 3;
            // Skip la ligne d'info du langage
            while i < len && bytes[i] != b'\n' { i += 1; }
            // Masquer jusqu'à la fermeture ```
            while i < len {
                if i + 2 < len && bytes[i] == b'`' && bytes[i+1] == b'`' && bytes[i+2] == b'`' {
                    i += 3;
                    break;
                }
                if out[i] == b'<' || out[i] == b'>' || out[i] == b'$' {
                    out[i] = b'\x01';
                }
                i += 1;
            }
            continue;
        }
 
        // Code inline `...`
        if bytes[i] == b'`' {
            i += 1;
            while i < len && bytes[i] != b'`' {
                if out[i] == b'<' || out[i] == b'>' || out[i] == b'$' {
                    out[i] = b'\x01';
                }
                i += 1;
            }
            if i < len { i += 1; }
            continue;
        }
 
        i += 1;
    }
 
    // Safety: on n'a remplacé que des bytes ASCII par d'autres bytes ASCII (\x01)
    unsafe { String::from_utf8_unchecked(out) }
}

/// Extracts LaTeX math blocks before any other parsing occurs.
///
/// # Why do this first?
/// Math expressions frequently contain characters that conflict with HTML/JSX, 
/// most notably the greater-than sign (e.g., in `$t > 0$`). If we don't extract 
/// the math first, the JSX lexer might see the `>` and mistakenly assume it 
/// closes a JSX tag, causing fatal parsing errors.
///
/// # Returns
/// A tuple containing:
/// 1. The processed `String` with math replaced by placeholders.
/// 2. A `Vec<String>` (pool) of block math formulas (`$$`).
/// 3. A `Vec<String>` (pool) of inline math formulas (`$`).
pub fn extract_math(input: &str) -> (String, Vec<String>, Vec<String>) {
    let mut block_pool:  Vec<String> = Vec::new();
    let mut inline_pool: Vec<String> = Vec::new();
    let mut out = String::with_capacity(input.len());

    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Block math: $$…$$
        if i + 1 < len && chars[i] == '$' && chars[i + 1] == '$' {
            let start = i + 2;
            let mut j = start;
            while j + 1 < len && !(chars[j] == '$' && chars[j + 1] == '$') {
                j += 1;
            }
            if j + 1 < len {
                let math: String = chars[start..j].iter().collect();
                out.push_str(&make_mathb_placeholder(block_pool.len()));
                block_pool.push(math);
                i = j + 2;
                continue;
            }
            // If the $$ block is never closed, emit it as literal text to avoid eating the rest of the file.
        }

        // Inline math: $…$ (not $$)
        if chars[i] == '$' && (i == 0 || chars[i - 1] != '$') {
            let start = i + 1;
            let mut j = start;
            while j < len && chars[j] != '$' {
                j += 1;
            }
            if j < len && j > start {
                let math: String = chars[start..j].iter().collect();
                out.push_str(&make_mathi_placeholder(inline_pool.len()));
                inline_pool.push(math);
                i = j + 1;
                continue;
            }
        }

        out.push(chars[i]);
        i += 1;
    }

    (out, block_pool, inline_pool)
}

/// Converts a pre-processed Markdown string into a flattened Abstract Syntax Tree (AST).
///
/// This function drives the `pulldown-cmark` event loop. It intercepts Markdown events,
/// translates them into our custom `AstNode` structures, and dynamically expands the 
/// placeholders back into their original JSX or Math components.
pub fn parse_markdown(
    text:        &str,
    jsx:         &[String],
    block_math:  &[String],
    inline_math: &[String],
) -> Result<Vec<AstNode>, ParseError> {
    let parser = Parser::new_ext(text, Options::all());

    // A single, highly efficient regex to match all three placeholder types simultaneously.
    let ph_re = Regex::new(
        r"\x02(JSX|MATHB|MATHI)(\d+)\x03"
    ).expect("static regex is valid");

    let mut stack: Vec<AstNode> = Vec::new();
    let mut root:  Vec<AstNode> = Vec::new();

    for event in parser {
        match event {
            // HTML/Markdown Structural Tags (e.g., <p>, <h1>)
            Event::Start(tag) => {
                let mut node = AstNode::element(map_tag(&tag), false);
            
                match &tag {
                    Tag::Link { dest_url, title, .. } => {
                        let attrs = node.attributes.get_or_insert_with(std::collections::HashMap::new);

                        if !dest_url.is_empty() {
                            attrs.insert(
                                "href".to_string(),
                                crate::ast::AttrValue::Text(dest_url.to_string()),
                            );
                        }
                        if !title.is_empty() {
                            attrs.insert(
                                "title".to_string(),
                                crate::ast::AttrValue::Text(title.to_string()),
                            );
                        }
                    }
                    Tag::Image { dest_url, title, .. } => {
                        let attrs = node.attributes.get_or_insert_with(std::collections::HashMap::new);

                        if !dest_url.is_empty() {
                            attrs.insert(
                                "src".to_string(),
                                crate::ast::AttrValue::Text(dest_url.to_string()),
                            );
                        }
                        if !title.is_empty() {
                            attrs.insert(
                                "title".to_string(),
                                crate::ast::AttrValue::Text(title.to_string()),
                            );
                        }
                    }
                    _ => {}
                }
            
                stack.push(node);
            }

            Event::End(_) => {
                if let Some(node) = stack.pop() {
                    // Critical fix for React: prevent block-level elements from being trapped in <p> tags.
                    let node = unwrap_solo_jsx_paragraph(node);
                    push_child(node, &mut stack, &mut root);
                }
            }

            // Text Processing & Placeholder Expansion
            Event::Text(ref text_cow) => {
                expand_text(text_cow, &ph_re, jsx, &block_math, &inline_math,
                            &mut stack, &mut root)?;
            }

            // Code Blocks (`code`)
            Event::Code(code) => {
                let mut node = AstNode::element("code", false);
                node.content = Some(code.to_string());
                push_child(node, &mut stack, &mut root);
            }

            // Raw HTML Elements
            Event::Html(html) => {
                let mut node = AstNode::element("html", false);
                node.content = Some(html.to_string());
                push_child(node, &mut stack, &mut root);
            }

            // Line Breaks and Rules
            Event::SoftBreak | Event::HardBreak => {
                push_child(AstNode::element("br", true), &mut stack, &mut root);
            }

            Event::Rule => {
                push_child(AstNode::element("hr", true), &mut stack, &mut root);
            }

            _ => {}
        }
    }

    Ok(root)
}

/// Scans text nodes for STX/ETX control character placeholders and reinjects the raw data.
/// If a JSX placeholder is found, it triggers the recursive `parse_jsx` function to 
/// build the sub-tree on the fly.
fn expand_text(
    text:        &str,
    ph_re:       &Regex,
    jsx:         &[String],
    block_math:  &[String],
    inline_math: &[String],
    stack:       &mut Vec<AstNode>,
    root:        &mut Vec<AstNode>,
) -> Result<(), ParseError> {
    let mut last = 0usize;

    for cap in ph_re.captures_iter(text) {
        let m    = cap.get(0).unwrap();
        let kind = &cap[1];
        let idx: usize = cap[2].parse().expect("regex guarantees digits");

        // Push any plain text that occurred before the placeholder.
        if m.start() > last {
            let t = AstNode::text(&text[last..m.start()]);
            push_child(t, stack, root);
        }

        let node = match kind {
            "JSX" => {
                let raw = jsx.get(idx).ok_or(ParseError::UnexpectedToken {
                    pos: idx, got: '?',
                })?;
                parse_jsx(raw, block_math, inline_math)?
            }
            "MATHB" => {
                let math = block_math.get(idx).ok_or(ParseError::UnexpectedToken {
                    pos: idx, got: '?',
                })?;
                let mut n = AstNode::element("BlockMath", true);
                n.content = Some(math.trim().to_string());
                n
            }
            "MATHI" => {
                let math = inline_math.get(idx).ok_or(ParseError::UnexpectedToken {
                    pos: idx, got: '?',
                })?;
                let mut n = AstNode::element("InlineMath", true);
                n.content = Some(math.trim().to_string());
                n
            }
            _ => unreachable!(),
        };

        push_child(node, stack, root);
        last = m.end();
    }

    // Push any remaining text after the last placeholder.
    if last < text.len() {
        push_child(AstNode::text(&text[last..]), stack, root);
    }

    Ok(())
}

/// Attaches a newly parsed node to the AST tree, flattening synthetic "fragment" nodes.
fn push_child(node: AstNode, stack: &mut Vec<AstNode>, root: &mut Vec<AstNode>) {
    // A `fragment` node is a synthetic container produced by `unwrap_solo_jsx_paragraph`.
    // We flatten its children directly into the parent to maintain a clean DOM hierarchy.
    if node.node_type == "fragment" {
        for child in node.children {
            push_child(child, stack, root);
        }
        return;
    }
    if let Some(parent) = stack.last_mut() {
        parent.children.push(node);
    } else {
        root.push(node);
    }
}

/// Validates and fixes the hierarchy of Paragraph `<p>` nodes.
///
/// React strictly forbids rendering block-level elements (like `<div>` or custom components)
/// inside an inline `<p>` tag (Warning: `validateDOMNesting`). Because Markdown parsers 
/// aggressively wrap loose text in `<p>` tags, this function performs post-processing to fix the tree.
fn unwrap_solo_jsx_paragraph(node: AstNode) -> AstNode {
    if node.node_type != "p" {
        return node;
    }

    let is_block = |n: &AstNode| -> bool {
        // Inline math is safe inside a paragraph.
        if n.node_type == "InlineMath" {
            return false;
        }
        // Custom JSX components (uppercase) and BlockMath are strictly block-level.
        n.node_type.chars().next()
            .map(|c| c.is_ascii_uppercase())
            .unwrap_or(false)
    };

    let meaningful: Vec<usize> = node.children.iter().enumerate().filter_map(|(i, c)| {
        if c.node_type == "text" && c.content.as_deref()
            .map(|s| s.trim().is_empty()).unwrap_or(true) {
            return None;
        }
        Some(i)
    }).collect();

    // Case 1: Single meaningful block-level child
    // E.g., `<p><Chart /></p>` -> Hoist out the child to return just `<Chart />`.
    if meaningful.len() == 1 {
        let child = &node.children[meaningful[0]];
        if is_block(child) {
            return node.children.into_iter().nth(meaningful[0]).unwrap();
        }
    }

    // Case 2: Mixed inline and block content
    // E.g., `<p>Hello <Chart /> World</p>`
    // We split this into a flat fragment sequence: `[<p>Hello</p>, <Chart />, <p>World</p>]`.
    let has_block = node.children.iter().any(|c| is_block(c));
    if !has_block {
        return node; // Pure inline paragraph, totally valid HTML.
    }

    let mut fragments: Vec<AstNode> = Vec::new();
    let mut inline_buf: Vec<AstNode> = Vec::new();

    for child in node.children {
        if is_block(&child) {
            // Flush the accumulated inline buffer as a distinct <p> tag.
            let trimmed: Vec<AstNode> = inline_buf.drain(..)
                .skip_while(|n| n.node_type == "text"
                    && n.content.as_deref().map(|s| s.trim().is_empty()).unwrap_or(true))
                .collect();
            if !trimmed.is_empty() {
                let mut p = AstNode::element("p", false);
                p.children = trimmed;
                fragments.push(p);
            }
            fragments.push(child);
        } else {
            inline_buf.push(child);
        }
    }
    // Flush any trailing inline content.
    let trimmed: Vec<AstNode> = inline_buf.into_iter()
        .skip_while(|n| n.node_type == "text"
            && n.content.as_deref().map(|s| s.trim().is_empty()).unwrap_or(true))
        .collect();
    if !trimmed.is_empty() {
        let mut p = AstNode::element("p", false);
        p.children = trimmed;
        fragments.push(p);
    }

    // Wrap in a synthetic `fragment` node that `push_child` knows how to flatten.
    let mut frag = AstNode::element("fragment", false);
    frag.children = fragments;
    frag
}

/// Maps pulldown-cmark structural tags to standard HTML tag names.
fn map_tag(tag: &Tag) -> String {
    match tag {
        Tag::Paragraph       => "p".into(),
        Tag::Heading { level, .. } => format!("h{}", *level as u8),
        Tag::BlockQuote      => "blockquote".into(),
        Tag::CodeBlock(_)    => "pre".into(),
        Tag::List(Some(_))   => "ol".into(),
        Tag::List(None)      => "ul".into(),
        Tag::Item            => "li".into(),
        Tag::Emphasis        => "em".into(),
        Tag::Strong          => "strong".into(),
        Tag::Strikethrough   => "del".into(),
        Tag::Link { .. }     => "a".into(),
        Tag::Image { .. }    => "img".into(),
        Tag::Table(_)        => "table".into(),
        Tag::TableHead       => "thead".into(),
        Tag::TableRow        => "tr".into(),
        Tag::TableCell       => "td".into(),
        _                    => "div".into(),
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_mdx;

    fn parse(src: &str) -> Vec<crate::ast::AstNode> {
        // Always go through the full pipeline so math is extracted before JSX.
        parse_mdx(src).unwrap()
    }

    #[test]
    fn inline_math_produces_node() {
        let nodes = parse("Hello $x^2$ world");
        let p = &nodes[0];
        assert_eq!(p.node_type, "p");
        let has_math = p.children.iter().any(|c| c.node_type == "InlineMath");
        assert!(has_math, "expected InlineMath child");
    }

    #[test]
    fn block_math_produces_node() {
        let nodes = parse("$$\nx^2\n$$");
        let has_math = nodes.iter().any(|n| n.node_type == "BlockMath");
        assert!(has_math, "expected BlockMath node");
    }

    #[test]
    fn jsx_not_wrapped_in_paragraph() {
        let nodes = parse("<Alert />");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].node_type, "Alert");
    }

    #[test]
    fn inline_math_with_gt_inside_jsx() {
        // This was the crash: $t > 0$ inside a JSX block confused the lexer.
        let src = "<Note type=\"warning\">\n  Valid for $t > 0$.\n</Note>";
        let nodes = parse(src);
        assert_eq!(nodes[0].node_type, "Note");
        let has_math = nodes[0].children.iter().any(|c| c.node_type == "InlineMath");
        assert!(has_math, "expected InlineMath inside Note");
    }

    #[test]
    fn jsx_with_inline_math_child() {
        let src = "<Note type=\"info\">\n  See $E=mc^2$\n</Note>";
        let nodes = parse(src);
        assert_eq!(nodes[0].node_type, "Note");
    }
}