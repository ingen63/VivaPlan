# Teststrategie

Grundsatz: Die Rechenregeln sind die Referenz (`docs/ENTWICKLUNG.md`, Abschnitt «Rechenregeln»), nicht das frühere Programm.
Jede Regel ist mit einem von Hand nachgerechneten Fall belegt; dazu kommen Invarianten an vielen zufälligen
Planungen, statistische Prüfungen der Monte-Carlo-Pfade, Robustheit gegen fehlerhafte Eingaben und Tests der
Schnittstellen. Ziel: mindestens 80 % Zeilenabdeckung je Paket (`vivaplan-engine` und `vivaplan-app`, getrennt gemessen). Die Schwelle steht nur in `tools/coverage-min`; der pre-push-Hook und der Workflow `test-quality.yml` (Job `tests-coverage`, der einzige Lauf der Rust-Tests mit Abdeckung je Push) lesen sie dort.

## Schichten

| Schicht | Datei | Was geprüft wird |
|---|---|---|
| 1 Einheiten | `crates/engine/src/*.rs` (`#[cfg(test)]`) | Steuertabellen, Kurven, Monate, CSV, SNB-Format, Zufallszahlen, Obligationenformel, Szenario-Kombinationen |
| 2 Referenzfälle | `crates/engine/tests/reference.rs`, `crates/engine/tests/simulation.rs`, `crates/engine/tests/household.rs` (zweite Person, Strategien, Perzentile) | Je Regel ein kleiner Fall mit Rechnung im Kommentar: Teuerung (anteilig ab Start), PK-Zins (monatlich aufgelaufen, anteilig beim Bezug), Kapitalsteuern im selben Jahr, Einkommens- und Vermögenssteuer, Grundstückgewinnsteuer (Stufen, Zürcher Tabelle, Modell Aargau, Steuerfuss-Option, Besitzdauer in vollen Jahren, Freigrenze), Inflation je Immobilie (Wert, Nebenkosten, Miete), Tragbarkeit beim Kauf und bei jeder Hypothekenerneuerung (Einmalrückzahlung), Perzentile je Szenario, Immobilienereignisse, Entnahmestrategie bei Zuflüssen (`crates/engine/tests/inflow_strategy.rs`), 3-Topf nach Spezifikation mit Netto-Bedarf, Januar-Auffüllung, Renditekriterium und den Fällen A/B/C inkl. «alle Töpfe leer» (`crates/engine/tests/buckets_need.rs`), Hypothekarzins, SARON-Untergrenze, Amortisation direkt/indirekt, Säule 3a, Kauf/Verkauf, Kauf nach Verkauf, Tragbarkeit, Notverkauf, Miete, 3-Topf, «aufgebraucht», Fortschreibung |
| 3 Invarianten | `crates/engine/tests/invariants.rs` | 500 zufällige Planungen (Immobilien, Hypotheken, 3a, Steuern, beide Strategien): keine ungültigen Zahlen, Total = Liquid + Immobilien + Vorsorge, Liquid = Cash + Aktien + Obligationen, Steuern total = Summe der Monatssteuern, Ergebnis reproduzierbar; ohne Renditen/Steuern ändert sich das Cash genau um die Geldflüsse; Kauf und Verkauf zum Buchwert ändern das Gesamtvermögen nicht |
| 4 Daten und Statistik | `crates/engine/tests/data.rs`, `crates/engine/tests/library.rs` | Shiller-Import gegen die Rohspalten der Originaldatei, ausgelieferte Reihen = Import, CSV-Fehlerfälle, Bootstrap erhält Mittelwert, Streuung und Blocklänge, Kurssprünge mit erwarteter Häufigkeit, Szenario mit Ausgangswert = Grundlauf, geordnete Bandbreiten, Datenbibliothek |
| 5 Robustheit | `crates/engine/tests/robustness.rs` | 600 zufällig verfälschte Planungen (Extremwerte, falsche Verweise, gelöschte Felder) über alle Renditequellen: Prüfung, Vorschau und Simulation stürzen nie ab und liefern keine ungültigen Zahlen |
| 5b QA-Matrix | `crates/engine/tests/qa_matrix.rs` (Plan und Bericht: `QA.md`) | Eingabeprüfung (auch Besitzdauer-Tabelle und fehlendes Kaufdatum, Dateiformat), Planungsdauer 30–100 Jahre, Meilensteine, PK/3a/AHV, altersabhängige Ausgaben, Anlage- und Entnahmeparameter, Hypotheken, Steuerinterpolation, historische Krisenstarts, Monte-Carlo-Reproduzierbarkeit, Szenario-Konsistenz, Dateien, Last- und Stresstests |
| 5c Golden Masters | `crates/engine/tests/golden.rs`, `crates/engine/tests/golden/*.json` | Zehn feste Planungen mit eingecheckten Ergebnissen; jede unbeabsichtigte Änderung der Rechenergebnisse fällt auf (Aktualisieren mit `GOLDEN_UPDATE=1`) |
| 5d Referenz-Tabelle | `crates/engine/tests/ods_reference.rs`, `crates/engine/tests/golden/Test Retirement.ods` | Basisrechnung des Altprogramms als Tabelle; das Programm muss die Monatswerte der Tabelle auf den Rappen treffen (Blatt 1 bis zur Amortisation, Blatt 2 bis zur Pensionierung); bekannte Fehler der Tabelle sind festgehalten |
| 5e Optimierung und Abbruch | `crates/engine/tests/optimize_ranges.rs`, `crates/engine/tests/cancel.rs` | Parameter je Strategie fix oder in Bandbreite (Katalog mit Einheit und Planwert, Ungültiges abgelehnt), Entnahmerate im Ergebnis; Fortschrittszähler und Abbrechen von Simulation und Optimierung |
| 6 Schnittstellen | `crates/cli/tests/cli.rs`, `tools/ui-smoke.sh` | Alle Befehle der Kommandozeile (Rechnen, Prüfen, Import, Bibliothek, Leistungstest, Schema) und der Entwicklungsserver (Oberfläche, alle API-Befehle, Fehlerfälle). Oberfläche: jede Ansicht und Registerkarte mit allen Renditequellen in Chrome ohne Fenster, ohne JavaScript-Fehler, mit gezeichneten Grafiken |

