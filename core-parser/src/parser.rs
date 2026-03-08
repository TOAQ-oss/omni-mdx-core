use pulldown_cmark::{Event, Options, Parser as MarkdownParser, Tag};
use crate::ast::AstNode;
use std::collections::HashMap;
use regex::Regex;

#[derive(Debug, PartialEq)]
enum HtmlAction {
    Open(String, HashMap<String, String>),
    Close(String),
    SelfClosing(String, HashMap<String, String>),
    RawText(String),
}

fn extract_attributes(inner: &str) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    // Cette Regex capture les paires clé="valeur" ou clé={valeur}
    let re = Regex::new(r#"(\w+)=[\{"']([^"'\}]+)["'\}]"#).unwrap();
    
    for cap in re.captures_iter(inner) {
        attrs.insert(cap[1].to_string(), cap[2].to_string());
    }
    attrs
}

fn analyze_html_string(html: &str) -> HtmlAction {
    let trimmed = html.trim();
    if !trimmed.starts_with('<') || !trimmed.ends_with('>') {
        return HtmlAction::RawText(html.to_string());
    }

    let inner = trimmed[1..trimmed.len() - 1].trim();

    if inner.starts_with('/') {
        let tag_name = inner[1..].trim().split_whitespace().next().unwrap_or("").to_string();
        HtmlAction::Close(tag_name)
    } else if inner.ends_with('/') {
        let tag_content = inner[..inner.len() - 1].trim();
        let tag_name = tag_content.split_whitespace().next().unwrap_or("").to_string();
        let attrs = extract_attributes(tag_content);
        HtmlAction::SelfClosing(tag_name, attrs)
    } else {
        let tag_name = inner.split_whitespace().next().unwrap_or("").to_string();
        let attrs = extract_attributes(inner);
        HtmlAction::Open(tag_name, attrs)
    }
}

pub fn parse_to_ast(markdown_input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_MATH);
    options.insert(Options::ENABLE_TABLES);
    let parser = MarkdownParser::new_ext(markdown_input, options);
    let mut root_nodes: Vec<AstNode> = Vec::new();
    let mut stack: Vec<AstNode> = Vec::new();

    for event in parser {
        match event {
            Event::Start(tag) => {
                let node_type = match tag {
                    Tag::Heading { level, .. } => level.to_string(),
                    Tag::Paragraph => "p".to_string(),
                    Tag::BlockQuote(_) => "blockquote".to_string(),
                    Tag::List(_) => "ul".to_string(),
                    Tag::Item => "li".to_string(),
                    _ => "div".to_string(),
                };
                
                stack.push(AstNode { node_type, content: None, attributes: None, children: vec![] });
            }

            Event::InlineMath(math) => {
                let math_node = AstNode {
                    node_type: "inlineMath".to_string(),
                    content: Some(math.into_string()),
                    attributes: None,
                    children: vec![],
                };
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(math_node);
                } else {
                    root_nodes.push(math_node);
                }
            }

            Event::DisplayMath(math) => {
                let math_node = AstNode {
                    node_type: "displayMath".to_string(),
                    content: Some(math.into_string()),
                    attributes: None,
                    children: vec![],
                };
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(math_node);
                } else {
                    root_nodes.push(math_node);
                }
            }

            // GESTION DES BALISES PERSONNALISÉES (Le pont avec TOAQ)
            Event::Html(html) | Event::InlineHtml(html) => {
                match analyze_html_string(&html) {
                    HtmlAction::Open(tag_name, attrs) => {
                        // On empile le nouveau composant React (ex: "Note")
                        stack.push(AstNode {
                            node_type: tag_name,
                            content: None,
                            attributes: Some(attrs),
                            children: vec![]
                        });
                    }
                    HtmlAction::SelfClosing(tag_name, attrs) => {
                        // On l'ajoute directement sans l'empiler (ex: "Chart")
                        let node = AstNode {
                            node_type: tag_name,
                            content: None,
                            attributes: Some(attrs),
                            children: vec![]
                        };
                        if let Some(parent) = stack.last_mut() {
                            parent.children.push(node);
                        } else {
                            root_nodes.push(node);
                        }
                    }
                    HtmlAction::Close(tag_name) => {
                        // VERROU DE SÉCURITÉ ATOMIQUE :
                        // On vérifie que la balise qu'on ferme correspond bien au sommet de la pile.
                        // Si le chercheur a fait une erreur (ex: <div></Note>), on l'ignore silencieusement.
                        let mut should_pop = false;
                        if let Some(top) = stack.last() {
                            if top.node_type == tag_name {
                                should_pop = true;
                            }
                        }

                        if should_pop {
                            if let Some(finished_node) = stack.pop() {
                                if let Some(parent) = stack.last_mut() {
                                    parent.children.push(finished_node);
                                } else {
                                    root_nodes.push(finished_node);
                                }
                            }
                        }
                    }
                    HtmlAction::RawText(text) => {
                        // Si l'analyse échoue, on sauvegarde la donnée sous forme de texte pur
                        let text_node = AstNode { node_type: "text".to_string(), content: Some(text), attributes: None, children: vec![] };
                        if let Some(parent) = stack.last_mut() {
                            parent.children.push(text_node);
                        } else {
                            root_nodes.push(text_node);
                        }
                    }
                }
            }

            Event::Text(text) => {
                let text_node = AstNode { node_type: "text".to_string(), content: Some(text.into_string()), attributes: None, children: vec![] };
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(text_node);
                } else {
                    root_nodes.push(text_node);
                }
            }

            Event::End(_) => {
                if let Some(finished_node) = stack.pop() {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(finished_node);
                    } else {
                        root_nodes.push(finished_node);
                    }
                }
            }
            
            _ => {} 
        }
    }

    // VERROU DE SÉCURITÉ DE FIN DE FICHIER :
    // Si l'utilisateur a oublié de fermer une balise (ex: <Note> à la fin du texte sans </Note>),
    // la pile n'est pas vide. On vide proprement la pile dans la racine pour ne perdre aucune donnée.
    while let Some(finished_node) = stack.pop() {
        if let Some(parent) = stack.last_mut() {
            parent.children.push(finished_node);
        } else {
            root_nodes.push(finished_node);
        }
    }

    serde_json::to_string(&root_nodes).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_jsx_parsing() {
        let markdown = "
# Introduction
<Note>
Ceci est une note importante.
</Note>
<Chart />
<BrokenTag>
        ";
        let json_result = parse_to_ast(markdown);
        
        assert!(json_result.contains("\"type\":\"Note\""), "Le noeud <Note> devrait exister");
        assert!(json_result.contains("\"type\":\"Chart\""), "Le noeud <Chart /> devrait exister");
        assert!(json_result.contains("BrokenTag"), "Le moteur ne doit jamais perdre de données");
    }

    #[test]
    fn test_math_parsing() {
        let markdown = r#"Voici une formule en ligne $E=mc^2$ et un bloc mathématique : $$ \alpha + \beta = \gamma $$"#;
        let json_result = parse_to_ast(markdown);
        
        assert!(json_result.contains("inlineMath"));
        assert!(json_result.contains("E=mc^2"));
        assert!(json_result.contains("displayMath"));
        
        assert!(json_result.contains(r"\\alpha + \\beta = \\gamma"));
    }
}