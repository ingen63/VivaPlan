# Arbeitsanleitung für Claude Code

Lokale Desktop-App (Tauri 2/Rust) für die Schweizer Pensionsplanung. Workspace: `crates/engine` (Rechenkern), `crates/cli` (`vivaplan`), `crates/app` (Desktop-App, Oberfläche in `crates/app/ui/`: HTML/CSS/JS ohne Bibliotheken und ohne Build-Schritt); `fuzz/` ist ein eigener Workspace. Technik, Ordner und Rechenregeln: `docs/ENTWICKLUNG.md`; Überblick `docs/BESCHREIBUNG.md`; Tests `docs/TESTS.md`, `docs/QA.md`.

## Sprache

Deutsch mit «ss» statt «ß». Im Gespräch mit mir: «du». Commits, Issues und Entwicklerdoku (`docs/`, diese Datei) deutsch.

- Code intern englisch: Bezeichner, Kommentare, Doku-Kommentare, Testnamen; Fachbegriffe nach `docs/GLOSSAR.md` (#165). Neuer und geänderter Code sofort englisch; die Umstellung des bestehenden Codes ist abgeschlossen (#166–#169). Prüfung: `make english`.
- Mehrsprachigkeit (de, en, fr, it) ist geplant (#62). Bis dahin sind Oberfläche und Hilfe deutsch; sie sprechen die Nutzer mit «Sie» an. Zahlen und Daten in jeder Sprache im Schweizer Format.

## Ablauf pro Issue

Jede Änderung hat ein GitHub-Issue (nur als `#27` nennen). Gibt es keines, zuerst fragen oder eines vorschlagen. Die Abnahmekriterien (Häkchen-Liste) sind die Prüfliste; jedes bekommt, wo möglich, einen Test. Issues anlegen/pflegen: Skill `issue-anlegen`.

1. Issue lesen (`gh issue view <Nr>`). Unklares nachfragen, nicht still annehmen.
2. Direkt auf `main` arbeiten (eine Person, keine Branches/PRs; ein Branch nur mit meiner Zustimmung).
3. Klein und gezielt umsetzen, nichts ausserhalb des Issues. Nebenbaustellen als neues Issue vorschlagen.
4. Testen; im Bericht je Kriterium nennen, was geprüft ist und was nicht.
5. Commit mit Issue-Nummer in Klammern, Imperativ oder Substantiv, ohne Punkt: `Warnsymbol bei Notverkauf (#27)`. Kein `Fixes`/`Closes #27`. Nach dem Commit, der das Issue vollständig umsetzt: Skill `issue-umgesetzt`.
6. Abnahme macht der Mensch: Kriterien nicht abhaken, Issue nicht schliessen. Ausnahme «Prüfe #27»: Skill `pruefe-issue`.
7. Commit, Push und Version (Tag) nur auf ausdrückliches Wort. Nie `--force`, nie die Historie umschreiben.

## Tests

- Beim Arbeiten gezielt (`cargo nextest run -p <Paket> -E 'test(…)'`), vor dem Commit `make test-quiet`. Oberfläche: `e2e/` (Playwright, gezielt mit `npx playwright test <Datei> -g "…"`), Rauchtest `tools/ui-smoke.sh`.
- Untergrenze 80 % Zeilenabdeckung je Paket; `make coverage-diff`, `make mutants-diff` für den geänderten Code. Der Commit-Hook testet selbst und verlangt `rust-test-agent` nur bei Bedarf.
- Tests zu einem Kriterium nennen es: `#27/AK2` im Kommentar, `issue_27_ak2_…` im Rust-Testnamen, in `e2e/` `AK2` im Testnamen innerhalb von `test.describe("… (#27)")`.
- Änderungen am Rechenkern brauchen einen Regressionstest. Golden-Dateien (`crates/engine/tests/golden/`) nur mit Begründung im Issue ändern.

## Regeln

- `cargo fmt --all --check` vor jedem Commit (auch in `fuzz/`).
- Keine Netzwerkzugriffe in der App; keine privaten Daten in Code, Tests, Issues, Commits oder Beispielen (nur erfundene Daten).
- Hilfe (`crates/app/ui/hilfe/`), `docs/BESCHREIBUNG.md` und Spezifikation im selben Zug nachführen, wenn sich Verhalten oder Begriffe ändern.
- Berechnung in `crates/engine/`, Darstellung in `crates/app/ui/`.
- Keine neue Abhängigkeit (Crate, npm-Paket) ohne Rückfrage.
- Wenig Kontext: Dateien über rund 500 Zeilen (`sim.rs`, `app.js`, `optimize.rs`, `validate.rs`, `docs/ABNAHME.md`) nur gezielt lesen (Suche, Zeilenbereich). Golden-Dateien nie ganz lesen. Breite Suchen über viele Dateien an den Agenten `Explore` geben.
- Stopp-Regel: Übersetzungs- oder Testfehler nach drei Versuchen nicht behoben → Ursache nennen und nachfragen.