Nicht automatisch getestet: die Fenster- und Dialogschicht der Desktop-App (`crates/app`, nur Weiterleitung an
den Rechenkern) und die Darstellung der Grafiken im Detail (Sichtkontrolle mit Bildschirmfotos).

Die Bedienung der Oberfläche prüft Playwright in `e2e/ui.spec.js` (50 Tests, installiertes Google Chrome, lokaler
Entwicklungsserver; Abbrechen-Tests getrennt in `ui-cancel.spec.js`, weil der Abbruch für alle laufenden Berechnungen gilt).

## Ausführen

```sh
cargo test                                                              # Schichten 1–6 (Rust)
tools/ui-smoke.sh                                                       # Oberfläche (Rauchtest)
(cd e2e && npm install && npx playwright test)                     # Oberfläche (Bedienung, 50 Tests)
cargo llvm-cov --workspace --exclude vivaplan-app --summary-only  # Abdeckung
```

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

Zeilenabdeckung Rust (`cargo llvm-cov`, ohne Desktop-Hülle), 7. November 2026: **91,3 %** (4'443 Zeilen, 387 nicht
ausgeführt). Rechenkern: Simulation 97,9 %, Monte Carlo 97,8 %, Steuern 98,4 %, Zeitreihen 95,2 %, Import alter
Konfigurationen 94,1 %, Prüfung 91,6 %, Schnittstelle 82,6 %; Kommandozeile 93,8 %.
Der Entwicklungsserver (`crates/cli/src/devserver.rs`) erscheint mit 0 %, weil er im Test als eigener Prozess läuft und
beendet wird, bevor er Messdaten schreibt; seine Befehle sind in `crates/cli/tests/cli.rs` geprüft.

