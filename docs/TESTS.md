# Teststrategie

Grundsatz: Die Rechenregeln sind die Referenz (`docs/ENTWICKLUNG.md`, Abschnitt «Rechenregeln»), nicht das frühere Programm.
Jede Regel ist mit einem von Hand nachgerechneten Fall belegt; dazu kommen Invarianten an vielen zufälligen
Planungen, statistische Prüfungen der Monte-Carlo-Pfade, Robustheit gegen fehlerhafte Eingaben und Tests der
Schnittstellen. Ziel: mindestens 80 % Zeilenabdeckung je Paket (`vivaplan-engine`, `vivaplan-app` und `vivaplan-cli`, getrennt gemessen). Die Schwelle steht nur in `tools/coverage-min`; der pre-push-Hook und der Workflow `test-quality.yml` (Job `tests-coverage`, der einzige Lauf der Rust-Tests mit Abdeckung je Push) lesen sie dort.

## Schichten

| Schicht | Datei | Was geprüft wird |
|---|---|---|
| 1 Einheiten | `crates/engine/src/*.rs` (`#[cfg(test)]`) | Steuertabellen, Kurven, Monate, CSV, SNB-Format, Zufallszahlen, Obligationenformel, Szenario-Kombinationen |
| 2 Referenzfälle | `crates/engine/tests/reference.rs`, `crates/engine/tests/simulation.rs`, `crates/engine/tests/household.rs` (zweite Person, Strategien, Perzentile) | Je Regel ein kleiner Fall mit Rechnung im Kommentar: Teuerung (anteilig ab Start), PK-Zins (monatlich aufgelaufen, anteilig beim Bezug), Kapitalsteuern im selben Jahr, Einkommens- und Vermögenssteuer, Grundstückgewinnsteuer (Stufen, Zürcher Tabelle, Modell Aargau, Steuerfuss-Option, Besitzdauer in vollen Jahren, Freigrenze), Inflation je Immobilie (Wert, Nebenkosten, Miete), Tragbarkeit beim Kauf und bei jeder Hypothekenerneuerung (Einmalrückzahlung), Perzentile je Szenario, Immobilienereignisse, Entnahmestrategie bei Zuflüssen (`crates/engine/tests/inflow_strategy.rs`), 3-Topf nach Spezifikation mit Netto-Bedarf, Januar-Auffüllung, Renditekriterium und den Fällen A/B/C inkl. «alle Töpfe leer» (`crates/engine/tests/buckets_need.rs`), Hypothekarzins, SARON-Untergrenze, Amortisation direkt/indirekt, Säule 3a, Kauf/Verkauf, Kauf nach Verkauf, Tragbarkeit, Notverkauf, Miete, 3-Topf, «aufgebraucht», Fortschreibung |
| 2b Einzelregeln und Funktionen | `crates/engine/tests/sim_rules.rs` (Einzelregeln mit nachgerechneten Werten, #40), `crates/engine/tests/strategy.rs` (Anlagestrategie, #69), `crates/engine/tests/buckets_guardrails.rs` (3-Topf mit Guardrails, Fixe Rate, VPW, Endowment), `crates/engine/tests/cash_housing.rs` (Lebenshaltung mit Wohnkosten, Cash getrennt vom Portfolio), `crates/engine/tests/income_indexed.rs` (Einkommen mit Teuerung), `crates/engine/tests/pillar3a.rs` und `crates/engine/tests/pillar3a_plan.rs` (Säule 3a, indirekte Amortisation, Auszahlungsplan), `crates/engine/tests/reserve_standard_only.rs` (Cash-Reserve nur bei «Standard»), `crates/engine/tests/milestones_limits.rs` (feste Meilensteine und ihre Grenzen) | Je Funktion von Hand nachrechenbare, erfundene Planungen; die Tests nennen Issue und Abnahmekriterium |
| 2c Eingabeprüfung | `crates/engine/tests/mandatory_inputs.rs`, `crates/engine/tests/validate_extra.rs` | Jede Zeile der Pflichteingaben (`docs/Pflichteingaben.md`) löst ihren Fehler mit der erwarteten Meldung aus; Person 2, Grenzen der Monte-Carlo-Angaben, Warnungen zu Strategie und Variationen |
| 3 Invarianten | `crates/engine/tests/invariants.rs`, `crates/engine/tests/proptest_plans.rs` | 500 zufällige Planungen; jeder Test verlangt, dass mehr als die Hälfte ohne Prüffehler gerechnet wird (#81) (Immobilien, Hypotheken, 3a, Steuern, beide Strategien): keine ungültigen Zahlen, Total = Liquid + Immobilien + Vorsorge, Liquid = Cash + Aktien + Obligationen, Steuern total = Summe der Monatssteuern, Ergebnis reproduzierbar; ohne Renditen/Steuern ändert sich das Cash genau um die Geldflüsse; Kauf und Verkauf zum Buchwert ändern das Gesamtvermögen nicht. **proptest (#85):** dieselben Bilanzen über alle Renditequellen, alle sechs Entnahmestrategien (dazu 3-Topf mit Guardrails) und Haushalte mit Partner; Monte Carlo mit 20 Pfaden. Ein Fehlschlag wird verkleinert und in `crates/engine/tests/proptest-regressions/proptest_plans.txt` gespeichert (einchecken; die Fälle laufen dann bei jedem Lauf zuerst) |
| 4 Daten und Statistik | `crates/engine/tests/data.rs`, `crates/engine/tests/library.rs`, `crates/engine/tests/jst.rs` (Weltindex aus den JST-Daten), `crates/engine/tests/proptest_parsers.rs` (Parser und CSV für beliebige Eingaben, Fallzahl über `PROPTEST_CASES`) | Shiller-Import gegen die Rohspalten der Originaldatei, ausgelieferte Reihen = Import, CSV-Fehlerfälle, Bootstrap erhält Mittelwert, Streuung und Blocklänge, Kurssprünge mit erwarteter Häufigkeit, Szenario mit Ausgangswert = Grundlauf, geordnete Bandbreiten, Datenbibliothek |
| 5 Robustheit | `crates/engine/tests/robustness.rs` | 600 zufällig verfälschte Planungen über alle Renditequellen, je zur Hälfte nur Zahlen durch Extremwerte ersetzt (prüft die Rechnung) und beliebig verfälscht (falsche Verweise, Typwechsel, gelöschte Felder; prüft Lesen und Eingabeprüfung). Mindestens ein Viertel der ersten und ein Zehntel der zweiten Hälfte wird gerechnet (#81): Prüfung, Vorschau und Simulation stürzen nie ab und liefern keine ungültigen Zahlen |
| 5b QA-Matrix | `crates/engine/tests/qa_matrix.rs` (Plan und Bericht: `QA.md`) | Eingabeprüfung (auch Besitzdauer-Tabelle und fehlendes Kaufdatum, Dateiformat), Planungsdauer 30–100 Jahre, Meilensteine, PK/3a/AHV, altersabhängige Ausgaben, Anlage- und Entnahmeparameter, Hypotheken, Steuerinterpolation, historische Krisenstarts, Monte-Carlo-Reproduzierbarkeit, Szenario-Konsistenz, Dateien, Last- und Stresstests |
| 5c Golden Masters | `crates/engine/tests/golden.rs`, `crates/engine/tests/golden/*.json` | Zehn feste Planungen mit eingecheckten Ergebnissen; jede unbeabsichtigte Änderung der Rechenergebnisse fällt auf (Aktualisieren mit `GOLDEN_UPDATE=1`). Gespeichert sind die Monatswerte jedes Januars und Julis, der letzte Monat und je Zahlenfeld die Summe über alle Monate, damit auch eine Änderung in einem anderen Monat auffällt (#82). **Release-Profil (#146):** Nächtlich laufen Golden Masters und Referenztabelle zusätzlich im ausgelieferten Profil `release` (Job `release-golden` in `test-quality.yml`, ohne Überlaufprüfung, mit derselben Toleranz); alle übrigen Tests laufen im Testprofil |
| 5d Referenz-Tabelle | `crates/engine/tests/ods_reference.rs`, `crates/engine/tests/golden/Test Retirement.ods` | Basisrechnung des Altprogramms als Tabelle; das Programm muss die Monatswerte der Tabelle auf den Rappen treffen (Blatt 1 bis zur Amortisation, Blatt 2 bis zur Pensionierung); bekannte Fehler der Tabelle sind festgehalten |
| 5e Optimierung und Abbruch | `crates/engine/tests/optimize_ranges.rs`, `crates/engine/tests/cancel.rs`, `crates/engine/tests/optimize_goal.rs` (Ziel «Kein Notverkauf»), `crates/engine/tests/optimization_settings.rs` (Einstellungen in der Planung), `crates/engine/tests/rerun_paths.rs` (Läufe einer Variationsstufe nachrechnen) | Parameter je Strategie fix oder in Bandbreite (Katalog mit Einheit und Planwert, Ungültiges abgelehnt), Entnahmerate im Ergebnis; Fortschrittszähler und Abbrechen von Simulation und Optimierung |
| 5f Dateien und Formate | `crates/engine/tests/legacy_format.rs`, `crates/engine/tests/legacy_import.rs`, `crates/engine/tests/example_neu.rs`, `crates/engine/tests/upgrade_api.rs`, `crates/engine/tests/proptest_upgrade.rs`, `crates/engine/tests/files.rs` | Altes Format `.rplan` bleibt lesbar, Import alter `config.json`, mitgelieferte Planung `Neu.vplan` rechenbar, Nachführen älterer Planungen (`api::upgrade_plan`). **Idempotenz (#137):** Ein zweites Nachführen ändert nichts und meldet nichts, als Eigenschaftstest über verfälschte Planungen (Beispiel, `Neu.vplan`, Altformat, Golden-Planungen; die Grenzen `limits` bleiben unverfälscht, sie kommen aus den geprüften Einstellungen); die mitgelieferten Planungen brauchen kein Nachführen. Atomares Speichern (#136) |
| 6 Schnittstellen | `crates/cli/tests/cli.rs`, `tools/ui-smoke.sh`, `crates/engine/tests/bench.rs` (Messung der Rechenzeit für die Dauerschätzung), Vertragstest `crates/app/src/main.rs` (`issue_142_…`) mit der Tabelle `crates/app/ipc-commands.json` | Alle Befehle der Kommandozeile (Rechnen, Prüfen, Import, Bibliothek, Leistungstest, Schema) und der Entwicklungsserver (Oberfläche, alle API-Befehle, Fehlerfälle). Oberfläche: jede Ansicht und Registerkarte mit allen Renditequellen in Chrome ohne Fenster, ohne JavaScript-Fehler, mit gezeichneten Grafiken. **Vertrag Desktop-App ↔ Entwicklungsserver (#142):** Die Oberflächentests laufen gegen den Entwicklungsserver, dessen Rechenbefehle `engine::dispatch` beantwortet. Der Vertragstest schickt jeden Rechenbefehl der Tabelle mit den Argumenten der Oberfläche durch die Tauri-IPC der Desktop-App und durch `engine::dispatch` und verlangt dieselbe Antwort (ohne Rechen- und Buildzeit); ein neuer Befehl ohne Eintrag in der Tabelle lässt ihn fehlschlagen |

Nicht automatisch getestet: die Fenster- und Dialogschicht der Desktop-App (`crates/app`, nur Weiterleitung an
den Rechenkern) und die Darstellung der Grafiken im Detail (Sichtkontrolle mit Bildschirmfotos).

Alle Dateien in `crates/engine/tests/` sind Module der einen Testdatei `all.rs` (#43); eine neue Datei muss dort
eingetragen werden, sonst schlägt ein Test fehl.

Die Bedienung der Oberfläche prüft Playwright in `e2e/` (176 Tests, Stand 16.11.2026: `ui.spec.js`, `optimum.spec.js`
und getrennt `ui-cancel.spec.js`, weil der Abbruch für alle laufenden Berechnungen gilt; installiertes Google Chrome,
lokaler Entwicklungsserver).

### Oberflächentests unabhängig von der Rechnerlast (#111)

Ob ein Oberflächentest grün ist, darf nicht davon abhängen, wie schnell der Rechner ist oder wie viele Tests parallel
laufen. Regeln:

- **Auf Zustände warten, nie auf Zeit.** Kein `waitForTimeout`, kein `setTimeout` mit Wartezeit in der Seite. Signale
  der App auf `<html>`: `data-ready` (gestartet), `data-job` (Berechnung läuft), `data-rendered` (Ergebnis gezeichnet),
  `data-preview` (`pending`/`idle`: eine Vorschau steht aus). Die Testhilfe `previewIdle(page)` wartet auf die Vorschau,
  bevor ein Test nach einer Auswahl sofort weitertippt. Ins Menü klickt die Testhilfe `nav(page, titel)`: Jede Vorschau
  zeichnet das Menü neu, und ein Klick, der in eine Vorschau fällt, ginge verloren.
- **Eingaben verlassen.** Geprüft und gerechnet wird beim Verlassen des Felds (#57); ein Test, der ein berechnetes
  Ergebnis erwartet, drückt nach `fill` die Tabulatortaste und verlässt sich nicht auf eine noch ausstehende Vorschau.
- **Kleine Rechnungen.** Die Testzugänge `#results/…` und `#runsflow` rechnen Bootstrap und Parametrisch mit 500 Pfaden
  und dem Zufallsstartwert 42: schnell und bei jedem Lauf gleich. Tests, die selbst rechnen, setzen wenige Pfade.
- **Keine Messung im Hintergrund.** Die Testhilfe `open()` öffnet mit dem Testzugang `#nobench`; die App misst dann die
  Rechenzeit nicht (Anzeige «Dauer wird geschätzt…», keine Rückfrage «Lange Berechnung»). Nur Tests, die die echte
  Messung prüfen, öffnen mit `open(page, …, { bench: true })`.
- **«Es geschieht nichts»** prüfen die #57-Tests mit der Uhr von Playwright (`page.clock.install()`,
  `page.clock.runFor(800)`): Die Zeit springt vor, jeder fällige Timer läuft, und die Seite zählt die Anfragen an den
  Rechenkern; während der Eingabe darf keine starten.
- **Keine eigenen Timeouts.** Es gelten die Grenzwerte aus `e2e/playwright.config.js` als Schutz gegen Hänger. Ausnahme
  sind die Abbruchtests in `ui-cancel.spec.js`, die absichtlich lange rechnen.

Geprüft am 16.11.2026 auf einem Rechner mit 10 Kernen: mit 2 Workern (wie der GitHub-Runner) und mit 10 Workern je
mehrmals hintereinander grün; der ganze Satz dauert mit 2 Workern rund 1 Minute (vorher rund 5 Minuten). Bei starker
Überbelegung (16 Worker) kann einzelnes an den Grenzwerten scheitern; das ist gewollt, die Grenzwerte schützen vor Hängern.

## Ausführen

```sh
make test                                        # Schichten 1–6 (Rust), mit nextest
make ui-unit                                     # Hilfsfunktionen der Oberfläche (node --test, ohne Browser)
make clippy                                      # clippy, Warnungen sind Fehler (wie im Workflow, #88)
tools/ui-smoke.sh                                # Oberfläche (Rauchtest)
(cd e2e && npm install && npx playwright test)   # Oberfläche (Bedienung, 176 Tests)
make coverage                                    # Abdeckung (cargo llvm-cov mit nextest)
```

Die Rust-Tests laufen mit `make test` (`cargo nextest run --workspace`), nicht mit `cargo test`. Grund: nextest startet
jeden Test in einem eigenen Prozess. Unter `cargo test` laufen die Tests als Threads eines Prozesses und teilen sich den
prozessweiten Fortschritt und Abbruch (`crates/engine/src/progress.rs`); ein Abbrechen-Test kann dann eine gleichzeitig
laufende Berechnung eines anderen Tests abbrechen. Hook, CI und Abdeckung verwenden deshalb ebenfalls nextest.

Die reinen Hilfsfunktionen der Oberfläche (Zahlenformate, Monate, Pfadzugriff, Steuersatz, Dauerschätzung) liegen in
`crates/app/ui/util.js` und haben Unit-Tests in `e2e/unit/util.test.js` (`node --test`, unter 1 s, #87). Der Job
`ui-unit` in `test-quality.yml` führt sie bei jedem Push aus. Neue Funktionen ohne DOM und ohne Zustand der Planung
gehören in `util.js`.

## Fuzzing

`make fuzz` (cargo-fuzz, Nightly, eigener Workspace `fuzz/`) und nächtlich `test-quality.yml` führen jedes Ziel aus:
`parse_csv`, `parse_snb` und `validate_plan` auf rohe Eingaben, `run_plan` (#86) auf die ganze Rechnung, `upgrade_plan`
(#137: beliebiges JSON durch das Nachführen beim Öffnen und danach die Prüfung) und `import_legacy` (#137: beliebiger Text
durch den Import alter `config.json`). `run_plan`
verändert die Beispielplanung strukturiert (je vier Bytes: Feld, Art der Änderung, Wert; höchstens 255 Monte-Carlo-Pfade,
keine Variationen) und ruft `api::run` auf; Absturz oder ungültige Zahl im Ergebnis ist ein Fund. Der Startkorpus liegt
in `fuzz/seeds/<Ziel>/` (für `run_plan`: alle Renditequellen mit allen Strategien), neue Eingaben in `fuzz/corpus/<Ziel>/`
(nicht eingecheckt, nächtlich im Cache von GitHub Actions). Ein Fund lässt den Lauf fehlschlagen; die Eingabe liegt im
Artefakt `fuzz-artifacts`.

## Gefundene Fehler

Mit dieser Strategie gefunden und behoben (November 2026):

1. Erste Teuerungsanpassung rechnete ein volles Jahr, auch bei Start unter dem Jahr.
2. Pensionskasse verzinste Beiträge vom November für das ganze Jahr.
3. Beim Kapitalbezug unter dem Jahr ging der Zins seit Januar verloren.
4. Eine während der Simulation gekaufte Immobilie wurde zum geplanten Datum nie verkauft.
5. Grundstückgewinnsteuer: Stufen wurden umsortiert, der Gewinn über der letzten Stufe blieb steuerfrei.
6. Grundstückgewinnsteuer und 3a-Kapitalsteuer fehlten in «Steuern total».
7. «Vermögen aufgebraucht» auch ohne Vermögen, aber bei gedeckten Ausgaben.
8. Kaufdatum vor dem Simulationsstart wurde stillschweigend ignoriert.
9. Fortschreibung mit fixen Werten verwendete immer den Wert des Startmonats.
10. Absturz durch Zahlenüberlauf bei extremen Laufzeiten oder Altersangaben.
11. Jahressätze unter −100 % ergaben ungültige Zahlen.

12. Import akzeptierte beliebiges JSON als alte Konfiguration.
13. Geburtsdatum nach dem Simulationsstart und Alter über 120 Jahre am Ende wurden nicht beanstandet.

## Abdeckung

Zeilenabdeckung Rust (`make coverage`, cargo llvm-cov mit nextest), 14. November 2026: **95,9 %** über den
Arbeitsbereich (12'337 Zeilen, 503 nicht ausgeführt). Je Paket (Schwelle 80 %): Rechenkern `vivaplan-engine` 98,1 %,
Desktop-App `vivaplan-app` 89,9 %. Rechenkern: Simulation 99,3 %, Monte Carlo 100 %, Steuern 99,0 %, Optimierung
98,9 %, Import alter Konfigurationen 98,8 %, Prüfung 96,5 %, Schnittstelle 97,1 %; Kommandozeile 94,3 %. Seit #105 gilt die Schwelle auch für `vivaplan-cli`; der Dev-Server (`devserver.rs`, vorher nur von den Oberflächentests benutzt) hat eigene Rust-Tests.
Der Entwicklungsserver (`crates/cli/src/devserver.rs`) erscheint mit 0 %, weil er im Test als eigener Prozess läuft und
beendet wird, bevor er Messdaten schreibt; seine Befehle sind in `crates/cli/tests/cli.rs` geprüft.

## Testqualität: Mutationstests, Abdeckung des geänderten Codes, Übersichtsseite

Abdeckung zeigt, was ausgeführt wird, nicht was geprüft wird. Deshalb gelten drei Dinge nebeneinander:

- **Untergrenze:** mindestens 80 % Zeilenabdeckung je Paket (Schwelle in `tools/coverage-min`). Sie ist eine Untergrenze, kein Ziel.
- **Abdeckung des geänderten Codes (#39):** `make coverage-diff` (Basis mit `BASE=<Zweig>`, Vorgabe `origin/main`) zeigt, welcher Anteil der geänderten Rust-Zeilen von Tests ausgeführt wird, mit den nicht abgedeckten Zeilen. Dasselbe steht in der Zusammenfassung des Laufs von `test-quality.yml` und auf der Übersichtsseite: bei einem Push auf `main` gegenüber dem Stand vor dem Push (das Projekt arbeitet ohne Pull Requests), bei einem Pull Request gegenüber dem Basis-Zweig, beim Start von Hand gegenüber der Eingabe `base` (leer = Vorgänger des letzten Commits; `gh workflow run test-quality.yml -f base=<Commit>`). Lokal vor dem Push zeigt `make coverage-diff` mit `origin/main` denselben Bereich, nämlich die noch nicht gepushten Commits (dazu den Arbeitsbaum). Ein Hinweis, keine Sperre.
- **Mutationstests (#38, #40, #43):** `cargo mutants` verändert den Rechenkern und prüft, ob ein Test fehlschlägt.
  - Der `rust-test-agent` lässt vor dem Push `cargo mutants --in-diff` auf die geänderten Zeilen laufen. Sein Ziel ist «keine neuen nicht erkannten Mutanten im Rechenkern», nicht nur «80 % erreicht». Er schreibt keine Tests ohne Prüfung.
  - **Nächtlich auf GitHub:** Da das Projekt ohne Pull Requests arbeitet, läuft der Job `mutants-diff` in `test-quality.yml` auch nächtlich (nach einem neuen Commit). Verglichen wird mit dem letzten erfolgreichen nächtlichen Lauf (sonst mit dem Stand vor 24 Stunden); die Basis steht in der Zusammenfassung des Laufs. Nicht erkannte Mutanten lassen den Lauf fehlschlagen.
  - **Einzeln beurteilen, kein Score-Ziel (#40):** Jeder nicht erkannte (missed) Mutant in `vivaplan-engine` wird mit einem Test getötet oder mit Begründung ausgeschlossen (`#[mutants::skip]` oder `exclude_re` in `.cargo/mutants.toml`, mit Kommentar; Ausschlüsse ohne Begründung sind nicht erlaubt). Einen Mutation Score als Zielwert gibt es nicht. Zwei Arten von Ausschlüssen sind zulässig: **gleichwertige** Mutanten (das Ergebnis bleibt bei jeder Eingabe gleich) und **Grenzfälle** (der Mutant ändert nur das Verhalten bei exakter Gleichheit zweier gerechneter Frankenbeträge oder innerhalb der Rundungstoleranz von 0.005). Die Einträge sind an Datei, Spalte und Funktion gebunden; verschiebt sich ein Ausdruck, erscheint der Mutant wieder und wird neu beurteilt. Die offenen missed Mutanten stehen unten (Stand #40), die des letzten vollen Laufs in der Zusammenfassung von `mutation-full.yml`, lokal in `local/mutants.out/missed.txt` und auf der Übersichtsseite (siehe unten); sie werden **pro Version durchgesehen**.
  - **Laufzeit (#43, #116):** Jeder Mutant hat höchstens etwa 1 Minute (`minimum_test_timeout = 60` und `timeout_multiplier = 1.0` in `.cargo/mutants.toml`; bis #116 waren es 2 Minuten); ein Timeout zählt als erkannt. Mutationsläufe verwenden das nextest-Profil `mutants` (`.config/nextest.toml`), das zehn schwere Tests überspringt (Testlauf rund 4 s statt rund 45 s). Die Begründung je Test steht in der Konfiguration; die Tests laufen weiter in `make test` und in CI. nextest führt keine Doctests aus: ein «missed» in einer Zeile, die nur ein Doctest prüft, ist kein echter Befund. Der volle Lauf auf GitHub (`mutation-full.yml`) verwendet 16 Shards.
  - **Warum ein Lauf Stunden statt Minuten dauerte (#43, Analyse):** Das Bauen eines Mutanten wuchs im Lauf von 85 s auf über 900 s, die Tests selbst brauchten nur wenige Sekunden. Ursache: Die Integrationstests waren 24 einzelne Programme, und jedes Programm hinterliess rund 350 Objektdateien (Debug-Informationen) im Zielverzeichnis; nach 800 Mutanten lagen je Arbeitskopie rund 285'000 Dateien. macOS prüft jedes neu gebundene Programm, und die Zeit dafür wuchs mit der Dateizahl; vier bis acht parallele Arbeitskopien warteten gegenseitig auf die Platte. Dazu kamen Timeouts von 2 Minuten, die ein Viertel der Laufzeit der Tests selbst ausmachten.
  - **Massnahmen (Qualität der Ergebnisse unverändert):** (1) Die Integrationstests sind ein Programm, `crates/engine/tests/all.rs` mit einem Modul je Datei (Testnamen `<datei>::<test>`; ein Test prüft, dass keine Datei fehlt; **neue Testdateien dort eintragen**). (2) Eigenes Build-Profil `mutants` in `Cargo.toml` ohne Debug-Informationen und mit `opt-level = 1`; Überlaufprüfung und `debug_assert` bleiben an. (3) `make mutants` nutzt 8 parallele Arbeitskopien. Zeitgrenze, Testauswahl und Mutantenliste sind unverändert.
  - **Speichergrenze je Testprozess (#116):** Ein Mutant kann eine Schleife endlos machen, die Vektoren füllt. Am 16.11.2026 brachte so der Mutant `montecarlo.rs:299` (`k += len` zu `k *= len`) einen Mac mit 16 GB zweimal zum Absturz (Kernel-Panic, Speicher erschöpft; 16 Testprozesse belegten 26,5 GB), auch mit `-j 2`; ein Neustart mit `--iterate` traf ihn wieder. Seither gilt:
    - Die Testprogramme von Engine (Unit-Tests und `tests/all.rs`), App und Kommandozeile setzen `engine::test_alloc::Limited` als globalen Speicherverwalter. Er zählt die belegten Bytes und lässt eine Anforderung über **1,5 GB je Prozess** fehlschlagen; Rust bricht den Prozess dann ab («memory allocation of … bytes failed»), der Test schlägt fehl, der Mutant gilt als erkannt. macOS selbst kennt keine solche Grenze (`ulimit -v` wirkt nicht). Zum Vergleich: Der schwerste Test braucht rund 0,7 GB (`make test`), die Tests des Profils `mutants` höchstens 0,2 GB. Die App verwendet weiter den Speicherverwalter des Systems. `test_alloc.rs` ist vom Mutieren ausgenommen (Testinfrastruktur).
    - Das nextest-Profil `mutants` startet höchstens 4 Testprozesse je Job (`test-threads = 4`, sonst so viele wie Kerne). Ein Job bleibt damit auch im schlimmsten Fall unter 6 GB.
    - Die Schleife in `Parametric::path` prüft mit `debug_assert`, dass sie vorankommt; die Mutanten an Zeile 288 und 299 enden dadurch sofort mit einem Fehler.
  - **App und Kommandozeile (#106):** Der Mutationslauf umfasst alle drei Pakete (`--workspace`); nur `crates/cli/src/main.rs` (reine Befehlsverteilung) ist ausgeschlossen. Messung 15.11.2026 lokal, 4 parallele Kopien: Kommandozeile und `migrate.rs` 79 Mutanten (0 nicht erkannt), App `main.rs` 121 Mutanten in 6 Minuten (84 erkannt, 37 nicht übersetzbar, 0 nicht erkannt). Ausgeschlossen mit Begründung in `.cargo/mutants.toml` sind nur Stellen, die ein Test ohne echtes Fenster, Dialog oder Prozessende nicht erreicht (Beenden, Fensterereignisse, native Dialoge, Menü, Start). Der Test der Einstellungen räumt seinen Schlüssel vor dem Lauf auf, weil ein von einem Mutanten abgebrochener Lauf ihn hinterlassen kann.
  - **Messung (12.11.2026, lokal, 10 Kerne, `make mutants`):** 2965 Mutanten in 86 Minuten (vorher bei gleichem Umfang hochgerechnet über 40 Stunden). Bauen je Mutant rund 8 s, gleichbleibend. Ergebnis: 2009 erkannt, 844 nicht erkannt, 38 Timeouts, 74 nicht übersetzbar. Alle 810 Mutanten des abgebrochenen früheren Laufs haben dasselbe Ergebnis wie vorher. `make mutants` endet mit Fehlercode 3, solange es nicht erkannte Mutanten gibt (Verhalten von cargo-mutants).
  - **Stand nach #40 (12.11.2026, `make mutants`, nach einem Abbruch mit `--iterate` fortgesetzt):** 2926 Mutanten: 2788 erkannt, 40 Timeouts, 87 nicht übersetzbar, 11 nicht erkannt (vorher 844). Die Tests dazu stehen in den Testmodulen der Dateien in `crates/engine/src/` und in `crates/engine/tests/sim_rules.rs` (Rechenregeln der Simulation mit erfundenen, von Hand nachrechenbaren Zahlen). Die 11 sind danach erledigt, **offen ist keiner**:
    - getestet (5): abbezahlte Tranche bei der Erneuerung (`paid_off_tranche_does_not_count_in_the_emergency_criterion`, `new_period_of_a_paid_off_tranche_is_no_renewal`) und Neustart nach Phase 8 mit einem reservierten Verkaufserlös (`restart_after_phase_eight_does_not_count_the_reserved_proceeds`); gezielte Läufe erkennen alle Mutanten an diesen Stellen.
    - ausgeschlossen (6): `bucket_step` mit Topf 1 genau 0 (Grenzfall), die Reservierung von 0 (wirkt nur auf das Protokoll) und Auffüllung C2 und Notverkauf, während ein Verkaufserlös reserviert ist (bewusst nicht getestet: braucht zwei Immobilien und einen Verkauf mit Folgekauf im selben Monat).
  - 47 Ausschlüsse mit Begründung in `.cargo/mutants.toml` (Stand #40; nach #106 56, nach #124 82). Ein Eintrag (`after_sales`, Zeile 1170, seit #124 Zeile 1351) ist an die Zeile gebunden, weil Spalte und Funktion dort auch andere Bedingungen treffen; nach jeder Änderung an den Ausschlüssen zeigt ein Vergleich von `cargo mutants --list` vorher und nachher, dass nur die gemeinten Mutanten wegfallen.
  - **Stand nach #124 (16.11.2026, voller Lauf `cargo mutants --workspace`, Stand 51395f5):** 4211 erkannt, 41 Timeouts, 116 nicht übersetzbar, 103 nicht erkannt. Danach:
    - getestet: Tests mit `issue_124_` im Namen in `crates/engine/tests/pillar3a.rs` (3a-Konto der zweiten Person auch bei Kauf «nach Verkauf», Einzahlung nach der Pensionierung in Reserve, 3-Topf und Guardrails, Netto-Bedarf, Wechsel der Amortisation beim AHV-Bezug, geplante Auszahlung im AHV-Monat), `simulation.rs` (Einmalrückzahlung, Hypothekarzins in der Reserve), `cancel.rs` (`progress::cancel`, kurz, damit er auch im Profil `mutants` läuft), `strategy.rs` (gemeinsame Jahre der Indizes, SARON), den Testmodulen von `calibrate.rs`, `upgrade.rs`, `validate.rs`, `housing.rs` und `model.rs` sowie `create_windows` in der App;
    - ausgeschlossen (26 neue Einträge, 33 Mutanten): gleichwertige Mutanten und Grenzfälle, je mit Begründung im Block «#124» in `.cargo/mutants.toml`. Zwei Einträge waren nach Änderungen an `sim.rs` verrutscht und sind nachgeführt: `after_sales` (Zeile 1170 → 1351) und `stock_return` (Spalte 31 → 49). Der alte Eintrag zu `stock_return` traf seit #103 versehentlich `sim.rs:2010:31` (`- 1.0` der Jahresrendite); diese zwei Mutanten sind wieder im Lauf.

### Übersichtsseite «Testqualität» (#41, #42)

`make report` erzeugt `local/test-quality/report/index.html` aus den vorhandenen Ergebnissen: `target/nextest/ci/junit.xml` (`cargo nextest run --profile ci`), die Abdeckung (`make coverage`, nutzt cargo-llvm-cov), `local/mutants.out/` (`make mutants`), `e2e/results.json` (`npx playwright test`) und über `gh` die Issues. Die Seite funktioniert ohne Netzwerk und wird nicht eingecheckt. Die Oberflächentests laufen in GitHub nächtlich (Job `e2e` in `test-quality.yml`, nur nach einem neuen Commit, auch von Hand startbar; 20 bis 30 Minuten). Das Ergebnis `e2e/results.json` liegt als Artefakt `e2e-results` vor und fliesst in die Übersichtsseite ein; bei einem Fehlschlag liegen Bericht und Traces im Artefakt `e2e-failure`. In GitHub erzeugt `test-quality.yml` (bei jedem Push auf `main`, nächtlich und von Hand) dieselbe Seite im Ordner `report-html/` des Artefakts `test-quality-reports` und schreibt eine Kurzfassung in die Zusammenfassung.

Es gibt **keine Gesamtnote**. Jede Schicht hat ihren eigenen Status (erfüllt, Hinweis, nicht erfüllt, keine Daten) nach einer Regel, die auf der Seite steht:

| Schicht | erfüllt | Hinweis | nicht erfüllt |
|---|---|---|---|
| 1 Hygiene | keine Fehlschläge, keine flaky Tests | flaky Tests | ein Test schlägt fehl |
| 2 Angemessenheit | alle drei Pakete ≥ 80 %, keine nicht erkannten Mutanten | Abdeckung erreicht, aber nicht erkannte Mutanten oder kein Mutationslauf | ein Paket unter 80 % |
| 3 Anforderungsbezug | jedes Abnahmekriterium durch Test belegt, geprüft oder abgehakt | Kriterien ohne Test, ohne Prüfung und ohne Haken | – |
| 4 Ergebnis | jeder durchgerutschte Fehler nennt den Test, der ihn hätte fangen müssen, und den Regressionstest | eine Antwort fehlt | – |

Fehlende Daten erscheinen als «keine Daten», nicht als Fehler. Die Schichten 3 und 4 lesen die Issues nur beim Erzeugen der Seite über `gh` (nicht die App); ohne Anmeldung sind beide «keine Daten», die übrige Seite bleibt vollständig. Ein «Verlauf» vergleicht Abdeckung je Paket, Zahl der nicht erkannten Mutanten, flaky Tests und die Anteile der Kriterien mit Test und der geprüften Kriterien mit der vorherigen Version (Stand je Version in `local/test-quality/history/`).

Die Werkzeuge haben eigene Tests: `python3 -m unittest discover tools/report` (Statusregeln, Kennzeichnung der Tests, Fundort-Auswertung; läuft auch in `test-quality.yml`).

**Kennzeichnung der Tests (#42):** Ein Test, der ein Abnahmekriterium belegt, nennt Issue und Kriterium im Kommentar oder im Namen: `// #127/AK2` (Zweites Kriterium des Issues #127) oder `fn issue_127_ak2_…`; in den Oberflächentests im Testtitel (`#127/AK2`). Gezählt wird die Reihenfolge der Häkchen-Liste des Issues. Führende Nullen spielen keine Rolle (`#08/AK4` und `#8/AK4` gehören zum Issue #8). Die Zuordnung läuft über die Nummer, nicht über den Titel; alte Kennungen (`FEA-…/AKn`, `fea_…_akn`) zählen nicht mehr (#117). In den Oberflächentests genügt `AK2` im Testtitel, wenn der Test in einem `test.describe` steht, dessen Titel genau eine Issue-Nummer nennt (`test.describe("… (#127)")`); nennt der Titel mehrere Nummern, zählt der Test nicht (#113).

**Kriterien je Issue (#113):** Der Abschnitt «3 Anforderungsbezug» teilt jedes Kriterium einer von vier Arten zu, in dieser Rangfolge:

| Art | Bedeutung |
|---|---|
| mit Test | ein gekennzeichneter Test belegt das Kriterium |
| geprüft | eine Prüfung auf Anweisung («Prüfe #Nr», siehe `CLAUDE.md`) hält es im Issue als erfüllt fest |
| nur manuell | abgehakt, ohne gekennzeichneten Test und ohne solche Prüfung |
| ohne | weder Test noch Prüfung noch Haken |

Als Prüfung zählen Kommentare, die mit «Prüfung …», «Erneute Prüfung …», «Nacharbeit AKn …» oder «AKn erfüllt in <Commit>» beginnen. Darin gilt ein Kriterium als geprüft bei einer Zeile «AKn erfüllt», «AKn ok» oder «AKn ist belegt» (auch mehrere: «AK1, AK3 ok»), bei «Alle … Kriterien erfüllt» und bei «Ergebnis: x von y Kriterien ok». «AKn nicht …», «offen», «teilweise» oder «überholt» nehmen es wieder heraus. Je Kriterium zählt die letzte Aussage. Eine Abnahme durch den Anwender («Prüfung … durch den Anwender») zählt nicht als Prüfung, sondern bleibt «nur manuell».

**Fundort durchgerutschter Fehler:** Bug-Issues tragen eines der Labels `found: test` (ein Test fand ihn, vor der Abnahme), `found: acceptance` oder `found: usage`. Der Text beantwortet **«Welcher Test hätte ihn fangen müssen?»** und nennt den **Regressionstest** (Vorlage `.github/ISSUE_TEMPLATE/fehler.md`).
