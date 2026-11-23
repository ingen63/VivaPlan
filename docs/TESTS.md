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
| 3 Invarianten | `crates/engine/tests/invariants.rs`, `crates/engine/tests/proptest_plans.rs` | 500 zufällige Planungen; jeder Test verlangt, dass mehr als die Hälfte ohne Prüffehler gerechnet wird (#81) (Immobilien, Hypotheken, 3a, Steuern, beide Strategien): keine ungültigen Zahlen, Total = Liquid + Immobilien + Vorsorge, Liquid = Cash + Aktien + Obligationen, Steuern total = Summe der Monatssteuern, Ergebnis reproduzierbar; ohne Renditen/Steuern ändert sich das Cash genau um die Geldflüsse; Kauf und Verkauf zum Buchwert ändern das Gesamtvermögen nicht. **Hauptbuch (#133):** Die Veränderung des Gesamtvermögens entspricht jeden Monat den Geldflüssen und Bewertungsänderungen (`sim::Ledger`, nicht im Ergebnis), mit Immobilien, Hypotheken, PK, 3a und Renditen; geprüft in `proptest_plans.rs`, `qa_matrix.rs::check_books` und `golden.rs`. Ein doppelt gebuchter Verkaufserlös lässt die Prüfung fehlschlagen. **proptest (#85):** dieselben Bilanzen über alle Renditequellen, alle sechs Entnahmestrategien (dazu 3-Topf mit Guardrails) und Haushalte mit Partner; Monte Carlo mit 20 Pfaden. Ein Fehlschlag wird verkleinert und in `crates/engine/tests/proptest-regressions/proptest_plans.txt` gespeichert (einchecken; die Fälle laufen dann bei jedem Lauf zuerst); dazu gehört ein benannter Regressionstest (siehe «Funde der Eigenschaftstests») |
| 4 Daten und Statistik | `crates/engine/tests/data.rs`, `crates/engine/tests/library.rs`, `crates/engine/tests/jst.rs` (Weltindex aus den JST-Daten), `crates/engine/tests/proptest_parsers.rs` (Parser und CSV für beliebige Eingaben, Fallzahl über `PROPTEST_CASES`), `crates/engine/tests/statistics.rs` (#148: Vertrauensgrenzen aus der Stichprobengrösse: Bootstrap-Mittel und -Streuung der Monatswerte = Quelle, parametrische Jahres-Logrendite = Modell (μ, σ, Sprünge), Korrelationen von Obligationen und Inflation = Modell (Fisher-z, 99 %); jeder Test zeigt, dass 1 Prozentpunkt Verschiebung pro Jahr auffiele) | Shiller-Import gegen die Rohspalten der Originaldatei, ausgelieferte Reihen = Import, CSV-Fehlerfälle, Bootstrap erhält Mittelwert, Streuung und Blocklänge, Kurssprünge mit erwarteter Häufigkeit, Szenario mit Ausgangswert = Grundlauf, geordnete Bandbreiten, Datenbibliothek |
| 5 Robustheit | `crates/engine/tests/robustness.rs`, `crates/engine/tests/command_sweep.rs` (#221) | 600 zufällig verfälschte Planungen über alle Renditequellen, je zur Hälfte nur Zahlen durch Extremwerte ersetzt (prüft die Rechnung) und beliebig verfälscht (falsche Verweise, Typwechsel, gelöschte Felder; prüft Lesen und Eingabeprüfung). Mindestens ein Viertel der ersten und ein Zehntel der zweiten Hälfte wird gerechnet (#81): Prüfung, Vorschau und Simulation stürzen nie ab und liefern keine ungültigen Zahlen. **Befehls-Sweep (#221):** Jeder Rechenbefehl der Tabelle `crates/app/ipc-commands.json` (Schlüssel `compute`, ein neuer Befehl ist ohne weiteres Zutun dabei) geht mit unsinnigen Argumenten an `engine::dispatch`: ganzes Argument ersetzt, jedes Feld gelöscht oder durch falschen Typ, `null`, leeres Objekt, Extremwerte (0, negativ, sehr gross) oder eine Liste mit 10'000 Einträgen ersetzt, Mengenfelder der Planung extrem; dazu zufällige Änderungen auch in der Planung. Jeder Aufruf endet mit Ergebnis oder Fehlermeldung; ein Panic in irgendeinem Thread (auch Rayon, über einen Panic-Hook) lässt den Test fehlschlagen und nennt Befehl, Änderung und Argumente. Vor jedem Aufruf steht er in der Ausgabe, so nennt die letzte Zeile bei einem Hänger (Zeitgrenze im nextest-Profil `ci`) oder einem Prozessabbruch den Befehl; eine Rechnung über 5 s bricht der Test wie die Oberfläche ab. Fallzahl über `PROPTEST_CASES`: bei jedem Push rund 130 Aufrufe (unter 10 s), nächtlich alle rund 700 systematischen und 625 zufällige. Mit gültigen Argumenten verlangt er ein Ergebnis ohne NaN und ∞: `engine::dispatch` meldet eine ungültige Zahl im Ergebnis als Fehler `result_not_finite` mit dem Feld (serde_json schriebe sie sonst als `null`) |
| 5b QA-Matrix | `crates/engine/tests/qa_matrix.rs` (Plan und Bericht: `QA.md`) | Eingabeprüfung (auch Besitzdauer-Tabelle und fehlendes Kaufdatum, Dateiformat), Planungsdauer 30–100 Jahre, Meilensteine, PK/3a/AHV, altersabhängige Ausgaben, Anlage- und Entnahmeparameter, Hypotheken, Steuerinterpolation, historische Krisenstarts, Monte-Carlo-Reproduzierbarkeit, Szenario-Konsistenz, Dateien, Last- und Stresstests |
| 5c Golden Masters | `crates/engine/tests/golden.rs`, `crates/engine/tests/golden/*.json` | Zehn feste Planungen mit eingecheckten Ergebnissen; jede unbeabsichtigte Änderung der Rechenergebnisse fällt auf (Aktualisieren mit `GOLDEN_UPDATE=1`). Gespeichert sind die Monatswerte jedes Januars und Julis, der letzte Monat und je Zahlenfeld die Summe über alle Monate, damit auch eine Änderung in einem anderen Monat auffällt (#82). **Release-Profil (#146):** Nächtlich laufen Golden Masters und Referenztabelle zusätzlich im ausgelieferten Profil `release` (Job `release-golden` in `test-quality.yml`, ohne Überlaufprüfung, mit derselben Toleranz); alle übrigen Tests laufen im Testprofil. **Golden-Zahlen bei jedem Push (#195):** `GOLDEN_UPDATE=1` macht die Golden Masters auch bei falscher Ausgabe grün. Der Job `golden` in `test-quality.yml` vergleicht deshalb die Golden-Dateien mit dem Stand vor dem Push (`tools/golden_numbers.py --base <vorher> --gate`: Zahlen, Schlüssel, Listenlängen und Monate, Texte nicht) und nennt geänderte Zahlen mit Pfad in der Zusammenfassung. Geänderte Zahlen lassen den Lauf fehlschlagen, ausser einer der gepushten Commits begründet sie mit einer eigenen Zeile `Golden: #Nr Begründung` in der Commit-Nachricht (z. B. `Golden: #190 13. AHV-Rente im Dezember`). Neue Golden-Dateien werden genannt, nicht verglichen; eine gelöschte zählt als Änderung. Lokal vorab: `make golden-numbers BASE=origin/main` |
| 5d Referenz-Tabelle | `crates/engine/tests/ods_reference.rs`, `crates/engine/tests/golden/Test Retirement.ods` | Basisrechnung des Altprogramms als Tabelle; das Programm muss die Monatswerte der Tabelle auf den Rappen treffen (Blatt 1 bis zur Amortisation, Blatt 2 bis zur Pensionierung); bekannte Fehler der Tabelle sind festgehalten |
| 5e Optimierung und Abbruch | `crates/engine/tests/optimize_ranges.rs`, `crates/engine/tests/cancel.rs`, `crates/engine/tests/optimize_goal.rs` (Ziel «Kein Notverkauf»), `crates/engine/tests/optimization_settings.rs` (Einstellungen in der Planung), `crates/engine/tests/rerun_paths.rs` (Läufe einer Variationsstufe nachrechnen) | Parameter je Strategie fix oder in Bandbreite (Katalog mit Einheit und Planwert, Ungültiges abgelehnt), Entnahmerate im Ergebnis; Fortschrittszähler und Abbrechen von Simulation und Optimierung |
| 5f Dateien und Formate | `crates/engine/tests/legacy_format.rs`, `crates/engine/tests/legacy_import.rs`, `crates/engine/tests/example_neu.rs`, `crates/engine/tests/upgrade_api.rs`, `crates/engine/tests/proptest_upgrade.rs`, `crates/engine/tests/files.rs` | Altes Format `.rplan` bleibt lesbar, Import alter `config.json`, mitgelieferte Planung `new.vplan` rechenbar, Nachführen älterer Planungen (`api::upgrade_plan`). **Idempotenz (#137):** Ein zweites Nachführen ändert nichts und meldet nichts, als Eigenschaftstest über verfälschte Planungen (Beispiel, `new.vplan`, Altformat, Golden-Planungen; die Grenzen `limits` bleiben unverfälscht, sie kommen aus den geprüften Einstellungen); die mitgelieferten Planungen brauchen kein Nachführen. Atomares Speichern (#136) |
| 6 Schnittstellen | `crates/cli/tests/cli.rs`, `tools/ui-smoke.sh`, `crates/engine/tests/bench.rs` (Messung der Rechenzeit für die Dauerschätzung), Vertragstest `crates/app/src/main.rs` (`issue_142_…`) mit der Tabelle `crates/app/ipc-commands.json` | Alle Befehle der Kommandozeile (Rechnen, Prüfen, Import, Bibliothek, Leistungstest, Schema) und der Entwicklungsserver (Oberfläche, alle API-Befehle, Fehlerfälle). Oberfläche: jede Ansicht und Registerkarte mit allen Renditequellen in Chrome ohne Fenster, ohne JavaScript-Fehler, mit gezeichneten Grafiken. **Vertrag Desktop-App ↔ Entwicklungsserver (#142):** Die Oberflächentests laufen gegen den Entwicklungsserver, dessen Rechenbefehle `engine::dispatch` beantwortet. Der Vertragstest schickt jeden Rechenbefehl der Tabelle mit den Argumenten der Oberfläche durch die Tauri-IPC der Desktop-App und durch `engine::dispatch` und verlangt dieselbe Antwort (ohne Rechen- und Buildzeit); ein neuer Befehl ohne Eintrag in der Tabelle lässt ihn fehlschlagen |

**Funde der Eigenschaftstests (#223):** Findet ein proptest-Test einen Fehler, wird der verkleinerte Fall im selben
Commit wie die Korrektur zusätzlich ein benannter Test (`issue_<Nr>_…`) mit der Eingabe als lesbarem Wert und einem
Kommentar, welcher Fehler es war; er schlägt ohne die Korrektur fehl. Die Regressionsdateien unter
`crates/engine/tests/proptest-regressions/` bleiben eingecheckt, dokumentieren den Fund aber nicht (nur Prüfsumme und
Kurzform). War der Fund kein Fehler im Code, sondern ein falsch formulierter Test, wird der Test korrigiert und der
Eintrag entfernt. Beispiel: `proptest_tax.rs::issue_223_ak2_…` (Fehler im Prüforakel von #134, Eintrag entfernt).

**Plattformen (#141, #225):** Die Rust-Tests laufen bei jedem Push unter Linux (mit Abdeckung) und unter Windows (ohne
Abdeckung, ohne Mutationstests), nächtlich zusätzlich unter Windows und macOS (Job `tests-os` in `test-quality.yml`,
Runner von GitHub); macOS bleibt nächtlich. Beim Push läuft Windows nur, wenn Rust-Code, `Cargo.toml`/`Cargo.lock` oder
`test-quality.yml` geändert wurden (Ausgabe `rust` des Jobs `changes`). Budget: 300 Actions-Minuten pro Woche für den
Windows-Job beim Push (Schätzung in #225); wird es überschritten, testet Windows beim Push nur `vivaplan-engine` und die
Pfadtests der App. Windows-Pfade (Laufwerk, Backslash, UNC, Umlaute) prüft
`issue_141_ak3_windows_paths` in `crates/app/src/main.rs` (nur unter Windows ausgeführt, überall übersetzt, sonst
übersprungen mit Grund, #224). Vor dem
Release-Entwurf startet `pakete.yml` jedes gebaute Programm mit `--smoke`: Fenster und Webansicht öffnen, Beispielplanung
rechnen, beenden (Code 0 = in Ordnung, Meldung in `VIVAPLAN_SMOKE_OUT`; Linux in `xvfb`). Bis auf Weiteres baut
`pakete.yml` nur das `.dmg` für macOS, der Rauchtest läuft also nur dort (#202); Linux und Windows schaltet #203 wieder ein.

**Übersprungene Tests (#224):** Tests, die etwas Besonderes brauchen, dürfen lokal übersprungen werden, aber nie still:
Sie fragen `engine::test_resource` (`crates/engine/src/test_resource.rs`), das den Grund auf stderr schreibt
(`übersprungen: es fehlt …`; unter nextest sichtbar mit `--success-output immediate`). Je Ressource gibt es eine
Umgebungsvariable: leer oder nicht gesetzt = überspringen, `1` = Pflicht (fehlt die Ressource, schlägt der Test fehl),
jeder andere Wert = Fehler, der die Variable und den erlaubten Wert nennt.

| Variable | Ressource | Tests | Gesetzt in (`test-quality.yml`) |
|---|---|---|---|
| `VIVAPLAN_WINDOWS_REQUIRED` | Windows | `issue_141_ak3_windows_paths` (App) | `tests-os` unter Windows (Push und nächtlich) |
| `VIVAPLAN_UNIX_REQUIRED` | Unix (macOS, Linux): Dateirechte, symbolische Links | `issue_136_ak2_…`, `issue_136_saving_through_a_symlink_…` (Engine, `files.rs`), `issue_136_ak4_…` (App) | `tests-coverage` (Linux), `tests-os` unter macOS (nächtlich) |
| `VIVAPLAN_DATA_REQUIRED` | Shiller-Originaltabelle `crates/engine/data/quellen/shiller-index.ods` (nur lokal, #94) | `data.rs` (Shiller-Spalten), `library::shipped_series_match_import`, `library_import_commands` (CLI), `shiller_import_from_table_and_error_case` (App) | kein Job (die Tabelle liegt nicht im Repository) |

Unter Windows sind die Unix-Tests eigene Platzhalter gleichen Namens (`#[cfg(not(unix))]`), die nur überspringen oder
fehlschlagen. Die drei Last- und Messtests in `qa_matrix.rs` sind `#[ignore = "Grund"]`: zwei laufen nächtlich im
Release-Profil (`release-golden`, `--run-ignored only`), `benchmark_paths` nur von Hand. Die Oberflächentests
(`e2e/`, `tools/ui-smoke.sh`) überspringen nichts: fehlt Chrome, schlagen sie fehl. Plattformabhängige Prüfungen
innerhalb eines Tests (`#[cfg(target_os = "macos")]` in den Menütests) sind kein Überspringen; der Test läuft überall.
Neue bedingte Tests verwenden `test_resource` und tragen sich hier ein.

**Netzwerkfreiheit (#157):** Die App verspricht, keine Netzwerkzugriffe zu machen. Zwei Prüfungen belegen das
automatisch. `tools/check-network-free.sh` (`make network-free`, Job `network-free` in `test-quality.yml` bei jedem Push)
führt `cargo tree -e normal,build` für `vivaplan-app`, `vivaplan-engine` und `vivaplan-cli` aus, ohne
Entwicklungsabhängigkeiten, je Desktop-Ziel (Linux, Windows, macOS, je x86_64 und aarch64), und schlägt fehl, sobald ein
Crate der Sperrliste im Skript vorkommt (u. a. `reqwest`, `hyper`, `h2`, `ureq`, `curl`, `tokio-tungstenite`,
`tauri-plugin-http`, `tauri-plugin-updater`). Verglichen werden exakte Crate-Namen, keine Teilstrings (`dispatch2` ist
nicht `h2`). Nicht `--target all`: `tauri` zieht für Android und iOS `reqwest` und `hyper` herein, für Mobilgeräte wird
aber nicht gebaut. Dass ein Treffer wirklich fehlschlägt, prüfen die Tests des Skripts mit gestellten Listen
(`--liste DATEI`): `python3 -m unittest discover -s tools -p 'test_check_network_free.py'`. Dazu prüft
`crates/app/tests/csp.rs` (`issue_157_ak2_…`) die Content-Security-Policy in `crates/app/tauri.conf.json`: `connect-src`
enthält nur `'self'` und die IPC-Quellen `ipc:` und `http://ipc.localhost`, keine externen Adressen.

**Code intern englisch (#165):** `tools/check-english.sh` (`make english`, Job `english` in `test-quality.yml`) meldet
deutsche Kommentare und Testnamen mit Datei und Zeile; seit #168 ist die Übergangsliste leer und die Prüfung läuft
auch in `make test-quiet` (`--gate`). Regeln, Ausnahmen und Übergangsliste in
[`ENTWICKLUNG.md`](ENTWICKLUNG.md#sprache-im-code). Die Tests des Skripts (Treffer, kein Treffer, Ausnahmen,
Übergangsliste) laufen mit gestellten Quelldateien in einem temporären Git-Repo:
`python3 -m unittest discover -s tools -p 'test_check_english.py'`.

**Mehrsprachigkeit (#170):** `tools/check_ui_texts.py` (`make ui-texts`, in `make test-quiet` und im Job `english`)
meldet deutsche Texte in `crates/app/ui/*.js`, `index.html` und `crates/app/src/*.rs` ausserhalb der Texttabelle; Tests
des Skripts: `python3 -m unittest discover -s tools -p 'test_check_ui_texts.py'`. Der Rust-Test
`issue_170_ak4_tables_are_complete` vergleicht jede Tabelle in `crates/app/ui/i18n/` mit `de.json`.
`e2e/ui-sprache.spec.js` prüft Sprachwahl, Rückfall auf Deutsch, Schweizer Zahlenformat und gleiche Dateien mit einer
Pseudo-Sprache (alle Texte mit «Ⓔ» markiert); die übrigen Oberflächentests starten ausdrücklich in Deutsch
(`locale: "de-CH"`, `helpers.open` prüft `lang="de-CH"`).

**Prüfung der Übersetzungen (#180):** `tools/check_i18n.py` (`make i18n-check`, in `make test-quiet` und im Job
`english`) vergleicht `crates/app/ui/i18n/{en,fr,it}.json` Schlüssel für Schlüssel und `crates/app/ui/hilfe/{en,fr,it}/`
Abschnitt für Abschnitt (zwischen zwei Überschriften) mit dem Deutschen. Prüfungen: `keys` (gleiche Schlüssel),
`terms` (Glossarbegriffe aus [`GLOSSAR.md`](GLOSSAR.md#übersetzungen) in der festgelegten Form, keine verbotene
Variante; gelesen mit `tools/glossary.py`), `values` (Platzhalter, Zahlen, CHF, %; Schweizer Schreibweisen wie
1'000, 1 000, 6,8 und 6.8 gelten als gleich; in der Hilfe muss jede Zahl des deutschen Abschnitts vorkommen),
`leftover` (Text gleich dem deutschen; in der Hilfe eine wörtlich übernommene Zeile), `structure` (Überschriften mit
Anker, Links und Tabellenzeilen je Hilfedatei), `address` (keine Du-Form in FR/IT, keine Anführungszeichen „…“ in
allen Sprachen) und `length` (mehr als 40 % länger als ein deutscher Text ab 20 Zeichen: nur Hinweis, `--hints`
listet sie). Jeder Befund nennt Datei, Schlüssel oder Zeile mit Abschnitt, Prüfung und Grund. Unklare Befunde stehen
mit Begründung in `tools/i18n_check_exceptions.json` (`findings`: Datei, Ort, Prüfung, Gegenstand, Grund;
`identical`: Wörter, die gleich wie im Deutschen bleiben dürfen, z. B. Eigennamen); eine nicht mehr gebrauchte
Ausnahme ist selbst ein Befund. Tests des Skripts (ein Fall je Prüfung, mit gestellten Dateien):
`python3 -m unittest discover -s tools -p 'test_check_i18n.py'`.

**Platz für die Texte (#182/AK4):** `e2e/ui-textfit.spec.js` öffnet die Oberfläche in DE, EN, FR und IT in der
Fenstergrösse der App (1440 × 920) und prüft mit `e2e/textfit.js` jeden Editor-Abschnitt (auch mit Partner, jeder
Entnahmestrategie und Renditequelle), die Dialoge, die Ergebnisansichten, Liste und Vergleich der Simulationen und
jedes Hilfethema. Befund ist abgeschnittener Text (Element mit `overflow: hidden`/`clip` oder Auslassungspunkten,
Auswahlliste, Knopf oder Eingabefeld, in die der Text nicht passt; der Pfeil einer Auswahlliste zählt 20 px), ein
Rahmen, der seitwärts rollt, und Text, der sich mit anderem Text überdeckt (auch in Grafiken). Der Test nennt je
Ansicht die Befunde mit Element und Text.

**Nächtliche Läufe von Hand starten (#159):** Die nächtlichen Jobs von `test-quality.yml` (`tests-os`, `e2e`,
`release-golden`, `fuzz`, Eigenschaftstests mit 10'000 Fällen) laufen täglich um 02:17 UTC (`17 2 * * *`), nur nach einem
neuen Commit. `mutants-diff` läuft seit #202 nur noch wöchentlich, am Sonntag um 02:17 UTC (eigener Zeitplan
`17 2 * * 0`, die Jobs unterscheiden die beiden an `github.event.schedule`); am Sonntag laufen beide Läufe nebeneinander.
GitHub löst `schedule` bei hoher Last verspätet (mehrere Stunden) oder gar nicht aus. Dann alle nächtlichen Jobs
und `mutants-diff` von Hand starten:

```bash
gh workflow run test-quality.yml --ref main -f nightly=true
```

Ohne `nightly` startet der Lauf von Hand wie bisher ohne `fuzz` und `mutants-diff`. Nächtliche Läufe (auch von Hand mit
`nightly`) und der wöchentliche Lauf haben je eine eigene Concurrency-Gruppe: Ein Push bricht sie nicht ab, sondern nur
einen vorherigen Push-Lauf.

Nicht automatisch getestet: die Fenster- und Dialogschicht der Desktop-App (`crates/app`, nur Weiterleitung an
den Rechenkern) und die Darstellung der Grafiken im Detail (Sichtkontrolle mit Bildschirmfotos).

Alle Dateien in `crates/engine/tests/` sind Module der einen Testdatei `all.rs` (#43); eine neue Datei muss dort
eingetragen werden, sonst schlägt ein Test fehl.

**Temporäre Dateien:** Rust-Tests legen Ordner und Dateien nur über `tempfile` an
(`tempfile::Builder::new().prefix("vivaplan-…-").tempdir()`), nie selbst unter `std::env::temp_dir()`. Der Ordner wird
gelöscht, sobald der zurückgegebene `TempDir` endet, auch wenn der Test fehlschlägt. Die Hilfen `tmp(…)` geben deshalb
das Paar `(TempDir, PathBuf)` zurück; der Test hält den ersten Teil bis zum Schluss (`let (_tmp, dir) = tmp("…");`).
Nextest startet jeden Test in einem eigenen Prozess; Ordner mit der Prozessnummer im Namen blieben sonst bei jedem Lauf
liegen. Ausnahme: Das Fuzz-Ziel `import_legacy` verwendet immer denselben leeren Ordner.

Die Bedienung der Oberfläche prüft Playwright in `e2e/` (176 Tests, Stand 16.11.2026: `ui-*.spec.js` nach Bereichen, gemeinsame Hilfsfunktionen in `helpers.js`,
`optimum.spec.js` und getrennt `ui-cancel.spec.js`, weil der Abbruch für alle laufenden Berechnungen gilt; installiertes Google Chrome,
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
make test-quiet                                  # dasselbe, nur Fehlschläge und Zusammenfassung (#160)
make ui-unit                                     # Hilfsfunktionen der Oberfläche (node --test, ohne Browser)
make clippy                                      # clippy, Warnungen sind Fehler (wie im Workflow, #88)
make network-free                                # keine Netzwerk-Bibliothek in den Abhängigkeiten (#157)
make english                                     # deutsche Kommentare und Testnamen (#165)
tools/ui-smoke.sh                                # Oberfläche (Rauchtest)
(cd e2e && npm install && npx playwright test)   # Oberfläche (Bedienung, 176 Tests)
make coverage                                    # Abdeckung (cargo llvm-cov mit nextest)
python3 tools/slow_tests.py                      # Tests über der Laufzeitgrenze (nach dem Profil ci, #226)
```

Die Rust-Tests laufen mit `make test` (`cargo nextest run --workspace`), nicht mit `cargo test`. Grund: nextest startet
jeden Test in einem eigenen Prozess. Unter `cargo test` laufen die Tests als Threads eines Prozesses und teilen sich den
prozessweiten Fortschritt und Abbruch (`crates/engine/src/progress.rs`); ein Abbrechen-Test kann dann eine gleichzeitig
laufende Berechnung eines anderen Tests abbrechen. Hook, CI und Abdeckung verwenden deshalb ebenfalls nextest.

**Leise Ausgabe und Hook nach dem Commit (#160).** `make test-quiet` führt dieselben Tests wie `make test` aus, gibt aber
nur Fehlschläge, langsame Tests und die Zusammenfassung aus (nextest mit `--status-level fail --final-status-level slow
--hide-progress-bar --cargo-quiet`); so landen in Claude Code wenige Zeilen statt einer je Test im Kontext. Der
Claude-Code-Hook nach einem Commit (`.claude/hooks/after-commit.sh`, Zeitgrenze 15 Minuten in `.claude/settings.json`)
führt die Tests der geänderten Pakete selbst leise aus (Rechenkern oder `Cargo.toml`/`Cargo.lock` geändert: alle Pakete,
weil App und Kommandozeile davon abhängen) und schreibt die volle Ausgabe nach `target/after-commit.log`. Den Agenten
`rust-test-agent` verlangt er nur, wenn Tests fehlschlagen oder der Commit Code in `crates/engine/src` ändert, ohne einen
Test zu ändern oder hinzuzufügen (Datei unter `crates/engine/tests/` oder neues `#[test]`/`proptest!`); sonst meldet er
Claude nur die Zusammenfassung. Jeder Commit wird einmal geprüft. Die Prüfung vor dem Push (`tools/git-hooks/pre-push`,
Abdeckung und Mutanten) ist unverändert. Tests des Hooks: `python3 -m unittest discover -s tools -p 'test_after_commit.py'`
(kleines Git-Repository, `cargo` ersetzt).

**Laufzeitgrenzen (#226).** Langsame Tests verlängern jeden Push-Lauf und vervielfachen sich in den Mutationsläufen.
Jeder Rust-Test bleibt deshalb unter der Grenze seiner Art (Debug-Build):

| Art | Grenze | Zuordnung |
|---|---:|---|
| Unit-Test | 0,1 s | Tests in `src/` (Bibliothek oder Programm) |
| Integrationstest | 1 s | Tests in `tests/` (im Rechenkern das eine Testprogramm `all`) |
| startet ein Programm oder einen Server | 15 s | `crates/cli/tests/cli.rs` (ruft `vivaplan` auf), Tests des Entwicklungsservers (`serve(`) und der Tauri-Testapp (`setup(`, `setup_with(`, `mock_builder(`) |

Braucht ein Test länger, steht direkt über ihm (zwischen den übrigen Kommentar- und Attributzeilen) eine Begründung,
englisch wie alle Kommentare im Code:

```rust
// slow: 600 mutated plans, checks for crashes over many cases
#[test]
fn mutated_plans_never_crash() { … }
```

`tools/slow_tests.py` liest die Laufzeiten aus `target/nextest/ci/junit.xml` (`cargo nextest run --workspace --profile
ci`), ordnet jeden Test einer Art zu und listet jeden Test über der Grenze ohne Begründung mit Name, Dauer und Ort. Der
Befund ist ein Hinweis, kein Fehler: Er steht in der Zusammenfassung von `test-quality.yml` (Job `tests-coverage`) und
auf der Übersichtsseite in der Schicht «Hygiene». Die Laufzeiten in CI stammen aus dem Lauf mit Abdeckung und sind
etwas länger als die eines gewöhnlichen Debug-Builds; lokal hängen sie von der Last ab (alle Tests laufen parallel).
Das Profil `mutants` in `.config/nextest.toml` nimmt die schweren Tests unabhängig davon aus. Tests des Skripts:
`python3 -m unittest discover -s tools -p 'test_slow_tests.py'` (gestellte junit-Dateien).

Die reinen Hilfsfunktionen der Oberfläche (Zahlenformate, Monate, Pfadzugriff, Steuersatz, Dauerschätzung) liegen in
`crates/app/ui/util.js` und haben Unit-Tests in `e2e/unit/util.test.js` (`node --test`, unter 1 s, #87). Die Ergebnisliste
(`crates/app/ui/runs.js`, IndexedDB) prüft `e2e/unit/runs.test.js` gegen echtes IndexedDB-Verhalten mit `fake-indexeddb`
(reine Testabhängigkeit in `e2e/package.json`, #147): Ablegen, Liste, Lesen, Ändern, Löschen, abgebrochene Transaktion,
Rückfall auf den Arbeitsspeicher und ältere Ergebnisse. Die Oberflächentests verwenden die Liste nur im Arbeitsspeicher. Der Job
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
  - Der `rust-test-agent` lässt vor dem Push `cargo mutants --in-diff` auf die geänderten Zeilen laufen. Sein Ziel ist «keine neuen nicht erkannten Mutanten im Rechenkern», nicht nur «80 % erreicht». Er schreibt keine Tests ohne Prüfung und keine überflüssigen (Regel unten).
- **Kein Test ohne neue Prüfung (#222):** Ein neuer Test muss sich seinen Platz verdienen. Vor jedem neuen Test (vom `rust-test-agent` wie von Hand) stehen drei Angaben: welche **Regel** er schützt, welche **realistische Änderung** am Code sie bricht (Off-by-one, vertauschte Schritte, falscher Vergleich, geänderter Vorgabewert), und dass dabei **kein anderer Test** fehlschlägt. Kein eigener Test wird: Code, der sich durch Lesen prüfen lässt (wenige Zeilen, höchstens eine Verzweigung); reine Weiterleitung; Vorgabewert, Konstante oder einzelner Tabelleneintrag; was der Compiler prüft; Verhalten einer fremden Bibliothek; Duplikat; weitere Eingabewerte durch dieselbe Verzweigung (Grenzwerte, die einen anderen Pfad nehmen, zählen nicht dazu). Ausgenommen sind Regressionstests zu Speichern (Planungsdateien, Migration), zu Rechenregeln (Handrechnung, auch mit nur einer Konstante) und Golden Masters sowie benannte Regressionstests zu gefundenen Fehlern, auch Funde der Eigenschaftstests (#223); sie dürfen sich mit anderen Tests überschneiden. Der Bericht des Agenten nennt je neuem Test die geschützte Regel und die Änderung, die ihn fehlschlagen lässt. Die Regel gilt für neue Tests; bestehende werden damit nicht gekürzt. Vorbild: «keep rule» und «cut categories» in `standards/testing.md` von storytold/craft-contrib.
  - **Wöchentlich auf GitHub (#202):** Da das Projekt ohne Pull Requests arbeitet, läuft der Job `mutants-diff` in `test-quality.yml` einmal pro Woche (Sonntag 02:17 UTC, nur nach einem neuen Commit) und von Hand mit `nightly` (#159). Er mutiert nur die geänderten Zeilen des Rechenkerns (`crates/engine`); App und Kommandozeile prüft der volle Lauf von Hand (`mutation-full.yml`, `make mutants`). Verglichen wird mit dem letzten erfolgreichen wöchentlichen Lauf (sonst mit dem Stand vor 7 Tagen), höchstens 7 Tage zurück; die Mutanten sind auf 4 Jobs verteilt (`--shard i/4`, Zeitlimit 6 Stunden je Job) und werden inkrementell übersetzt (`CARGO_INCREMENTAL=1`). Bis #202 lief er nächtlich in 16 Teilen über den ganzen Workspace und brauchte rund 55 % der Actions-Minuten, vor allem für die Vorbereitung jedes Teils (gemessen am 18.11.2026: 3 Tage Änderungen rund 980 Mutanten, rund 75 s je Mutant). Basis und Dateien stehen in der Zusammenfassung des Laufs. Nicht erkannte Mutanten lassen den Lauf fehlschlagen, Zeitüberschreitungen nicht (sie zählen als erkannt, #159). Bei einem Pull Request mutiert der Job wie bisher alle geänderten Rust-Zeilen gegenüber dem Basis-Zweig.
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
  - **Stand nach #187 (21.11.2026, `mutants-diff` des nächtlichen Laufs):** 46 nicht erkannt im Code von #62, #170, #171 und #177. Danach:
    - getestet: Tests mit `issue_187_` im Namen in den Testmodulen von `message.rs`, `series_text.rs`, `model.rs`, `import.rs` und `main.rs` (App), dazu je eine Zeile in `startup_language_uses_the_saved_choice_or_the_system` (App) und `content_types_follow_the_extension` (Kommandozeile);
    - ausgeschlossen: gleichwertige Mutanten in `render` (fehlender Parameter, in Test-Builds `debug_assert!`), `placeholders`, `parts`, `capital_tax`, `i18n::pick` und `language_menu` (Menü nur im Hauptthread), je mit Begründung im Block «#187» in `.cargo/mutants.toml`. Sechs Einträge zu `validate.rs` mit fester Zeilennummer waren verrutscht und sind nachgeführt.

### Übersichtsseite «Testqualität» (#41, #42)

`make report` erzeugt `local/test-quality/report/index.html` aus den vorhandenen Ergebnissen: `target/nextest/ci/junit.xml` (`cargo nextest run --profile ci`), die Abdeckung (`make coverage`, nutzt cargo-llvm-cov), `local/mutants.out/` (`make mutants`), `e2e/results.json` (`npx playwright test`) und über `gh` die Issues. Die Seite funktioniert ohne Netzwerk und wird nicht eingecheckt. Die Oberflächentests laufen in GitHub nächtlich (Job `e2e` in `test-quality.yml`, nur nach einem neuen Commit, auch von Hand startbar; 20 bis 30 Minuten). Das Ergebnis `e2e/results.json` liegt als Artefakt `e2e-results` vor und fliesst in die Übersichtsseite ein; bei einem Fehlschlag liegen Bericht und Traces im Artefakt `e2e-failure`. In GitHub erzeugt `test-quality.yml` (bei jedem Push auf `main`, nächtlich und von Hand) dieselbe Seite im Ordner `report-html/` des Artefakts `test-quality-reports` und schreibt eine Kurzfassung in die Zusammenfassung. **Nach jedem Push lokal:** Lässt der pre-push-Hook einen Push auf `main` durch, startet er im Hintergrund `tools/fetch-report.sh`: Es wartet auf den Lauf von `test-quality.yml` zu diesem Commit, lädt `report-html/` nach `local/test-quality/report/` (der Commit steht in `commit` daneben) und meldet sich unter macOS mit einer Mitteilung. Protokoll `local/test-quality/fetch-report.log`; abschalten mit `FETCH_REPORT=0 git push`; von Hand `tools/fetch-report.sh [<Commit>]`. Die Seite überschreibt eine mit `make report` lokal erzeugte.

Es gibt **keine Gesamtnote**. Jede Schicht hat ihren eigenen Status (erfüllt, Hinweis, nicht erfüllt, keine Daten) nach einer Regel, die auf der Seite steht:

| Schicht | erfüllt | Hinweis | nicht erfüllt |
|---|---|---|---|
| 1 Hygiene | keine Fehlschläge, keine flaky Tests, kein Test über der Laufzeitgrenze ohne Begründung | flaky Tests oder Tests über der Laufzeitgrenze ohne `// slow: <Grund>` (#226) | ein Test schlägt fehl |
| 2 Angemessenheit | alle drei Pakete ≥ 80 %, keine nicht erkannten Mutanten | Abdeckung erreicht, aber nicht erkannte Mutanten oder kein Mutationslauf | ein Paket unter 80 % |
| 3 Anforderungsbezug | jedes Abnahmekriterium durch Test belegt, geprüft oder abgehakt | Kriterien ohne Test, ohne Prüfung und ohne Haken | – |
| 4 Ergebnis | jeder durchgerutschte Fehler nennt den Test, der ihn hätte fangen müssen, und den Regressionstest | eine Antwort fehlt | – |

Fehlende Daten erscheinen als «keine Daten», nicht als Fehler. Die Schichten 3 und 4 lesen die Issues nur beim Erzeugen der Seite über `gh` (nicht die App); ohne Anmeldung sind beide «keine Daten», die übrige Seite bleibt vollständig. Ein «Verlauf» vergleicht Abdeckung je Paket, Zahl der nicht erkannten Mutanten, flaky Tests und die Anteile der Kriterien mit Test und der geprüften Kriterien mit der vorherigen Version (Stand je Version in `local/test-quality/history/`).

Die Werkzeuge haben eigene Tests: `python3 -m unittest discover tools/report` (Statusregeln, Kennzeichnung der Tests, Fundort-Auswertung; läuft auch in `test-quality.yml`).

**Kennzeichnung der Tests (#42):** Ein Test, der ein Abnahmekriterium belegt, nennt Issue und Kriterium im Kommentar oder im Namen: `// #127/AK2` (Zweites Kriterium des Issues #127) oder `fn issue_127_ak2_…`; in den Oberflächentests im Testtitel (`#127/AK2`). Gezählt wird die Reihenfolge der Häkchen-Liste des Issues. Führende Nullen spielen keine Rolle (`#08/AK4` und `#8/AK4` gehören zum Issue #8). Die Zuordnung läuft über die Nummer, nicht über den Titel; alte Kennungen (`FEA-…/AKn`, `fea_…_akn`) zählen nicht mehr (#117). In den Oberflächentests genügt `AK2` im Testtitel, wenn der Test in einem `test.describe` steht, dessen Titel genau eine Issue-Nummer nennt (`test.describe("… (#127)")`); nennt der Titel mehrere Nummern, zählt der Test nicht (#113). Gesucht wird in den Rust-Quellen von Engine, Kommandozeile und App, in `e2e/*.js`, `e2e/unit/*.js`, `fuzz/fuzz_targets/` und in den Tests der Werkzeuge (`tools/**/test_*.py`, nicht in den Werkzeugen selbst). Beispiele in Tests, die keine Belege sind, verwenden die Nummer `#0` oder sind zusammengesetzt (`"#7/" "AK1"`), damit sie nicht zählen.

**Mehrere Kriterien in einer Kennung (#192):** Eine Kennung kann mehrere Kriterien nennen, als Liste oder Bereich: `#98/AK1–AK3` (auch `AK1-AK3`, `AK1 bis AK3`) zählt für AK1, AK2 und AK3; `#76/AK1, AK3` für AK1 und AK3 (nicht AK2); `#85/AK1 und AK2` für beide; im Testnamen `issue_53_ak4_ak5_…` für AK4 und AK5 (in Python auch `test_issue_53_ak4_…`). Das gilt in Kommentaren und Testnamen (Rust, JS, Python) und für `AK3, AK9` im Testtitel innerhalb eines `test.describe("… (#170)")`. Ein rückwärts laufender oder über 50 Kriterien langer Bereich zählt nur für seine beiden Enden.

**Nachgetragene Kriterien und verwaiste Kennungen (#192):** Neben der Häkchen-Liste im Text zählen Kriterien, die ein Kommentar des Issues als `- [ ] AKn …` oder `- [x] AKn …` nachträgt (Beispiel #69/AK8, #170/AK9); ein Kommentar ersetzt kein Kriterium des Textes. Der Abschnitt «3 Anforderungsbezug» listet unter «Verwaiste Kennungen» jede Kennung mit Fundstelle, deren Issue es nicht gibt oder deren Kriterium weder im Text noch in einem Kommentar steht; Issue `#0` (Beispiele) ist ausgenommen.

**Kriterien je Issue (#113):** Der Abschnitt «3 Anforderungsbezug» teilt jedes Kriterium einer von vier Arten zu, in dieser Rangfolge:

| Art | Bedeutung |
|---|---|
| mit Test | ein gekennzeichneter Test belegt das Kriterium |
| geprüft | eine Prüfung auf Anweisung («Prüfe #Nr», siehe `CLAUDE.md`) hält es im Issue als erfüllt fest |
| nur manuell | abgehakt, ohne gekennzeichneten Test und ohne solche Prüfung |
| ohne | weder Test noch Prüfung noch Haken |

Als Prüfung zählen Kommentare, die mit «Prüfung …», «Erneute Prüfung …», «Nacharbeit AKn …» oder «AKn erfüllt in <Commit>» beginnen. Darin gilt ein Kriterium als geprüft bei einer Zeile «AKn erfüllt», «AKn ok» oder «AKn ist belegt» (auch mehrere: «AK1, AK3 ok»), bei «Alle … Kriterien erfüllt» und bei «Ergebnis: x von y Kriterien ok». «AKn nicht …», «offen», «teilweise» oder «überholt» nehmen es wieder heraus. Je Kriterium zählt die letzte Aussage. Eine Abnahme durch den Anwender («Prüfung … durch den Anwender») zählt nicht als Prüfung, sondern bleibt «nur manuell».

**Fundort durchgerutschter Fehler:** Bug-Issues tragen eines der Labels `found: test` (ein Test fand ihn, vor der Abnahme), `found: acceptance` oder `found: usage`. Der Text beantwortet **«Welcher Test hätte ihn fangen müssen?»** und nennt den **Regressionstest** (Vorlage `.github/ISSUE_TEMPLATE/fehler.md`).
