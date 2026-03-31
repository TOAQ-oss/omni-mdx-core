use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use regex::Regex;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::ast::{AstNode, AttrValue, ParseError};
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

static PH_RE: OnceLock<Regex> = OnceLock::new();

fn get_ph_re() -> &'static Regex {
    PH_RE.get_or_init(|| {
        Regex::new(r"\x02(JSX|MATHB|MATHI)(\d+)\x03").expect("static regex is valid")
    })
}

const PFX_JSX: &str = "\x02JSX";
const PFX_MATHB: &str = "\x02MATHB";
const PFX_MATHI: &str = "\x02MATHI";
const SFX: &str = "\x03";

pub(crate) fn make_placeholder(n: usize) -> String {
    format!("{}{}{}", PFX_JSX, n, SFX)
}
fn make_mathb_placeholder(n: usize) -> String {
    format!("{}{}{}", PFX_MATHB, n, SFX)
}
fn make_mathi_placeholder(n: usize) -> String {
    format!("{}{}{}", PFX_MATHI, n, SFX)
}

/// Masks `<`, `>`, and `$` within fenced and inline code blocks
/// by replacing them with \x01 before `extract_math` and `extract_jsx` run.
/// Positions are preserved — `pulldown-cmark` will re-parse the actual input.
pub fn mask_code_blocks(input: &str) -> Cow<'_, str> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut out: Option<Vec<u8>> = None;
    let mut i = 0;

    while i < len {
        // --- Fenced Blocks ``` ---
        if i + 2 < len && bytes[i] == b'`' && bytes[i + 1] == b'`' && bytes[i + 2] == b'`' {
            i += 3;
            // Skip the language identifier (e.g., ```html)
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }

            while i < len {
                // Look for closing fence
                if i + 2 < len && bytes[i] == b'`' && bytes[i + 1] == b'`' && bytes[i + 2] == b'`' {
                    i += 3;
                    break;
                }

                // MASK EVERYTHING inside the fence
                if bytes[i] == b'<' || bytes[i] == b'>' || bytes[i] == b'$' {
                    if out.is_none() {
                        out = Some(bytes.to_vec());
                    }
                    let masked = match bytes[i] {
                        b'<' => 0x01,
                        b'>' => 0x04,
                        b'$' => 0x05,
                        _ => bytes[i],
                    };
                    out.as_mut().unwrap()[i] = masked;
                }
                i += 1;
            }
            continue;
        }

        // --- Inline Code ` ---
        if bytes[i] == b'`' {
            i += 1;
            while i < len && bytes[i] != b'`' {
                if bytes[i] == b'<' || bytes[i] == b'>' || bytes[i] == b'$' {
                    if out.is_none() {
                        out = Some(bytes.to_vec());
                    }
                    let masked = match bytes[i] {
                        b'<' => 0x01,
                        b'>' => 0x04,
                        b'$' => 0x05,
                        _ => bytes[i],
                    };
                    out.as_mut().unwrap()[i] = masked;
                }
                i += 1;
            }
            if i < len {
                i += 1;
            }
            continue;
        }
        i += 1;
    }

    match out {
        Some(v) => Cow::Owned(unsafe { String::from_utf8_unchecked(v) }),
        None => Cow::Borrowed(input),
    }
}

fn unmask_code(s: String) -> String {
    let mut bytes = s.into_bytes();
    for b in bytes.iter_mut() {
        match *b {
            0x01 => *b = b'<',
            0x04 => *b = b'>',
            0x05 => *b = b'$',
            _ => (),
        }
    }
    unsafe { String::from_utf8_unchecked(bytes) }
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
pub fn extract_math(input: &str) -> (Cow<'_, str>, Vec<String>, Vec<String>) {
    let mut block_pool: Vec<String> = Vec::new();
    let mut inline_pool: Vec<String> = Vec::new();
    let mut has_math = false;

    // First, we check whether we need to allocate a new string
    if !input.contains('$') {
        return (Cow::Borrowed(input), block_pool, inline_pool);
    }

    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
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
                has_math = true;
                continue;
            }
        }
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
                has_math = true;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }

    if has_math {
        (Cow::Owned(out), block_pool, inline_pool)
    } else {
        (Cow::Borrowed(input), block_pool, inline_pool)
    }
}

/// Enforces structural limits on the Markdown input to prevent excessive parsing times
/// and potential resource exhaustion during the AST generation phase.
fn verify_markdown_safety(text: &str) -> Result<(), ParseError> {
    if text.len() > 2_000_000 {
        return Err(ParseError::InputTooLong);
    }

    if text.matches("[^").count() > 100 {
        return Err(ParseError::ComplexityLimitExceeded(
            "ParseError: Document complexity limit exceeded (too many specific syntax tokens).".to_string()
        ));
    }

    if text.matches("-  -").count() > 50 || text.matches("- - -").count() > 50 {
        return Err(ParseError::ComplexityLimitExceeded(
            "ParseError: Excessive structural ambiguity detected.".to_string()
        ));
    }

    if text.matches(">>>>>>>>>").count() > 0 {
        return Err(ParseError::ComplexityLimitExceeded(
            "ParseError: Excessive nesting depth detected.".to_string()
        ));
    }

    let mut symbol_streak = 0;
    let mut max_symbol_streak = 0;

    for c in text.chars() {
        if !c.is_alphanumeric() && !c.is_whitespace() {
            symbol_streak += 1;
            if symbol_streak > max_symbol_streak {
                max_symbol_streak = symbol_streak;
            }
        } else if c.is_alphanumeric() {
            symbol_streak = 0;
        }

        if max_symbol_streak > 250 {
             return Err(ParseError::ComplexityLimitExceeded(
                "ParseError: Malformed document (abnormal symbol density).".to_string()
            ));
        }
    }

    Ok(())
}

