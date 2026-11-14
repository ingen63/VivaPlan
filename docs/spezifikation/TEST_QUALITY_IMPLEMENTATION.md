# Auftrag: Test-Qualitäts-Tooling für dieses Rust-Projekt einrichten

> Übergabe an Claude Code. Lies dieses Dokument vollständig, analysiere zuerst das Projekt (Schritt 0) und setze dann die Schritte 1–8 um. Frage nach, wenn eine Entscheidung nicht eindeutig aus dem Code ableitbar ist (z. B. Default-Branch, CI-Plattform).

## Ziel

Die Testqualität soll messbar und regelmässig überprüft werden – nicht nur „Tests grün“, sondern:

| Ebene | Frage | Werkzeug | Report |
| --- | --- | --- | --- |
| Ausführung | Laufen alle Tests, wie lange, welche sind flaky? | cargo-nextest | JUnit XML |
| Abdeckung | Welche Zeilen/Regionen werden ausgeführt? | cargo-llvm-cov | HTML, LCOV, Cobertura |
| Wirksamkeit | Würden die Tests einen echten Fehler bemerken? | cargo-mutants | `local/mutants.out/` (`outcomes.json`, `missed.txt`) |
| Eingabebreite | Unerwartete Eingaben, Nebenläufigkeit, UB? | proptest, cargo-fuzz, Miri, loom | Fehlerfälle, Korpora, UB-Meldungen |

**Regelmässige Läufe** (nächtlich bzw. wöchentlich) sollen **nur dann laufen, wenn es seit dem letzten erfolgreichen Lauf neue Commits gibt**. Gab es keinen neuen Commit, wird der Lauf übersprungen und nichts geändert.

## Schritt 0 – Projekt analysieren (vor jeder Änderung)

Ermittle und notiere kurz im PR-/Commit-Text:

1. Workspace-Struktur (`Cargo.toml`, Mitglieder-Crates), Default-Branch (`git symbolic-ref refs/remotes/origin/HEAD`).
2. CI-Plattform: Gibt es `.github/workflows/`? Ist das Remote GitHub? → Wenn nicht GitHub, nur den lokalen Teil (Schritt 7) umsetzen und nachfragen.
3. Welche Test-Ebenen aus der Matrix (unten) sind **relevant**:
   - `unsafe` im Code? (`rg -n "unsafe " --type rust`) → Miri
   - Atomics / eigene Synchronisation / lock-free? (`rg -n "Atomic|UnsafeCell|crossbeam" --type rust`) → loom
   - Parser, Decoder, Einlesen externer Daten (Dateien, Netzwerk, CLI-Eingaben)? → cargo-fuzz + proptest
   - Kern-Datenstrukturen / Berechnungen mit Invarianten (Round-Trip, Sortierung, Summen) → proptest
4. Aktuelle Baseline: `cargo test --workspace` muss grün sein, bevor du Gates einführst. Wenn nicht: stoppen und melden.

## Test-Matrix (wann welche Ebene läuft)

| Ebene | Werkzeug | Pull Request | Nächtlich (nur bei neuen Commits) | Wöchentlich (nur bei neuen Commits) |
| --- | --- | --- | --- | --- |
| Unit + Integration | cargo-nextest | immer | immer | – |
| Coverage | cargo-llvm-cov | immer (Gate) | immer (Report) | – |
| Mutation (nur geänderte Zeilen) | cargo-mutants `--in-diff` | immer (Gate) | – | – |
| Mutation (voll, 4 Shards) | cargo-mutants `--shard` | – | – | immer |
| Property-based | proptest | normale Fallzahl | `PROPTEST_CASES=10000` | – |
| Speichermodell / UB | Miri (nightly) | wenn `unsafe` geändert | wenn `unsafe` vorhanden | – |
| Nebenläufigkeit | loom | wenn relevant | wenn relevant | – |
| Fuzzing | cargo-fuzz (nightly) | – | 5 Min. pro Target | – |

