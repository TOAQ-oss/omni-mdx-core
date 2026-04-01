#!/bin/bash

echo "==================================================="
echo "🛡️  Omni-Core Continuous DoS Fuzzer (Phase 1)   🛡️"
echo "==================================================="
echo ""

mkdir -p artifacts

echo "[1/2] 🔨 Building fuzzer binary (Release mode)..."
cargo build --release --bin dos-fuzzer

echo ""
echo "======================================"
echo "🚀 DISCOVERY (CONTINUOUS FUZZING)"
echo "⚠️ Keep this terminal window open in the background."
echo "🛑 Press Ctrl+C to stop the fuzzer."
echo "======================================"
cargo run --release --bin dos-fuzzer