#!/bin/bash

# Le piège ABSOLU : laisse le terminal ouvert à la fin
trap 'echo ""; echo "🛑 Fin de la vérification. Appuyez sur Entrée pour fermer la fenêtre..."; read' EXIT

# Arrêt immédiat si la compilation échoue
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