//! Build Script for Omni-Core
//!
//! This script runs automatically before the main cargo compilation step.
//! It is strictly required when building the Node.js native addon (N-API)
//! to properly configure the linker flags and environment variables
//! needed by the `napi-rs` framework.

extern crate napi_build;

/// Triggers the N-API build environment setup.
fn main() {
    napi_build::setup();
}
