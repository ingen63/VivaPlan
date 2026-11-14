#!/usr/bin/env bash
# Legt Labels, Meilensteine und Issues aus manifest.json und bodies/ in GitHub an.
# Voraussetzung: GitHub CLI `gh`, angemeldet mit `gh auth login`.
# Ohne APPLY=1 wird nur ausgegeben, was geschehen würde (Trockenlauf).
#   APPLY=1 ./tools/github-issues/create.sh
set -euo pipefail
cd "$(dirname "$0")"
command -v gh >/dev/null || { echo "gh fehlt: brew install gh" >&2; exit 1; }
command -v jq >/dev/null || { echo "jq fehlt: brew install jq" >&2; exit 1; }
run() { if [ "${APPLY:-0}" = 1 ]; then "$@"; else echo "[Trockenlauf] $*"; fi; }

REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner)
echo "Repository: $REPO"

# Labels (idempotent)
label() { run gh label create "$1" --color "$2" --description "$3" --force; }
label bug 'd73a4a' 'Fehler oder vermuteter Fehler'
label feature '0e8a16' 'Neue Funktion oder Änderung'
label allgemein '5319e7' 'Projektweite Aufgabe'
label engine '1d76db' 'Rechenkern'
label ui 'fbca04' 'Oberfläche'
label repo 'c5def5' 'Repository und Daten'
label false-positive 'cfd3d7' 'Kein Fehler, nach Prüfung geschlossen'

# Meilensteine (Fehler bei bereits vorhandenen werden ignoriert)
for ms in "Nächste Version" "Später"; do
  run gh api "repos/$REPO/milestones" -f title="$ms" >/dev/null 2>&1 || true
done

# Issues
jq -c '.[]' manifest.json | while read -r it; do
  id=$(jq -r .id <<<"$it"); title=$(jq -r .title <<<"$it"); body=$(jq -r .body <<<"$it")
  ms=$(jq -r .milestone <<<"$it"); state=$(jq -r .state <<<"$it")
  labels=$(jq -r '.labels | join(",")' <<<"$it")
  if gh issue list --state all --search "\"$id \" in:title" --json number -q '.[0].number' 2>/dev/null | grep -q .; then
    echo "$id: gibt es schon, übersprungen"; continue
  fi
  if [ "${APPLY:-0}" = 1 ]; then
    url=$(gh issue create --title "$title" --body-file "$body" --label "$labels" --milestone "$ms")
    echo "$id → $url"
    case "$state" in
      closed-completed) gh issue close "$url" --reason completed ;;
      closed-notplanned) gh issue close "$url" --reason "not planned" ;;
    esac
  else
    echo "[Trockenlauf] $title  [$labels]  Meilenstein: $ms  Status: $state"
  fi
done