Ebenen, die laut Schritt 0 **nicht relevant** sind, werden nicht eingerichtet (keine leeren Jobs). Im Abschlussbericht begründen.

## Schritt 1 – nextest konfigurieren

Datei `.config/nextest.toml`:

```toml
[profile.default]
retries = 0

[profile.ci]
retries = 2
flaky-result = "fail"      # Test, der erst im Retry besteht, lässt den Lauf scheitern
fail-fast = false
slow-timeout = { period = "60s", terminate-after = 3 }

[profile.ci.junit]
path = "junit.xml"         # landet unter <target>/nextest/ci/junit.xml
store-success-output = false
store-failure-output = true
```

Hinweis: `flaky-result` braucht nextest ≥ 0.9.131. Falls ältere Version installiert ist, aktualisieren.

## Schritt 2 – Coverage-Ausnahmen und Lints

In `Cargo.toml` (Workspace-Root bzw. jede Crate):

```toml
[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = ['cfg(coverage,coverage_nightly)', 'cfg(loom)'] }
```

Nicht testbare Funktionen (z. B. `main`-Verdrahtung) nur gezielt ausnehmen:

```rust
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#[cfg_attr(coverage_nightly, coverage(off))]
fn main() { /* ... */ }
```

## Schritt 3 – cargo-mutants konfigurieren

Datei `.cargo/mutants.toml`:

```toml
# Reine Ausgabe/Verdrahtung nicht mutieren – anpassen nach Schritt 0
exclude_globs = ["src/main.rs"]
test_tool = "nextest"
```

Erstlauf lokal auf der Kern-Crate: `cargo mutants -p <core-crate> -j 4 --no-shuffle`. `local/mutants.out/missed.txt` sortiert sichten:
- Logik-Lücken → Test ergänzen (fehlende Assertion, Grenzwert `<` vs `<=`).
- Bewusst ungetestet (z. B. `Debug`/`Display`-Impls) → `#[mutants::skip]` (Crate `mutants` als Dependency).

`local/mutants.out*/` in `.gitignore` aufnehmen.

## Schritt 4 – Zusatz-Ebenen (nur wenn relevant laut Schritt 0)

- **proptest:** `proptest` als dev-dependency; Properties in `tests/proptest_*.rs` für Round-Trips/Invarianten der Kernlogik. Fallzahl über `PROPTEST_CASES` steuerbar lassen.
- **Miri:** Keine Codeänderung nötig. Langsame Tests unter Miri mit `#[cfg_attr(miri, ignore)]` markieren.
- **loom:** Tests in `tests/loom_*.rs` hinter `#[cfg(loom)]`; Ausführung mit `RUSTFLAGS="--cfg loom"`.
- **cargo-fuzz:** `cargo fuzz init`, je ein Target pro externer Eingabe (Parser/Decoder). `fuzz/` hat eigenes `Cargo.toml`, nicht im Workspace.

## Schritt 5 – Workflow für PRs und nächtliche Läufe

Datei `.github/workflows/test-quality.yml`. Kernpunkt: Job `changes` entscheidet bei geplanten Läufen, ob seit dem letzten **erfolgreichen geplanten** Lauf ein neuer Commit auf dem Default-Branch existiert. Nur dann laufen die übrigen Jobs.

