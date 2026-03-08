use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AstNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub content: Option<String>,
    pub attributes: Option<HashMap<String, String>>,
    pub children: Vec<AstNode>,
}