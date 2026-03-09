use crate::ast::{AstNode, ParseError};
use crate::lexer::extract_jsx;
use crate::markdown::{extract_math, parse_markdown};

/// The primary orchestrator for the entire MDX to AST parsing pipeline.
///
/// The execution order here is strictly deliberate and solves common MDX parsing bugs:
/// 
/// 1. **Math Extraction First:** LaTeX math blocks often contain structural symbols like `<` or `>`. 
///    By extracting math `$…$` and `$$…$$` into placeholders immediately, we guarantee that equations 
///    like `$x > y$` won't confuse the subsequent JSX lexer into closing a tag prematurely.
/// 
/// 2. **JSX Lexing:** Once the math is cloaked, we safely extract custom React components (`<UpperCase…>`).
///    These are also replaced with placeholders so the underlying Markdown engine doesn't mangle them.
/// 
/// 3. **Markdown Parsing & Reassembly:** Finally, we run the text (now just standard markdown + placeholders) 
///    through `pulldown-cmark`. As the text events are generated, we expand the placeholders back into 
///    fully typed, nested `AstNode` trees.
pub fn parse_mdx(input: &str) -> Result<Vec<AstNode>, ParseError> {
    // Step 1 — Safe math extraction (operates on the raw input).
    let (after_math, block_math, inline_math) = extract_math(input);

    // Step 2 — Safe JSX extraction (input is now immune to math-symbol collisions).
    let (markdown, jsx_pool) = extract_jsx(&after_math)?;

    // Step 3 — Markdown parsing and dynamic placeholder expansion.
    parse_markdown(&markdown, &jsx_pool, &block_math, &inline_math)
}
