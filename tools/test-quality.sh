#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
state=local/test-quality/last-run-sha
head=$(git rev-parse HEAD)
if [ -f "$state" ] && [ "$(cat "$state")" = "$head" ] && [ "${FORCE:-0}" != "1" ]; then
  echo "Kein neuer Commit seit letztem Lauf ($head) – übersprungen."; exit 0
fi
mkdir -p local/test-quality/reports
cargo llvm-cov nextest --workspace --profile ci --html --output-dir local/test-quality/reports/coverage
cargo llvm-cov report --summary-only | tee local/test-quality/reports/coverage-summary.txt
cargo mutants -p vivaplan-engine -j 8 --no-shuffle --output local/test-quality/reports || true
echo "$head" > "$state"   # nur nach erfolgreichem Durchlauf schreiben
echo "Reports: local/test-quality/reports/"
