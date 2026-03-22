// Opcodes for the Omni-Core Binary Protocol (OCP)
// Node Types
pub const NODE_TEXT: u8 = 0x01;
pub const NODE_ELEMENT: u8 = 0x02;

// Attribute Types (Based on the AttrValue enum)
pub const ATTR_TEXT: u8 = 0x10;
/// Evaluated JavaScript expression (e.g., count={42})
pub const ATTR_EXPRESSION: u8 = 0x11;
/// Implicit boolean attribute (e.g., disabled)
pub const ATTR_BOOLEAN: u8 = 0x12;
/// Attribute containing an MDX subtree (e.g., render props)
pub const ATTR_AST: u8 = 0x13;