## Testqualität: Mutationstests, Abdeckung des geänderten Codes, Übersichtsseite

Abdeckung zeigt, was ausgeführt wird, nicht was geprüft wird. Deshalb gelten drei Dinge nebeneinander:

- **Untergrenze:** mindestens 80 % Zeilenabdeckung je Paket (Schwelle in `tools/coverage-min`). Sie ist eine Untergrenze, kein Ziel.
- **Abdeckung des geänderten Codes (FEA-28):** `make coverage-diff` (Basis mit `BASE=<Zweig>`, Vorgabe `origin/main`) zeigt, welcher Anteil der geänderten Rust-Zeilen von Tests ausgeführt wird, mit den nicht abgedeckten Zeilen. Dasselbe steht in der Zusammenfassung des Laufs von `test-quality.yml` und auf der Übersichtsseite: bei einem Push auf `main` gegenüber dem Stand vor dem Push (das Projekt arbeitet ohne Pull Requests), bei einem Pull Request gegenüber dem Basis-Zweig, beim Start von Hand gegenüber der Eingabe `base` (leer = Vorgänger des letzten Commits; `gh workflow run test-quality.yml -f base=<Commit>`). Lokal vor dem Push zeigt `make coverage-diff` mit `origin/main` denselben Bereich, nämlich die noch nicht gepushten Commits (dazu den Arbeitsbaum). Ein Hinweis, keine Sperre.
- **Mutationstests (FEA-27, FEA-29, FEA-32):** `cargo mutants` verändert den Rechenkern und prüft, ob ein Test fehlschlägt.
  - Der `rust-test-agent` lässt vor dem Push `cargo mutants --in-diff` auf die geänderten Zeilen laufen. Sein Ziel ist «keine neuen nicht erkannten Mutanten im Rechenkern», nicht nur «80 % erreicht». Er schreibt keine Tests ohne Prüfung.
  - **Nächtlich auf GitHub:** Da das Projekt ohne Pull Requests arbeitet, läuft der Job `mutants-diff` in `test-quality.yml` auch nächtlich (nach einem neuen Commit). Verglichen wird mit dem letzten erfolgreichen nächtlichen Lauf (sonst mit dem Stand vor 24 Stunden); die Basis steht in der Zusammenfassung des Laufs. Nicht erkannte Mutanten lassen den Lauf fehlschlagen.
  - **Einzeln beurteilen, kein Score-Ziel (FEA-29):** Jeder nicht erkannte (missed) Mutant in `vivaplan-engine` wird mit einem Test getötet oder mit Begründung ausgeschlossen (`#[mutants::skip]` oder `exclude_re` in `.cargo/mutants.toml`, mit Kommentar; Ausschlüsse ohne Begründung sind nicht erlaubt). Einen Mutation Score als Zielwert gibt es nicht. Zwei Arten von Ausschlüssen sind zulässig: **gleichwertige** Mutanten (das Ergebnis bleibt bei jeder Eingabe gleich) und **Grenzfälle** (der Mutant ändert nur das Verhalten bei exakter Gleichheit zweier gerechneter Frankenbeträge oder innerhalb der Rundungstoleranz von 0.005). Die Einträge sind an Datei, Spalte und Funktion gebunden; verschiebt sich ein Ausdruck, erscheint der Mutant wieder und wird neu beurteilt. Die offenen missed Mutanten stehen unten (Stand FEA-29), die des letzten vollen Laufs in der Zusammenfassung von `mutation-full.yml`, lokal in `local/mutants.out/missed.txt` und auf der Übersichtsseite (siehe unten); sie werden **pro Version durchgesehen**.
  - **Laufzeit (FEA-32):** Jeder Mutant hat höchstens etwa 2 Minuten (`minimum_test_timeout = 120` und `timeout_multiplier = 1.0` in `.cargo/mutants.toml`); ein Timeout zählt als erkannt. Mutationsläufe verwenden das nextest-Profil `mutants` (`.config/nextest.toml`), das neun schwere Tests überspringt (Testlauf rund 4 s statt rund 45 s). Die Begründung je Test steht in der Konfiguration; die Tests laufen weiter in `make test` und in CI. nextest führt keine Doctests aus: ein «missed» in einer Zeile, die nur ein Doctest prüft, ist kein echter Befund. Der volle Lauf auf GitHub (`mutation-full.yml`) verwendet 16 Shards.
  - **Warum ein Lauf Stunden statt Minuten dauerte (FEA-32, Analyse):** Das Bauen eines Mutanten wuchs im Lauf von 85 s auf über 900 s, die Tests selbst brauchten nur wenige Sekunden. Ursache: Die Integrationstests waren 24 einzelne Programme, und jedes Programm hinterliess rund 350 Objektdateien (Debug-Informationen) im Zielverzeichnis; nach 800 Mutanten lagen je Arbeitskopie rund 285'000 Dateien. macOS prüft jedes neu gebundene Programm, und die Zeit dafür wuchs mit der Dateizahl; vier bis acht parallele Arbeitskopien warteten gegenseitig auf die Platte. Dazu kamen Timeouts von 2 Minuten, die ein Viertel der Laufzeit der Tests selbst ausmachten.
  - **Massnahmen (Qualität der Ergebnisse unverändert):** (1) Die Integrationstests sind ein Programm, `crates/engine/tests/all.rs` mit einem Modul je Datei (Testnamen `<datei>::<test>`; ein Test prüft, dass keine Datei fehlt; **neue Testdateien dort eintragen**). (2) Eigenes Build-Profil `mutants` in `Cargo.toml` ohne Debug-Informationen und mit `opt-level = 1`; Überlaufprüfung und `debug_assert` bleiben an. (3) `make mutants` nutzt 8 parallele Arbeitskopien. Zeitgrenze, Testauswahl und Mutantenliste sind unverändert.
  - **Messung (12.11.2026, lokal, 10 Kerne, `make mutants`):** 2965 Mutanten in 86 Minuten (vorher bei gleichem Umfang hochgerechnet über 40 Stunden). Bauen je Mutant rund 8 s, gleichbleibend. Ergebnis: 2009 erkannt, 844 nicht erkannt, 38 Timeouts, 74 nicht übersetzbar. Alle 810 Mutanten des abgebrochenen früheren Laufs haben dasselbe Ergebnis wie vorher. `make mutants` endet mit Fehlercode 3, solange es nicht erkannte Mutanten gibt (Verhalten von cargo-mutants).
  - **Stand nach FEA-29 (12.11.2026, `make mutants`, nach einem Abbruch mit `--iterate` fortgesetzt):** 2926 Mutanten: 2788 erkannt, 40 Timeouts, 87 nicht übersetzbar, 11 nicht erkannt (vorher 844). Die Tests dazu stehen in den Testmodulen der Dateien in `crates/engine/src/` und in `crates/engine/tests/sim_rules.rs` (Rechenregeln der Simulation mit erfundenen, von Hand nachrechenbaren Zahlen). Die 11 sind danach erledigt, **offen ist keiner**:
    - getestet (5): abbezahlte Tranche bei der Erneuerung (`paid_off_tranche_does_not_count_in_the_emergency_criterion`, `new_period_of_a_paid_off_tranche_is_no_renewal`) und Neustart nach Phase 8 mit einem reservierten Verkaufserlös (`restart_after_phase_eight_does_not_count_the_reserved_proceeds`); gezielte Läufe erkennen alle Mutanten an diesen Stellen.
    - ausgeschlossen (6): `bucket_step` mit Topf 1 genau 0 (Grenzfall), die Reservierung von 0 (wirkt nur auf das Protokoll) und Auffüllung C2 und Notverkauf, während ein Verkaufserlös reserviert ist (bewusst nicht getestet: braucht zwei Immobilien und einen Verkauf mit Folgekauf im selben Monat).
  - 47 Ausschlüsse mit Begründung in `.cargo/mutants.toml`. Ein Eintrag (`after_sales`, Zeile 1170) ist an die Zeile gebunden, weil Spalte und Funktion dort auch andere Bedingungen treffen; nach jeder Änderung an den Ausschlüssen zeigt ein Vergleich von `cargo mutants --list` vorher und nachher, dass nur die gemeinten Mutanten wegfallen.