/// Converts a pre-processed Markdown string into a flattened Abstract Syntax Tree (AST).
///
/// This function drives the `pulldown-cmark` event loop. It intercepts Markdown events,
/// translates them into our custom `AstNode` structures, and dynamically expands the
/// placeholders back into their original JSX or Math components.
pub fn parse_markdown<'a>(
    text: &'a str,
    jsx: &'a [String],
    block_math: &'a [String],
    inline_math: &'a [String],
) -> Result<Vec<AstNode<'a>>, ParseError> {
    if let Err(security_msg) = verify_markdown_safety(text) {
        return Err(ParseError::ComplexityLimitExceeded(security_msg.to_string()));
    }

    let parser = Parser::new_ext(text, Options::all());
    let ph_re = get_ph_re();

    let mut stack: Vec<AstNode<'a>> = Vec::new();
    let mut root: Vec<AstNode<'a>> = Vec::new();
    let mut in_code_block = false;
    let mut in_table_head = false;
    let mut in_table_body = false;

    for event in parser {
        match event {
            Event::Start(tag) => {
                if let Tag::CodeBlock(_) = tag {
                    in_code_block = true;
                }

                if let Tag::TableHead = tag {
                    in_table_head = true;
                    let mut thead_node = AstNode::element("thead", false);
                    let tr_node = AstNode::element("tr", false);
                    thead_node.children.push(tr_node);
                    stack.push(thead_node);
                    continue;
                }

                if let Tag::TableCell = tag {
                    let cell_type = if in_table_head { "th" } else { "td" };
                    let node = AstNode::element(cell_type, false);
                    stack.push(node);
                    continue;
                }

                if let Tag::TableRow = tag {
                    if !in_table_head && !in_table_body {
                        in_table_body = true;
                        stack.push(AstNode::element("tbody", false));
                    }
                    stack.push(AstNode::element("tr", false));
                    continue;
                }

                if let Tag::TableCell = tag {
                    let cell_type = if in_table_head { "th" } else { "td" };
                    stack.push(AstNode::element(cell_type, false));
                    continue;
                }

                let tag_name = map_tag(&tag);
                let mut node = AstNode::element(tag_name, false);
                match &tag {
                    Tag::Link {
                        dest_url, title: _, ..
                    } => {
                        let attrs = node.attributes.get_or_insert_with(HashMap::new);
                        attrs.insert(
                            "href".into(),
                            AttrValue::Text(Cow::Owned(dest_url.to_string())),
                        );
                    }
                    Tag::Image {
                        dest_url, title, ..
                    } => {
                        let attrs = node.attributes.get_or_insert_with(HashMap::new);
                        if !dest_url.is_empty() {
                            attrs.insert(
                                "src".into(),
                                AttrValue::Text(Cow::Owned(dest_url.to_string())),
                            );
                        }
                        if !title.is_empty() {
                            attrs.insert(
                                "title".into(),
                                AttrValue::Text(Cow::Owned(title.to_string())),
                            );
                        }
                    }
                    Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Fenced(lang))
                        if !lang.is_empty() =>
                    {
                        let attrs = node.attributes.get_or_insert_with(HashMap::new);
                        attrs.insert(
                            "className".into(),
                            AttrValue::Text(Cow::Owned(format!("language-{}", lang))),
                        );
                    }
                    _ => {}
                }
                stack.push(node);
            }
            Event::End(tag_end) => {
                if let TagEnd::CodeBlock = tag_end {
                    in_code_block = false;
                }
                if let TagEnd::TableHead = tag_end {
                    in_table_head = false;
                }
                if let TagEnd::Table = tag_end {
                    if in_table_body {
                        in_table_body = false;
                        if let Some(tbody) = stack.pop() {
                            push_child(tbody, &mut stack, &mut root);
                        }
                    }
                }
                if let Some(node) = stack.pop() {
                    let node = unwrap_solo_jsx_paragraph(node);

                    if in_table_head && (node.node_type == "th" || node.node_type == "td") {
                        if let Some(thead) = stack.last_mut() {
                            if let Some(tr) = thead.children.first_mut() {
                                tr.children.push(node);
                                continue;
                            }
                        }
                    }

                    push_child(node, &mut stack, &mut root);
                }
            }
            Event::Text(text_cow) => {
                let owned_text = text_cow.into_string();

                if in_code_block {
                    let cleaned = unmask_code(owned_text);
                    push_child(AstNode::text(cleaned), &mut stack, &mut root);
                } else {
                    expand_text(
                        &owned_text,
                        ph_re,
                        jsx,
                        block_math,
                        inline_math,
                        &mut stack,
                        &mut root,
                    )?;
                }
            }
            Event::Code(code) => {
                let mut node = AstNode::element("code", false);
                node.content = Some(Cow::Owned(unmask_code(code.into_string())));
                push_child(node, &mut stack, &mut root);
            }
            Event::Html(html_cow) => {
                let owned_html = html_cow.into_string();
                expand_text(
                    &owned_html,
                    ph_re,
                    jsx,
                    block_math,
                    inline_math,
                    &mut stack,
                    &mut root,
                )?;
            }
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
fn expand_text<'a>(
    text: &str,
    ph_re: &Regex,
    jsx: &'a [String],
    block_math: &'a [String],
    inline_math: &'a [String],
    stack: &mut Vec<AstNode<'a>>,
    root: &mut Vec<AstNode<'a>>,
) -> Result<(), ParseError> {
    let mut last = 0usize;
    for cap in ph_re.captures_iter(text) {
        let m = cap.get(0).unwrap();
        if m.start() > last {
            let t = crate::ast::AstNode::text(unmask_code(text[last..m.start()].to_string()));
            push_child(t, stack, root);
        }
        let kind = &cap[1];
        let idx: usize = cap[2].parse().unwrap_or(0);
        let node_option: Option<Result<AstNode<'a>, ParseError>> = match kind {
            "JSX" => jsx.get(idx).map(|s| parse_jsx(s, block_math, inline_math)),
            "MATHB" => block_math.get(idx).map(|s| {
                let mut n = AstNode::element("BlockMath", true);
                n.content = Some(Cow::Owned(unmask_code(s.trim().to_string())));
                Ok(n)
            }),
            "MATHI" => inline_math.get(idx).map(|s| {
                let mut n = AstNode::element("InlineMath", true);
                n.content = Some(Cow::Owned(unmask_code(s.trim().to_string())));
                Ok(n)
            }),
            _ => None,
        };

        match node_option {
            Some(Ok(node)) => push_child(node, stack, root),
            Some(Err(e)) => return Err(e),
            _ => {
                push_child(AstNode::text(m.as_str().to_string()), stack, root);
            }
        }

        last = m.end();
    }
    if last < text.len() {
        let final_text = unmask_code(text[last..].to_string());
        push_child(AstNode::text(final_text), stack, root);
    }
    Ok(())
}

