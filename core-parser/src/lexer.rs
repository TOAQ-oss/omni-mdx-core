use crate::ast::ParseError;
use crate::markdown::make_placeholder;

/// Scans the input string to extract top-level JSX blocks, cloaking them from the Markdown parser.
///
/// Markdown parsers natively struggle with complex JSX (especially nested components,
/// expressions in braces `{...}`, and multi-line attributes). This function acts as a pre-processor:
/// it finds JSX blocks, lifts them into a secondary `pool`, and replaces the original text
/// with a safe, alphanumeric placeholder (e.g., `\x02JSX0\x03`).
///
/// # Arguments
/// * `input` - The raw MDX string to be processed.
///
/// # Returns
/// A tuple containing:
/// 1. The cloaked `String` safe for Markdown parsing.
/// 2. A `Vec<String>` (the pool) containing the raw, extracted JSX blocks in order of their placeholders.
///
/// # Errors
/// Returns [`ParseError::UnclosedJsxBlock`] if a JSX tag is opened but the file ends before it closes.
/// Returns [`ParseError::InvalidUtf8`] if the byte slicing corrupts the string encoding.
pub fn extract_jsx(input: &str) -> Result<(String, Vec<String>), ParseError> {
    let mut pool: Vec<String> = Vec::new();
    let mut output = String::with_capacity(input.len());

    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Detect the start of a custom JSX block: `<` followed strictly by an ASCII uppercase letter.
        // This explicitly avoids cloaking standard HTML tags (like `<div>` or `<p>`), leaving them for the Markdown parser.
        if bytes[i] == b'<' && i + 1 < len && (bytes[i + 1] as char).is_ascii_uppercase() {
            let start = i;

            // Advance `i` to the end of the JSX block using the state machine.
            scan_jsx_block_pub(bytes, &mut i, len)
                .map_err(|_| ParseError::UnclosedJsxBlock { pos: start })?;

            let block = std::str::from_utf8(&bytes[start..i])
                .map_err(|_| ParseError::InvalidUtf8)?
                .to_string();

            let placeholder = make_placeholder(pool.len());
            pool.push(block);
            output.push_str(&placeholder);
            continue;
        }

        // Not a JSX block — advance by one full Unicode character, not just one byte.
        // This is crucial to prevent slicing multi-byte UTF-8 sequences (e.g., emojis, accents like 'é')
        // down the middle, which would result in fatal Mojibake or Panics.
        let ch_len = utf8_char_len(bytes[i]);
        let ch_end = (i + ch_len).min(len);
        if let Ok(s) = std::str::from_utf8(&bytes[i..ch_end]) {
            output.push_str(s);
        }
        i = ch_end;
    }

    Ok((output, pool))
}

// Core state machine to traverse a JSX block and find its strict boundary.
///
/// This function operates directly on `&[u8]` for maximum performance, avoiding the overhead
/// of UTF-8 boundary checks at every step. It intelligently handles quotes, nested braces,
/// and self-closing tags to determine exactly where the root component ends.
///
/// # State Variables
/// * `depth` - Tracks the nesting level of paired tags (`<Box>` increases it, `</Box>` decreases it).
/// * `in_tag` - True if the parser is currently scanning attributes inside an opening tag (`<Tag ... >`).
/// * `quote` - Holds the current quote character (`"`, `'`, or `` ` ``) if inside a string literal.
/// * `brace` - Tracks nesting of JSX expressions (`{...}`) to ignore `<` or `>` inside them.
/// * `last_slash` - Flags if the last non-whitespace character in a tag was `/`, indicating a self-closing tag `/>`.
///
/// On entry, `*i` must point at the opening `<`.
/// On success, `*i` points one byte past the closing `>`.
pub(crate) fn scan_jsx_block_pub(bytes: &[u8], i: &mut usize, len: usize) -> Result<(), ()> {
    let mut depth: i32 = 0;
    let mut quote: Option<u8> = None;
    let mut in_tag = false;
    let mut last_slash = false;
    let mut brace: i32 = 0;

    while *i < len {
        let c = bytes[*i];

        // 1. If we are inside a string literal, ignore all structural characters until the closing quote.
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
            *i += 1;
            continue;
        }

        if in_tag {
            // 2. We are scanning inside the brackets of a tag (e.g., evaluating attributes).
            match c {
                b'"' | b'\'' | b'`' => quote = Some(c),
                b'{' => brace += 1,
                b'}' => {
                    brace -= 1;
                    if brace < 0 {
                        brace = 0;
                    }
                }
                b'/' if brace == 0 => last_slash = true,
                b'>' if brace == 0 => {
                    in_tag = false;
                    if last_slash {
                        // It was a self-closing tag (e.g., `<Foo />`).
                        depth -= 1;
                    }
                    // If we closed the root tag, the block scan is complete.
                    last_slash = false;
                    if depth == 0 {
                        *i += 1;
                        return Ok(());
                    }
                }
                b' ' | b'\t' | b'\n' | b'\r' => { /* Whitespace doesn't negate a preceding slash */
                }
                _ => {
                    if brace == 0 {
                        last_slash = false;
                    }
                }
            }
        } else {
            // 3. We are scanning children/text outside of tag brackets.
            match c {
                b'<' if *i + 1 < len => {
                    let next = bytes[*i + 1];
                    if next == b'/' {
                        // Found a closing tag `</...>` — consume it until `>`.
                        *i += 2;
                        while *i < len && bytes[*i] != b'>' {
                            *i += 1;
                        }
                        depth -= 1;
                        // Root block fully closed.
                        if depth == 0 {
                            *i += 1;
                            return Ok(());
                        }
                    } else if (next as char).is_ascii_alphabetic() || next == b'_' {
                        // Found a new opening tag `<Child>`.
                        depth += 1;
                        in_tag = true;
                        last_slash = false;
                    }
                    // Any other sequence (like `<=`) is treated as plain text.
                }
                _ => {}
            }
        }

        *i += 1;
    }

    // End of file reached before depth returned to 0.
    Err(())
}

