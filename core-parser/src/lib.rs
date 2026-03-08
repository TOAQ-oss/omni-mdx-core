use wasm_bindgen::prelude::*;

pub mod ast;
pub mod parser;

#[wasm_bindgen]
pub fn parse_mdx_to_json(markdown_input: &str) -> String {
    parser::parse_to_ast(markdown_input)
}