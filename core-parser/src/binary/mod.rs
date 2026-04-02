//! Omni-Core Protocol (OCP) Binary System
//!
//! This module handles the serialization and deserialization of the AST into a highly
//! compact binary format. This format bypasses the slow JSON parsing overhead of
//! JavaScript engines and is heavily optimized for Little-Endian architectures (WASM).

/// The binary decoder, featuring robust anti-DoS shields and strict depth limits.
pub mod decoder;

/// The binary encoder, responsible for generating contiguous byte streams.
pub mod encoder;

/// Shared instruction constants (Opcodes) utilized by both the encoder and decoder.
pub mod opcodes;

// Re-export the primary functions and types to simplify the public API.
pub use decoder::{decode_ast, DecodeError};
pub use encoder::encode_ast;
