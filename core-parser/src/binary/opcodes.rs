//! Omni-Core Protocol (OCP) Opcodes
//!
//! This module defines the raw byte constants (opcodes) used by the OCP
//! binary encoder and decoder to identify AST node structures and attribute types
//! in the byte stream.

// ============================================================================
// Node Types
// ============================================================================

/// Represents a pure text leaf node.
pub const NODE_TEXT: u8 = 0x01;

/// Represents a structural HTML element or a JSX component node.
pub const NODE_ELEMENT: u8 = 0x02;

// ============================================================================
// Attribute Types (Maps to the `AttrValue` enum)
// ============================================================================

/// A standard string literal attribute (e.g., `id="header"` or `class="btn"`).
pub const ATTR_TEXT: u8 = 0x10;

/// An evaluated JavaScript/JSX expression (e.g., `count={42}`).
pub const ATTR_EXPRESSION: u8 = 0x11;

/// An implicit boolean attribute without an explicit value (e.g., `<input disabled />`).
pub const ATTR_BOOLEAN: u8 = 0x12;

/// An attribute containing a nested MDX subtree (e.g., React render props).
pub const ATTR_AST: u8 = 0x13;
