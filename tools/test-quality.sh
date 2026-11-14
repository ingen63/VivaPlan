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
# FEA-66: Fehlercodes von cargo-mutants (src/exit_code.rs): 0 alle erkannt, 2 nicht erkannte Mutanten, 3 Timeouts
# (zählen als erkannt, docs/TESTS.md). Das sind Ergebnisse, kein Abbruch: Der Lauf ist durchgelaufen. Alle anderen
# Codes (1 Aufruf, 4 Tests schon ohne Mutation rot, 5/6 Diff, 70 interner Fehler) brechen ab, ohne den Stand zu schreiben.
rc=0
cargo mutants -p vivaplan-engine -j 8 --no-shuffle --output local/test-quality/reports || rc=$?
case "$rc" in
  0|2|3) ;;
  *) echo "cargo mutants fehlgeschlagen (Fehlercode $rc) – Stand nicht geschrieben." >&2; exit "$rc" ;;
esac
echo "$head" > "$state"   # nur nach erfolgreichem Durchlauf schreiben
echo "Reports: local/test-quality/reports/"