```yaml
name: Test quality
on:
  pull_request:
  schedule:
    - cron: "17 2 * * *"          # nächtlich 02:17 UTC
  workflow_dispatch:              # manuell: läuft immer

permissions:
  contents: read
  actions: read                   # für gh run list im changes-Job

concurrency:
  group: test-quality-${{ github.ref }}
  cancel-in-progress: true

jobs:
  changes:
    runs-on: ubuntu-latest
    outputs:
      run: ${{ steps.check.outputs.run }}
    steps:
      - id: check
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          if [ "${{ github.event_name }}" != "schedule" ]; then
            echo "run=true" >> "$GITHUB_OUTPUT"; exit 0
          fi
          last=$(gh run list -R "$GITHUB_REPOSITORY" --workflow test-quality.yml \
                   --event schedule --status success --limit 1 \
                   --json headSha -q '.[0].headSha')
          if [ "$last" = "$GITHUB_SHA" ]; then
            echo "Kein neuer Commit seit letztem Lauf ($last) – übersprungen." >> "$GITHUB_STEP_SUMMARY"
            echo "run=false" >> "$GITHUB_OUTPUT"
          else
            echo "run=true" >> "$GITHUB_OUTPUT"
          fi

  tests-coverage:
    needs: changes
    if: needs.changes.outputs.run == 'true'
    runs-on: ubuntu-latest
    env:
      PROPTEST_CASES: ${{ github.event_name == 'schedule' && '10000' || '256' }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: llvm-tools-preview }
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@v2
        with: { tool: "cargo-llvm-cov,nextest" }
      - name: Tests + Coverage (JUnit via nextest-Profil ci)
        run: |
          cargo llvm-cov nextest --workspace --all-features --profile ci \
            --lcov --output-path lcov.info --fail-under-lines <BASELINE>
          cargo llvm-cov report --html --output-dir coverage-html
          cargo llvm-cov report --cobertura --output-path cobertura.xml
      - name: JUnit einsammeln
        if: ${{ !cancelled() }}
        run: cp "$(find target -path '*nextest/ci/junit.xml' | head -n1)" junit.xml || true
      - name: Coverage-Zusammenfassung
        if: ${{ !cancelled() }}
        run: |
          { echo '### Coverage'; echo '```'; cargo llvm-cov report --summary-only; echo '```'; } >> "$GITHUB_STEP_SUMMARY"
      - uses: actions/upload-artifact@v4
        if: ${{ !cancelled() }}
        with:
          name: test-quality-reports
          path: |
            lcov.info
            cobertura.xml
            coverage-html/
            junit.xml

  mutants-diff:
    needs: changes
    if: github.event_name == 'pull_request'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@v2
        with: { tool: "cargo-mutants,nextest" }
      - name: Mutanten auf geänderten Zeilen
        run: |
          git diff origin/${{ github.base_ref }}...HEAD > pr.diff
          if ! grep -q '\.rs$' <(git diff --name-only origin/${{ github.base_ref }}...HEAD); then
            echo "Keine Rust-Änderungen – keine Mutanten." >> "$GITHUB_STEP_SUMMARY"; exit 0
          fi
          cargo mutants --in-diff pr.diff --no-shuffle -j 2
      - name: Mutations-Zusammenfassung
        if: ${{ !cancelled() }}
        run: |
          f=local/mutants.out/outcomes.json
          [ -f "$f" ] || exit 0
          jq -r '"### Mutation testing\ncaught: \(.caught)  missed: \(.missed)  timeout: \(.timeout)  unviable: \(.unviable)"' "$f" >> "$GITHUB_STEP_SUMMARY"
          if [ -s local/mutants.out/missed.txt ]; then
            echo '#### Nicht erkannte Mutanten' >> "$GITHUB_STEP_SUMMARY"
            sed 's/^/- /' local/mutants.out/missed.txt >> "$GITHUB_STEP_SUMMARY"
          fi
      - uses: actions/upload-artifact@v4
        if: ${{ !cancelled() }}
        with: { name: mutants-diff, path: local/mutants.out/ }

  # ---- Nur einrichten, wenn laut Schritt 0 relevant ----
  miri:
    needs: changes
    if: needs.changes.outputs.run == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with: { components: miri }
      - run: cargo miri test --workspace --lib

  loom:
    needs: changes
    if: needs.changes.outputs.run == 'true'
    runs-on: ubuntu-latest
    env:
      RUSTFLAGS: "--cfg loom"
      LOOM_MAX_PREEMPTIONS: "2"
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --release --test 'loom_*'

  fuzz:
    needs: changes
    if: github.event_name == 'schedule' && needs.changes.outputs.run == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - uses: taiki-e/install-action@v2
        with: { tool: cargo-fuzz }
      - name: Alle Fuzz-Targets je 5 Minuten
        run: |
          for t in $(cargo +nightly fuzz list); do
            cargo +nightly fuzz run "$t" -- -max_total_time=300
          done
      - uses: actions/upload-artifact@v4
        if: failure()
        with: { name: fuzz-artifacts, path: fuzz/artifacts/ }
```

