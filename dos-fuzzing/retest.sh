#!/bin/bash
set -e

echo "==================================================="
echo "🔬  Omni-Core Artifact Verification (Phase 2)   🔬"
echo "==================================================="
echo ""

echo "[1/2] 🔨 Building retest binary (Release mode)..."
cargo build --release --bin retest

echo ""
echo "======================================"
echo "🔬 VERIFICATION (RETEST)"
echo "======================================"
cargo run --release --bin retest

echo ""
echo "✅ Audit Complete."