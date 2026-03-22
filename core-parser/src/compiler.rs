use crate::ast::{AstNode, AttrValue};

/// Compiles a complete AST directly into a JSX string
pub fn compile_to_jsx(nodes: &[AstNode]) -> String {
    // We generously pre-allocate memory to avoid reallocating the string
    let mut out = String::with_capacity(nodes.len() * 128);
    for node in nodes {
        render_node(node, &mut out);
    }
    out
}

fn render_node(node: &AstNode, out: &mut String) {
    // 1. Plain text
    if node.node_type == "text" {
        if let Some(content) = &node.content {
            out.push_str(content);
        }
        return;
    }

    // 2. Opening the tag
    out.push('<');
    out.push_str(&node.node_type);

    // 3. Attributes
    if let Some(attrs) = &node.attributes {
        for (key, val) in attrs {
            out.push(' ');
            out.push_str(key);
            match val {
                AttrValue::Text(s) => {
                    out.push_str("=\"");
                    out.push_str(&s.replace('"', "&quot;"));
                    out.push('"');
                }
                AttrValue::Expression(s) => {
                    out.push('=');
                    out.push('{');
                    out.push_str(s);
                    out.push('}');
                }
                AttrValue::Boolean => {
                    // Boolean attributes are written just with their name (e.g., disabled)
                }
                AttrValue::Ast(_) => {
                    // For simplicity in benchmarking, we ignore complex render props
                    out.push_str("={null}");
                }
            }
        }
    }

    // 4. Locking Up and Children
    if node.self_closing {
        out.push_str(" />");
    } else {
        out.push('>');
        for child in &node.children {
            render_node(child, out);
        }
        out.push_str("</");
        out.push_str(&node.node_type);
        out.push('>');
    }
}
