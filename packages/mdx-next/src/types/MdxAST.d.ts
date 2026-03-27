// AST types (mirror Rust output exactly)

type AttrValueKind =
  | { kind: "text";       value: string }
  | { kind: "expression"; value: string }
  | { kind: "boolean" }
  | { kind: "ast";        value: AstNode[] };

interface AstNode {
  node_type:     string;
  content?:      string;
  self_closing?: boolean;
  child_count?:  number;
  attributes?:   Record<string, AttrValueKind> | string;
  children?:     AstNode[];
}

type MDXComponents = Record<string, React.ComponentType<any>>;

export {
    AttrValueKind,
    AstNode,
    MDXComponents
}