Anpassen: `<BASELINE>` = aktuelle Line-Coverage minus 1 Prozentpunkt (aus Schritt 0 gemessen, abgerundet). Miri nur auf PRs laufen lassen, wenn `unsafe`-Dateien geändert wurden (über `dorny/paths-filter` oder `git diff`-Check), sonst nur nächtlich.

## Schritt 6 – Wöchentlicher voller Mutationslauf

Datei `.github/workflows/mutation-full.yml` – gleiche „nur bei neuen Commits“-Logik, aber gegen den letzten erfolgreichen Lauf **dieses** Workflows:

```yaml
name: Mutation (full)
on:
  schedule:
    - cron: "17 3 * * 1"          # montags 03:17 UTC
  workflow_dispatch:

permissions:
  contents: read
  actions: read

jobs:
  changes:
    runs-on: ubuntu-latest
    outputs:
      run: ${{ steps.check.outputs.run }}
    steps:
      - id: check
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          if [ "${{ github.event_name }}" != "schedule" ]; then echo "run=true" >> "$GITHUB_OUTPUT"; exit 0; fi
          last=$(gh run list -R "$GITHUB_REPOSITORY" --workflow mutation-full.yml \
                   --event schedule --status success --limit 1 --json headSha -q '.[0].headSha')
          if [ "$last" = "$GITHUB_SHA" ]; then
            echo "Kein neuer Commit – übersprungen." >> "$GITHUB_STEP_SUMMARY"
            echo "run=false" >> "$GITHUB_OUTPUT"
          else
            echo "run=true" >> "$GITHUB_OUTPUT"
          fi

  mutants:
    needs: changes
    if: needs.changes.outputs.run == 'true'
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix: { shard: [0, 1, 2, 3] }
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@v2
        with: { tool: "cargo-mutants,nextest" }
      - run: cargo mutants --shard ${{ matrix.shard }}/4 --no-shuffle -j 2
      - uses: actions/upload-artifact@v4
        if: ${{ !cancelled() }}
        with: { name: "mutants-shard-${{ matrix.shard }}", path: local/mutants.out/ }

  summary:
    needs: mutants
    if: ${{ !cancelled() && needs.mutants.result != 'skipped' }}
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with: { pattern: "mutants-shard-*" }
      - run: |
          jq -s '{caught: (map(.caught)|add), missed: (map(.missed)|add), timeout: (map(.timeout)|add), unviable: (map(.unviable)|add)}
                 | . + {score: (if (.caught+.missed)>0 then (100*.caught/(.caught+.missed)|floor) else null end)}' \
             mutants-shard-*/outcomes.json > total.json
          jq -r '"### Mutation score (voll): \(.score)%\ncaught \(.caught) · missed \(.missed) · timeout \(.timeout) · unviable \(.unviable)"' total.json >> "$GITHUB_STEP_SUMMARY"
          echo '#### Nicht erkannte Mutanten' >> "$GITHUB_STEP_SUMMARY"
          cat mutants-shard-*/missed.txt | sort | sed 's/^/- /' >> "$GITHUB_STEP_SUMMARY"
```

Hinweis: Ein übersprungener Lauf ist ebenfalls „success“ mit demselben SHA – das ist gewollt, der Vergleich bleibt korrekt.

## Schritt 7 – Lokale Ausführung (Makefile + Skript)