### Übersichtsseite «Testqualität» (FEA-30, FEA-31)

`make report` erzeugt `local/test-quality/report/index.html` aus den vorhandenen Ergebnissen: `target/nextest/ci/junit.xml` (`cargo nextest run --profile ci`), die Abdeckung (`make coverage`, nutzt cargo-llvm-cov), `local/mutants.out/` (`make mutants`), `e2e/results.json` (`npx playwright test`) und über `gh` die Issues. Die Seite funktioniert ohne Netzwerk und wird nicht eingecheckt. In GitHub erzeugt `test-quality.yml` (bei jedem Push auf `main`, nächtlich und von Hand) dieselbe Seite im Ordner `report-html/` des Artefakts `test-quality-reports` und schreibt eine Kurzfassung in die Zusammenfassung.

Es gibt **keine Gesamtnote**. Jede Schicht hat ihren eigenen Status (erfüllt, Hinweis, nicht erfüllt, keine Daten) nach einer Regel, die auf der Seite steht:

| Schicht | erfüllt | Hinweis | nicht erfüllt |
|---|---|---|---|
| 1 Hygiene | keine Fehlschläge, keine flaky Tests | flaky Tests | ein Test schlägt fehl |
| 2 Angemessenheit | beide Pakete ≥ 80 %, keine nicht erkannten Mutanten | Abdeckung erreicht, aber nicht erkannte Mutanten oder kein Mutationslauf | ein Paket unter 80 % |
| 3 Anforderungsbezug | jedes Abnahmekriterium durch Test belegt oder abgehakt | Kriterien ohne Test und ohne Haken | – |
| 4 Ergebnis | jeder durchgerutschte Fehler nennt den Test, der ihn hätte fangen müssen, und den Regressionstest | eine Antwort fehlt | – |