/// Attaches a newly parsed node to the AST tree, flattening synthetic "fragment" nodes.
fn push_child<'a>(node: AstNode<'a>, stack: &mut Vec<AstNode<'a>>, root: &mut Vec<AstNode<'a>>) {
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
fn unwrap_solo_jsx_paragraph<'a>(mut node: AstNode<'a>) -> AstNode<'a> {
    if node.node_type != "p" || node.children.is_empty() {
        return node;
    }

    let is_block = |n: &AstNode| -> bool {
        if n.node_type == "InlineMath" {
            return false;
        }
        n.node_type
            .chars()
            .next()
            .map(|c| c.is_ascii_uppercase())
            .unwrap_or(false)
    };

    let has_block = node.children.iter().any(is_block);
    if !has_block {
        return node;
    }

    if node.children.len() == 1 && is_block(&node.children[0]) {
        return node.children.pop().unwrap();
    }

    let mut fragments = Vec::new();
    let mut inline_buf = Vec::new();

    for child in node.children {
        if is_block(&child) {
            if !inline_buf.is_empty() {
                let mut p = AstNode::element("p", false);
                p.children = std::mem::take(&mut inline_buf);
                fragments.push(p);
            }
            fragments.push(child);
        } else {
            inline_buf.push(child);
        }
    }
    if !inline_buf.is_empty() {
        let mut p = AstNode::element("p", false);
        p.children = inline_buf;
        fragments.push(p);
    }

    let mut frag = AstNode::element("fragment", false);
    frag.children = fragments;
    frag
}

/// Maps pulldown-cmark structural tags to standard HTML tag names.
fn map_tag(tag: &Tag) -> Cow<'static, str> {
    match tag {
        Tag::Paragraph => "p".into(),
        Tag::Heading { level, .. } => format!("h{}", *level as u8).into(),
        Tag::BlockQuote => "blockquote".into(),
        Tag::CodeBlock(_) => "pre".into(),
        Tag::List(Some(_)) => "ol".into(),
        Tag::List(None) => "ul".into(),
        Tag::Item => "li".into(),
        Tag::Emphasis => "em".into(),
        Tag::Strong => "strong".into(),
        Tag::Strikethrough => "del".into(),
        Tag::Link { .. } => "a".into(),
        Tag::Image { .. } => "img".into(),
        Tag::Table(_) => "table".into(),
        Tag::TableHead => "thead".into(),
        Tag::TableRow => "tr".into(),
        Tag::TableCell => "td".into(),
        _ => "div".into(),
    }
}
