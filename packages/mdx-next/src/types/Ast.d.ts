/**
 * Matches the Rust `AttrValue` enum exactly.
 * Defines the various ways an attribute can be evaluated in MDX.
 */
export type AttrValue =
  | { kind: "text";       value: string }
  | { kind: "expression"; value: string }
  | { kind: "boolean" }
  | { kind: "ast";        value: AstNode[] };

export interface AstNode {
  node_type:     string;
  content?:      string;
  self_closing?: boolean;
  attributes?:   Record<string, AttrValue>;
  children?:     AstNode[];
}