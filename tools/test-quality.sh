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
# #84: exit codes of cargo-mutants (src/exit_code.rs): 0 all caught, 2 missed mutants, 3 timeouts
# (count as caught, docs/TESTS.md). These are results, not an abort: the run has completed. All other
# codes (1 usage, 4 tests already fail without mutation, 5/6 diff, 70 internal error) abort without writing the state.
rc=0
cargo mutants -p vivaplan-engine -j 8 --no-shuffle --output local/test-quality/reports || rc=$?
case "$rc" in
  0|2|3) ;;
  *) echo "cargo mutants fehlgeschlagen (Fehlercode $rc) – Stand nicht geschrieben." >&2; exit "$rc" ;;
esac
echo "$head" > "$state"   # write only after a complete run
echo "Reports: local/test-quality/reports/"
