#!/usr/bin/env bash
# Claude-Code-Hook (PostToolUse auf Bash):
# Erkennt einen erfolgreichen `git commit` und weist Claude an,
# den rust-test-agent zu starten.
set -euo pipefail

input="$(cat)"
cmd="$(printf '%s' "$input" | jq -r '.tool_input.command // ""')"

# Nur auf git commit reagieren
if ! printf '%s' "$cmd" | grep -Eq '(^|[;&|(][[:space:]]*|[[:space:]])git([[:space:]]+(-C|-c|--git-dir|--work-tree)[[:space:]]+[^[:space:]]+|[[:space:]]+-[^[:space:]]+)*[[:space:]]+commit([[:space:]]|$)'; then
  exit 0
fi

cd "${CLAUDE_PROJECT_DIR:-.}"

# Nur bei Rust-Projekten
[ -f Cargo.toml ] || exit 0

# Keine Endlosschleife: Commits des Agenten selbst ignorieren
last_msg="$(git log -1 --pretty=%s 2>/dev/null || true)"
case "$last_msg" in
  "test(auto):"*) exit 0 ;;
esac

# Nur wenn der Commit Rust-Dateien enthält
if ! git diff-tree --root --no-commit-id --name-only -r HEAD 2>/dev/null | grep -Eq '\.rs$|Cargo\.toml$'; then
  exit 0
fi

# Reine Testcommits (nur Dateien unter tests/) brauchen keine Prüfung durch den Agenten
if ! git diff-tree --root --no-commit-id --name-only -r HEAD 2>/dev/null | grep -E '\.rs$|Cargo\.toml$' | grep -vq '/tests/'; then
  exit 0
fi

sha="$(git rev-parse --short HEAD)"

jq -n --arg sha "$sha" '{
  decision: "block",
  reason: ("Commit " + $sha + " enthält Rust-Änderungen. Starte jetzt den Subagenten rust-test-agent im SCHNELLMODUS (siehe Agentenbeschreibung, Ziel unter 1 Minute): nur die Tests der geänderten Pakete mit cargo nextest ausführen, Testfälle für die Änderungen in " + $sha + " ergänzen, KEINE Abdeckungsmessung (die läuft vollständig erst vor dem Push). Neuen Testcode mit cargo fmt formatieren (cargo fmt --all --check muss sauber sein), nur gezielt Testdateien ändern und stagen. Fasse danach sein Ergebnis kurz zusammen.")
}'
