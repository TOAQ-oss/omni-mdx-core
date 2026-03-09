#!/usr/bin/env bash
# build.sh — Run all tests, then compile to WASM and publish to the NPM package.
#
# Usage:
#   bash build.sh           # tests + release WASM build
#   bash build.sh --skip-tests  # WASM build only (faster iteration)
#
# Output: ../packages/mdx-next/omni-core/
# (wasm-pack generates: mdx_parser.js, mdx_parser_bg.wasm, mdx_parser.d.ts, package.json)

set -euo pipefail

SKIP_TESTS=false
for arg in "$@"; do
  [[ "$arg" == "--skip-tests" ]] && SKIP_TESTS=true
done

OUT_DIR="../packages/mdx-next/omni-core"

echo ""
echo "╔══════════════════════════════════════════╗"
echo "║        omni-mdx-core  build pipeline     ║"
echo "╚══════════════════════════════════════════╝"
echo ""

# ── Step 1: tests ─────────────────────────────────────────────────────────────
if [[ "$SKIP_TESTS" == false ]]; then
  echo "▶ Step 1/2 — Running test suite…"
  echo ""

  PASS=0; FAIL=0

  run_test() {
    local name=$1
    echo "  ┌─ $name"
    if cargo run --bin "$name" 2>&1 | sed 's/^/  │ /'; then
      echo "  └─ ✅ passed"
      PASS=$((PASS + 1))
    else
      echo "  └─ ❌ FAILED"
      FAIL=$((FAIL + 1))
    fi
    echo ""
  }

  run_test test_ast
  run_test test_errors
  run_test test_perf

  echo "  Results: $PASS passed, $FAIL failed"
  echo ""

  if [[ $FAIL -gt 0 ]]; then
    echo "❌ Tests failed — aborting WASM build."
    exit 1
  fi

  echo "✅ All tests passed."
  echo ""
else
  echo "⚠️  Skipping tests (--skip-tests)"
  echo ""
fi

# ── Step 2: WASM build ────────────────────────────────────────────────────────
echo "▶ Step 2/2 — Building WASM → $OUT_DIR"
echo ""

wasm-pack build \
  --target bundler \
  --release \
  --features wasm \
  --out-dir "$OUT_DIR"

echo ""
echo "✅ WASM build complete."
echo ""

# Print what was generated
echo "  Generated files:"
for f in "$OUT_DIR"/*; do
  size=$(wc -c < "$f" 2>/dev/null || echo "?")
  printf "    %-40s %s bytes\n" "$(basename "$f")" "$size"
done

echo ""
echo "╔══════════════════════════════════════════╗"
echo "║              Build successful ✅          ║"
echo "╚══════════════════════════════════════════╝"
echo ""