Fehlende Daten erscheinen als «keine Daten», nicht als Fehler. Die Schichten 3 und 4 lesen die Issues nur beim Erzeugen der Seite über `gh` (nicht die App); ohne Anmeldung sind beide «keine Daten», die übrige Seite bleibt vollständig. Ein «Verlauf» vergleicht Abdeckung je Paket, Zahl der nicht erkannten Mutanten, flaky Tests und den Anteil der Kriterien mit Test mit der vorherigen Version (Stand je Version in `local/test-quality/history/`).

Die Werkzeuge haben eigene Tests: `python3 -m unittest discover tools/report` (Statusregeln, Kennzeichnung der Tests, Fundort-Auswertung; läuft auch in `test-quality.yml`).

**Kennzeichnung der Tests (FEA-31):** Ein Test, der ein Abnahmekriterium belegt, nennt Issue und Kriterium im Kommentar oder im Namen: `// FEA-100/AK2` (Zweites Kriterium des Issues FEA-100) oder `fn fea_100_ak2_…`; in den Oberflächentests im Testtitel (`FEA-100/AK2`). Gezählt wird die Reihenfolge der Häkchen-Liste des Issues.

**Fundort durchgerutschter Fehler:** Bug-Issues tragen eines der Labels `gefunden:test` (ein Test fand ihn, vor der Abnahme), `gefunden:abnahme` oder `gefunden:nutzung`. Der Text beantwortet **«Welcher Test hätte ihn fangen müssen?»** und nennt den **Regressionstest** (Vorlage `.github/ISSUE_TEMPLATE/fehler.md`).
