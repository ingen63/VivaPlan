# Arbeitsanleitung für Claude Code

Lokale Desktop-App (Tauri/Rust) für die Schweizer Pensionsplanung. Überblick in `docs/BESCHREIBUNG.md` und `README.md`, Technik und Rechenregeln in `docs/ENTWICKLUNG.md`, Tests in `docs/TESTS.md` und `docs/QA.md`, Spezifikationen in `docs/` (Pflichteingaben, Weltindex JST, MCP-Konzept).

## Sprache

Deutsch mit «ss» statt «ß» (Schweizer Rechtschreibung). Im Gespräch mit mir: Anrede «du». Texte der Oberfläche, Hilfe, Commits und Issues sind deutsch; Oberfläche und Hilfe sprechen die Nutzerinnen und Nutzer weiterhin mit «Sie» an.

## Anforderungen kommen aus GitHub Issues

Jede Änderung hat ein Issue. Der Titel beginnt mit der ID (`FEA-100 …`, `ISS-12 …`), der Text enthält die Anforderung und die **Abnahmekriterien als Häkchen-Liste**. Die Abnahmekriterien sind die Prüfliste: Jedes Kriterium bekommt, wo möglich, einen Test.

Gibt es zur Aufgabe kein Issue, zuerst fragen oder eines vorschlagen. Nichts umsetzen, was keine Anforderung hat.

## Ablauf pro Issue

1. **Issue lesen** (`gh issue view <Nr>`). Ist etwas unklar oder widersprüchlich, nachfragen. Nicht still annehmen.
2. **Direkt auf `main` arbeiten.** Das Projekt hat eine einzige Person, deshalb gibt es keine Feature-Branches und keine Pull Requests. Ein Branch ist die Ausnahme für grosse oder riskante Arbeiten und braucht deine Zustimmung.
3. **Umsetzen**, klein und gezielt. Keine Änderungen ausserhalb des Issues. Gefundene Nebenbaustellen als neues Issue vorschlagen, nicht mitmachen.
4. **Testen** (siehe unten). Jedes Abnahmekriterium prüfen und im Bericht nennen, was geprüft ist und was nicht.
5. **Commit** mit ID und Issue-Nummer, Imperativ oder Substantiv, ohne Punkt:
   `FEA-100 Warnsymbol bei Notverkauf (#27)`.
   Eine Anforderung pro Commit (oder wenige Commits). Kein `Fixes #27` in der Commit-Nachricht, denn das schliesst das Issue beim Push von selbst.
   Nach dem Commit, der die Anforderung vollständig umsetzt, bekommt das Issue das Label `umgesetzt` und einen Kommentar
   «Umgesetzt in <Commit>. Abnahme ausstehend.» (`gh issue edit <Nr> --add-label umgesetzt`, `gh issue comment`).
   Teilschritte bekommen das Label nicht. Wird das Issue wieder geöffnet, weil etwas fehlt, kommt das Label weg, bis die
   Nacharbeit committet ist.
6. **Abnahme** macht der Mensch. Claude hakt die Kriterien im Issue **nicht** selbst ab und schliesst es nicht.
   **Ausnahme: Einzelprüfung auf Anweisung** («Prüfe FEA-xx», Grundlage `docs/ABNAHME.md`). Dann prüft Claude die Kriterien
   (vorhandene Tests laufen lassen, sonst von Hand) und hält das Ergebnis fest: Kommentar im Issue mit dem Ergebnis je
   Kriterium, erfüllte Kriterien abhaken, das Issue schliessen, wenn alle erfüllt sind (sonst offen lassen und den Fehler
   nennen), und in `docs/ABNAHME.md` die Spalten «Test» und «Geprüft» nachführen. Die Änderung an `docs/ABNAHME.md` wird
   nur auf ausdrückliches Wort committet.
7. **Push und Version** (Tag) nur auf ausdrückliches Wort.

Commit und Push nur, wenn es ausdrücklich verlangt wird. Nie `--force`, nie die Git-Historie umschreiben. Die Hooks (`rust-test-agent`) und die GitHub-Workflows sind die Kontrolle anstelle eines Pull Requests.

## Tests

- Alle Rust-Tests: `make test` (nextest). Weitere Werkzeuge: `make coverage`, `make mutants`, `make proptest`, `make fuzz`, `make quality`.
- Oberflächentests: `e2e/` (Playwright), Rauchtest `tools/ui-smoke.sh`.
- Engine (`vivaplan-engine`), App (`vivaplan-app`) und Kommandozeile (`vivaplan-cli`) haben jeweils mindestens 80 % Zeilenabdeckung (Schwelle in `tools/coverage-min`). Die 80 % sind eine **Untergrenze**; zusätzlich zeigen `make coverage-diff` die Abdeckung des geänderten Codes und `make mutants-diff` die nicht erkannten Mutanten (jeder wird getestet oder begründet ausgeschlossen, kein Score-Ziel). Nach einem Commit und vor einem Push läuft der Agent `rust-test-agent` über die Hooks.
- Tests, die ein Abnahmekriterium belegen, nennen Issue und Kriterium (`FEA-100/AK2` im Kommentar oder Namen, `AK2` = zweites Häkchen). `make report` zeigt die Übersichtsseite Testqualität (Beschreibung in `docs/TESTS.md`).
- Änderungen am Rechenkern brauchen einen Regressionstest. Die Golden-Dateien (`crates/engine/tests/golden/`) dürfen nur mit Begründung im Issue ändern.

## Regeln

- Rust-Code wird mit `cargo fmt` formatiert (Einstellungen in `rustfmt.toml`). Vor jedem Commit `cargo fmt --all --check`
  (und in `fuzz/` dasselbe, eigener Workspace); der Workflow «Test quality» prüft es bei jedem Push.
- Keine Netzwerkzugriffe in der App. Alle Daten bleiben lokal.
- **Keine privaten Daten** in Code, Tests, Issues, Commits oder Beispielplanungen: keine echten Namen, Adressen, Beträge oder Planungsdateien. Beispiele und Tests brauchen erfundene, neutrale Daten.
- Hilfetexte (`crates/app/ui/hilfe/`), `docs/BESCHREIBUNG.md` und die Spezifikation im selben Zug nachführen, wenn sich Verhalten oder Begriffe ändern.
- Rechenkern und Oberfläche getrennt halten: Berechnung in `crates/engine/`, Darstellung in `crates/app/ui/`.

## Issues anlegen und pflegen

- Vorlagen liegen in `.github/ISSUE_TEMPLATE/` (Fehler, Funktion). Labels: `bug`, `feature`, `allgemein`, `engine`, `ui`, `repo`, `false-positive`, `umgesetzt` (siehe Ablauf, Schritt 5); bei Fehlern zusätzlich der Fundort `gefunden:test`, `gefunden:abnahme` oder `gefunden:nutzung`. Meilensteine: «Nächste Version», «Später».
- Eine Anforderung pro Issue. Die Abnahmekriterien nach dem Start nicht stillschweigend ändern: Änderungen als Kommentar mit Begründung.
- Fehlalarme mit `false-positive` als «not planned» schliessen, mit Begründung.
- Das Skript `tools/github-issues/create.sh` hat die ersten Issues aus den Spezifikationen angelegt. Es ist wiederholbar und überspringt vorhandene Issues.
