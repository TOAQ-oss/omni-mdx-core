use crate::ast::{AstNode, ParseError};
use crate::lexer::extract_jsx;
use crate::markdown::{extract_math, mask_code_blocks, parse_markdown};

/// Maximum JSX nesting depth allowed before the parser returns an error.
/// Prevents stack overflow from deeply nested components.
pub const MAX_JSX_DEPTH: usize = 128;

/// Parses an MDX string and converts it into a static Abstract Syntax Tree (AST).
///
/// This is the main entry point of the `omni-mdx-core` engine. It processes Markdown,
/// LaTeX math, and JSX components in a highly secure, multi-pass pipeline.
///
/// # Security & Anti-DoS Features
/// To protect host servers and WebAssembly clients from CPU/Memory exhaustion, this function
/// enforces strict constraints before parsing begins:
/// * **Payload Size Limit:** Rejects inputs larger than 2,000,000 bytes.
/// * **Null-Byte Protection:** Rejects inputs containing `\0` to prevent C-string bridging exploits.
/// * **Algorithmic Complexity Shield:** Limits the number of structural Markdown symbols
///   (like `*`, `_`, `[`, etc.) to 50,000. This prevents catastrophic backtracking and $O(n^2)$
///   thread-freezing attacks caused by highly nested pathological patterns.
///
/// # Parsing Pipeline
/// 1. **Code Block Masking:** Protects raw code blocks from being misinterpreted as JSX or Math.
/// 2. **Math Extraction:** Safely extracts block (`$$`) and inline (`$`) LaTeX equations.
/// 3. **JSX Extraction:** Isolates and validates React/MDX components against `MAX_JSX_DEPTH`.
/// 4. **Markdown Parsing:** Parses standard markdown structure and injects the extracted Math/JSX pools.
/// 5. **Static Allocation:** Converts all node lifetimes to `'static` for safe cross-boundary usage (Python/WASM).
///
/// # Arguments
/// * `input` - The raw MDX string to parse.
///
/// # Returns
/// * `Ok(Vec<AstNode<'static>>)` - The parsed AST.
/// * `Err(ParseError)` - If the input is malformed, too deep, or exceeds security limits.
pub fn parse_mdx(input: &str) -> Result<Vec<AstNode<'static>>, ParseError> {
    // Guard: reject inputs that are too large (2 MB limit)
    if input.len() > 2_000_000 {
        return Err(ParseError::InputTooLong);
    }

    // Guard: prevent unexpected C-string termination issues
    if input.as_bytes().contains(&0) {
        return Err(ParseError::InvalidUtf8);
    }

    // Guard: O(N^2) Pathological pattern protection
    let mut structural_symbols = 0;
    for b in input.as_bytes() {
        match b {
            b'-' | b'*' | b'_' | b'#' | b'>' | b'[' | b'|' => structural_symbols += 1,
            _ => {}
        }
    }

    if structural_symbols > 50_000 {
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