`Makefile` im Projekt-Root (bestehendes erweitern, nicht überschreiben):

```makefile
.PHONY: test coverage mutants mutants-diff proptest miri loom fuzz quality
test:        ; cargo nextest run --workspace
coverage:    ; cargo llvm-cov nextest --workspace --html --open
mutants:     ; cargo mutants -j 4 --no-shuffle
mutants-diff:; git diff origin/main...HEAD > /tmp/pr.diff && cargo mutants --in-diff /tmp/pr.diff
proptest:    ; PROPTEST_CASES=10000 cargo nextest run --workspace -E 'test(/proptest/)'
miri:        ; cargo +nightly miri test --workspace --lib
loom:        ; RUSTFLAGS="--cfg loom" LOOM_MAX_PREEMPTIONS=2 cargo test --release --test 'loom_*'
fuzz:        ; for t in $$(cargo +nightly fuzz list); do cargo +nightly fuzz run $$t -- -max_total_time=60; done
quality:     ; ./scripts/test-quality.sh
```

Nicht relevante Targets weglassen; `main` durch den echten Default-Branch ersetzen.

`tools/test-quality.sh` (ausführbar) – für lokale/zeitgesteuerte Läufe ohne CI, ebenfalls **nur bei neuen Commits**:

```bash
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
cargo mutants -j 4 --no-shuffle --output local/test-quality/reports || true
echo "$head" > "$state"   # nur nach erfolgreichem Durchlauf schreiben
echo "Reports: local/test-quality/reports/"
```

`local/test-quality/` in `.gitignore`. `FORCE=1 make quality` erzwingt einen Lauf.

## Schritt 8 – Dokumentation

Kurzen Abschnitt `## Test quality` in `README.md` (oder `CONTRIBUTING.md`): welche Ebenen es gibt, wie man sie lokal startet (`make …`), wo die Reports liegen (Actions-Artefakte, Job-Summary), dass geplante Läufe ohne neue Commits übersprungen werden.

## Schwellenwerte

| Metrik | Start-Gate | Ziel |
| --- | --- | --- |
| Line-Coverage | Baseline − 1 Punkt, nie sinkend | 80–90 % |
| Nicht erkannte Mutanten auf geänderten Zeilen | 0 (oder `#[mutants::skip]` mit Begründung) | 0 |
| Mutation-Score voller Lauf | nur Report | ≥ 80 % in Kernlogik |
| Flaky Tests | `flaky-result = "fail"` | 0 |

## Abnahmekriterien

- [ ] `cargo nextest run --profile ci` grün, `junit.xml` wird erzeugt.
- [ ] `make coverage` erzeugt HTML-Report; CI-Gate mit gemessener Baseline aktiv.
- [ ] Erstlauf cargo-mutants auf Kern-Crate durchgeführt, `missed.txt` triagiert (Tests ergänzt oder begründet übersprungen).
- [ ] PR-Workflow: Coverage + `--in-diff`-Mutation laufen und schreiben Job-Summary.
- [ ] Nächtlicher und wöchentlicher Lauf: zweiter manueller Test zeigt „übersprungen“, wenn kein neuer Commit vorliegt (prüfen via `gh workflow run` zweimal hintereinander bzw. Logik im `changes`-Job lokal mit Fake-SHA testen).
- [ ] Nur relevante Zusatz-Ebenen (Miri/loom/fuzz/proptest) eingerichtet; im Abschlussbericht begründet, welche weggelassen wurden.
- [ ] `tools/test-quality.sh` überspringt bei unverändertem HEAD und läuft mit `FORCE=1`.
- [ ] README-Abschnitt ergänzt.

## Abschlussbericht an mich

Am Ende kurz zusammenfassen: gemessene Baseline-Coverage, Mutation-Score des Erstlaufs, Anzahl ergänzter Tests, welche Ebenen aktiv/weggelassen sind und warum, offene Punkte.
