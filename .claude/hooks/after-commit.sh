#!/usr/bin/env bash
# Claude-Code-Hook (PostToolUse auf Bash) nach einem `git commit` mit Rust-Änderungen.
#
# #160: Der Hook führt die Tests selbst und leise aus (wenig Kontext für Claude). Er verlangt den Subagenten
# rust-test-agent nur noch,
#   - wenn Tests fehlschlagen, oder
#   - wenn der Commit Code des Rechenkerns (crates/engine/src) ändert, ohne einen Test zu ändern oder hinzuzufügen.
# Sonst meldet er Claude nur die Zusammenfassung des Testlaufs. Die Prüfung vor dem Push (tools/git-hooks/pre-push,
# Abdeckung und Mutanten) bleibt unverändert.
#
# Getestete Pakete: die geänderten und die davon abhängigen (Rechenkern → auch App und Kommandozeile; Cargo.toml oder
# Cargo.lock im Wurzelverzeichnis → alle). Volle Ausgabe: target/after-commit.log.
# Zum Testen lassen sich `cargo` (über PATH) und der Ordner der Protokolle (AFTER_COMMIT_STATE) ersetzen,
# siehe tools/test_after_commit.py.
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

sha="$(git rev-parse --short HEAD 2>/dev/null || true)"
[ -n "$sha" ] || exit 0

# Denselben Commit nur einmal prüfen (z. B. wenn ein `git commit` fehlschlug und HEAD unverändert blieb)
state="${AFTER_COMMIT_STATE:-target}"
mkdir -p "$state"
[ "$(cat "$state/after-commit.last" 2>/dev/null || true)" = "$sha" ] && exit 0

files="$(git diff-tree --root --no-commit-id --name-only -r HEAD 2>/dev/null || true)"
rust="$(printf '%s\n' "$files" | grep -E '\.rs$|Cargo\.(toml|lock)$' | grep -v '^fuzz/' || true)"

# Nur wenn der Commit Rust-Dateien enthält (fuzz/ ist ein eigener Workspace)
[ -n "$rust" ] || exit 0

# Pakete: geänderte und abhängige
if printf '%s\n' "$rust" | grep -Eq '^Cargo\.(toml|lock)$|^crates/engine/'; then
  pkgs="--workspace"
  names="alle Pakete"
else
  pkgs=""
  names=""
  for p in app cli; do
    if printf '%s\n' "$rust" | grep -q "^crates/$p/"; then
      pkgs="$pkgs -p vivaplan-$p"
      names="$names vivaplan-$p"
    fi
  done
  names="${names# }"
  [ -n "$pkgs" ] || exit 0
fi

echo "$sha" > "$state/after-commit.last"
log="$state/after-commit.log"
# shellcheck disable=SC2086
if cargo nextest run $pkgs --no-fail-fast --status-level fail --final-status-level fail --hide-progress-bar \
     --cargo-quiet >"$log" 2>&1; then
  ok=1
else
  ok=0
fi
summary="$(grep -E '^[[:space:]]*Summary' "$log" | tail -1 | sed 's/^[[:space:]]*//' || true)"
[ -n "$summary" ] || summary="keine Zusammenfassung (Übersetzungsfehler?), siehe $log"

if [ "$ok" = 0 ]; then
  failed="$(grep -E '^[[:space:]]+FAIL ' "$log" | sed 's/^[[:space:]]*//' | sort -u | head -20 || true)"
  [ -n "$failed" ] || failed="$(grep -E '^error' "$log" | head -10 || true)"
  jq -n --arg sha "$sha" --arg names "$names" --arg summary "$summary" --arg failed "$failed" --arg log "$log" '{
    decision: "block",
    reason: ("Commit " + $sha + ": Tests (" + $names + ") fehlgeschlagen. " + $summary + "\n" + $failed + "\nVolle Ausgabe: " + $log + ". Starte den Subagenten rust-test-agent im SCHNELLMODUS (siehe Agentenbeschreibung): Ursache der Fehlschläge klären; Fehler im Produktivcode nur melden, veraltete Tests begründet anpassen; KEINE Abdeckungsmessung. Fasse danach sein Ergebnis kurz zusammen.")
  }'
  exit 0
fi

# Rechenkern geändert, aber kein Test im Commit?
engine_src="$(printf '%s\n' "$rust" | grep -E '^crates/engine/src/.*\.rs$' || true)"
if [ -n "$engine_src" ]; then
  test_files="$(printf '%s\n' "$rust" | grep -E '^crates/engine/tests/' || true)"
  # shellcheck disable=SC2086
  test_added="$(git show --format= -U0 HEAD -- $engine_src | grep -E '^\+.*(#\[test\]|proptest!)' || true)"
  if [ -z "$test_files" ] && [ -z "$test_added" ]; then
    jq -n --arg sha "$sha" --arg summary "$summary" '{
      decision: "block",
      reason: ("Commit " + $sha + ": Tests grün (" + $summary + "), aber der Commit ändert Code des Rechenkerns (crates/engine/src) ohne neuen oder geänderten Test. Starte den Subagenten rust-test-agent im SCHNELLMODUS (siehe Agentenbeschreibung): fehlende Testfälle für die Änderungen in " + $sha + " ergänzen (ist die Änderung schon durch bestehende Tests geprüft oder ändert sie kein Verhalten, ohne Arbeit kurz berichten), KEINE Abdeckungsmessung, Testcode mit cargo fmt formatieren, nur Testdateien stagen. Fasse danach sein Ergebnis kurz zusammen.")
    }'
    exit 0
  fi
fi

jq -n --arg sha "$sha" --arg names "$names" --arg summary "$summary" '{
  hookSpecificOutput: {
    hookEventName: "PostToolUse",
    additionalContext: ("Commit " + $sha + ": Tests (" + $names + ") grün. " + $summary + ". Kein Subagent nötig.")
  }
}'