/// Determines the byte length of a UTF-8 character based on its leading byte.
///
/// This prevents the lexer from getting stuck or panicking when encountering multi-byte
/// characters (like accents or emojis) by advancing the exact number of bytes required
/// to consume the full codepoint.
#[inline]
fn utf8_char_len(first_byte: u8) -> usize {
    match first_byte {
        0x00..=0x7F => 1, // Standard ASCII
        0xC0..=0xDF => 2, // Latin accents, etc.
        0xE0..=0xEF => 3, // Asian scripts, complex symbols
        0xF0..=0xF7 => 4, // Emojis and rare symbols
        _ => 1,           // Fallback for continuation bytes to prevent infinite loops
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_closing_no_space() {
        let (md, pool) = extract_jsx("hello <Foo/> world").unwrap();
        assert_eq!(pool, vec!["<Foo/>"]);
        assert!(md.contains("\x02JSX0\x03"));
    }

    #[test]
    fn self_closing_with_space() {
        let (md, pool) = extract_jsx("hello <Foo /> world").unwrap();
        assert_eq!(pool, vec!["<Foo />"]);
        assert!(md.contains("\x02JSX0\x03"));
    }

    #[test]
    fn self_closing_with_attrs() {
        let (_, pool) = extract_jsx(r#"<Alert type="error" />"#).unwrap();
        assert_eq!(pool[0], r#"<Alert type="error" />"#);
    }

    #[test]
    fn paired_tag() {
        let (md, pool) = extract_jsx("<Box>hello</Box>").unwrap();
        assert_eq!(pool[0], "<Box>hello</Box>");
        assert!(md.contains("\x02JSX0\x03"));
    }

    #[test]
    fn nested_tags() {
        let (_, pool) = extract_jsx("<Outer><Inner /></Outer>").unwrap();
        assert_eq!(pool[0], "<Outer><Inner /></Outer>");
    }

    #[test]
    fn angle_bracket_in_quoted_attr() {
        let src = r#"<Comp label="a < b" />"#;
        let (_, pool) = extract_jsx(src).unwrap();
        assert_eq!(pool[0], src);
    }

    #[test]
    fn unclosed_block_returns_error() {
        assert!(extract_jsx("<Foo").is_err());
    }

    #[test]
    fn multiple_siblings() {
        let (md, pool) = extract_jsx("<A/> text <B/>").unwrap();
        assert_eq!(pool.len(), 2);
        assert!(md.contains("\x02JSX0\x03") && md.contains("\x02JSX1\x03"));
    }

    #[test]
    fn utf8_text_not_corrupted() {
        let (md, _) = extract_jsx("Texte avec défile et à gauche.").unwrap();
        assert_eq!(md, "Texte avec défile et à gauche.");
    }
}
