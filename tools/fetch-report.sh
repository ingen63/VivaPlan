#!/usr/bin/env bash
# Fetches the overview page «Testqualität» that test-quality.yml builds on GitHub for a pushed commit
# (artifact test-quality-reports, folder report-html/) into local/test-quality/report/.
# Started in the background by the pre-push hook (tools/git-hooks/pre-push); can also be run by hand.
# Usage: tools/fetch-report.sh [<commit>]   (default HEAD)
# Log of the background run: local/test-quality/fetch-report.log

set -uo pipefail
root="$(git rev-parse --show-toplevel)"
cd "$root" || exit 1
sha="$(git rev-parse "${1:-HEAD}")" || exit 1
out="local/test-quality/report"
wait_start="${FETCH_REPORT_WAIT:-300}"   # seconds until the run must have appeared on GitHub

say() { echo "$(date '+%d.%m.%Y %H:%M:%S') fetch-report ${sha:0:7}: $*"; }
notify() {   # macOS notification, silently skipped elsewhere
  command -v osascript >/dev/null && osascript -e "display notification \"$1\" with title \"VivaPlan Testqualität\"" >/dev/null 2>&1
  return 0
}

command -v gh >/dev/null || { say "gh fehlt, abgebrochen."; exit 1; }

# 1. Find the push run of test-quality.yml for this commit (it appears a few seconds after the push)
run=""
for _ in $(seq 0 10 "$wait_start"); do
  run="$(gh run list --workflow test-quality.yml --commit "$sha" --event push --limit 1 \
         --json databaseId --jq '.[0].databaseId // empty' 2>/dev/null)"
  [ -n "$run" ] && break
  sleep 10
done
[ -z "$run" ] && { say "kein Lauf von test-quality.yml nach ${wait_start} s gefunden."; exit 1; }
say "warte auf Lauf $run ..."

# 2. Wait for the end (result does not matter: the page also shows failed tests)
gh run watch "$run" --interval 30 >/dev/null 2>&1
conclusion="$(gh run view "$run" --json conclusion --jq .conclusion 2>/dev/null)"
say "Lauf $run beendet: ${conclusion:-unbekannt}"

# 3. Download the artifact and replace the local page
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
if ! gh run download "$run" -n test-quality-reports -D "$tmp" >/dev/null 2>&1 || [ ! -f "$tmp/report-html/index.html" ]; then
  say "kein Artefakt test-quality-reports (Job tests-coverage übersprungen oder abgebrochen)."
  notify "Kein Bericht für ${sha:0:7} (Lauf: ${conclusion:-unbekannt})"
  exit 1
fi
mkdir -p "$out"
rm -rf "${out:?}"/*
cp -R "$tmp/report-html/." "$out/"
echo "$sha" > "$out/commit"
say "Bericht in $out/index.html (Lauf: $conclusion)"
notify "Bericht für ${sha:0:7} geladen (Lauf: ${conclusion:-unbekannt})"
exit 0
