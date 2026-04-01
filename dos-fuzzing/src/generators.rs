use rand::Rng;

/// Represents a "Russian Doll" attack vector.
/// Example: `prefix` + `repeating` * N + `suffix`
pub struct Pattern {
    pub prefix: String,
    pub repeating: String,
    pub suffix: String,
}

/// A comprehensive dictionary covering MDX, HTML, LaTeX, and basic Markdown.
/// Designed to trigger complex AST nesting and state-machine edge cases.
const TOKENS: &[&str] = &[
    // Core Markdown
    " [^",
    "]\n",
    "* ",
    "    ",
    "> ",
    "---",
    "![",
    "`",
    "```\n",
    // JSX & MDX Components
    "<Box>",
    "</Box>",
    "<Speaker ",
    " data={{",
    "}}>",
    "/>",
    "{/*",
    "*/}",
    // Math & LaTeX (Often causes catastrophic backtracking)
    "$$",
    "$",
    "\\begin{equation}",
    "\\frac{1}{2}",
    "_",
    "^",
    // MDX ESM Imports/Exports
    "export const ",
    "import { ",
    "} from '",
];

/// Generates a randomized Russian Doll pattern.
pub fn generate_russian_doll(rng: &mut impl Rng, parts_len: usize) -> Pattern {
    let mut prefix = String::new();
    let mut repeating = String::new();
    let mut suffix = String::new();

    for _ in 0..parts_len {
        prefix.push_str(TOKENS[rng.gen_range(0..TOKENS.len())]);
    }
    for _ in 0..parts_len {
        repeating.push_str(TOKENS[rng.gen_range(0..TOKENS.len())]);
    }
    for _ in 0..parts_len {
        suffix.push_str(TOKENS[rng.gen_range(0..TOKENS.len())]);
    }

    Pattern {
        prefix,
        repeating,
        suffix,
    }
}
