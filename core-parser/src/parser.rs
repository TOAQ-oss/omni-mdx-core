use crate::ast::{AstNode, ParseError};
use crate::lexer::extract_jsx;
use crate::markdown::{extract_math, mask_code_blocks, parse_markdown};

pub fn parse_mdx(input: &str) -> Result<Vec<AstNode<'static>>, ParseError> {
    // Guard: reject inputs that are too large
    if input.len() > 500_000 {
        return Err(ParseError::InputTooLong);
    }

    if input.as_bytes().iter().any(|&b| b == 0) {
        return Err(ParseError::InvalidUtf8);
    }

    let mut structural_symbols = 0;
    for b in input.as_bytes() {
        match b {
            b'-' | b'*' | b'_' | b'#' | b'>' | b'[' | b'|' => structural_symbols += 1,
            _ => {}
        }
    }

    if structural_symbols > 5000 {
        return Err(ParseError::InputTooLong);
    }

    // 1. Code block protection
    let protected = mask_code_blocks(input);

    // 2. Extracting the math (LaTeX)
    // We pass &protected (which is a Cow<str>) to ensure we're working on the hidden text
    let (after_math, block_math, inline_math) = extract_math(&protected);

    // 3. Extracting JSX (Components)
    // We continue the chain with the result from the previous step
    let (markdown, jsx_pool) = extract_jsx(&after_math)?;

    // 4. Parsing Markdown and expanding placeholders
    // Here, `parse_markdown` will call `unmask_code` to restore the angle brackets in the ``` and ``` blocks
    let nodes = parse_markdown(&markdown, &jsx_pool, &block_math, &inline_math)?;

    // 5. Final conversion to ‘static’ to free up local references
    Ok(nodes.into_iter().map(|n| n.into_static()).collect())
}
