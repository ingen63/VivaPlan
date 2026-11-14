# QA-Bericht und Testmatrix

Stand 7. November 2026. Grundlage: Testplan «VivaPlan» (Haushalt bis Performance). Die bestehende Suite
(siehe `TESTS.md`) wurde gegen den Plan abgeglichen; Lücken sind in `crates/engine/tests/qa_matrix.rs` geschlossen.

## Testmatrix

«vorh.» = schon vor diesem Durchgang abgedeckt (Datei laut `TESTS.md`), «neu» = `qa_matrix.rs`.

| Bereich des Plans | Abdeckung |
|---|---|
| Invarianten: Vermögensgleichung (≤ 0.01 CHF) | vorh. (500 Zufallsplanungen), neu `check_books` in allen Fällen |
| Invariante: Cashflow | vorh. `cash_flows_are_conserved_without_returns` (ohne Renditen); neu `monthly_cash_flow_identity_with_returns_and_taxes` (mit Renditen, Steuern, Einmalbeträgen, Renten; alle vier Strategien, fixe, Bootstrap- und parametrische Pfade; Abweichung ≤ 0.01 CHF) |
| Invariante: Alter | neu: Alter im Monatsdatensatz = Alter-Meilenstein = Ereignismonat |
| Invariante: Reproduzierbarkeit | vorh. (fix), neu Bootstrap und parametrisch mit gleichem/anderem Startwert |
| Haushalt: Einzelperson, Ehepaar, 30/60/100 Jahre | neu |
| Haushalt: ungültige Fälle (Ende vor Start, Geburt in Zukunft, Alter > 120, leere Pflichtfelder) | neu, dabei zwei Fehler gefunden (F1, F2) |
| Meilensteine: Datum/Alter, mehrere, Verschieben der Pensionierung | neu |
| PK: 100 % Rente/Kapital/50-50, sehr klein/gross, Rente = Kapital × Umwandlungssatz | neu |
| Säule 3a: Auszahlung früh/bei Pensionierung/danach | neu; laufende Einzahlung und Nicht-Einzahlung vorh. (`reference.rs`) |
| AHV: fix, mit/ohne Teuerung | neu |
| Einnahmen/Ausgaben: Einmalzahlungen, Randwerte, altersabhängiger Wechsel im exakten Monat | neu |
| Aktienanteil 0–100 %, Reserve 0–60 Monate | neu |
| Strategien: Standard, 3-Topf (Januar), Guardrails, feste Rate 2–10 % | neu (Standard, Januar, Raten), Guardrails vorh. (`household.rs`) |
| Entnahmerate: Basis ohne Immobilien (liquid 1'000'000 + Immobilie 1'000'000 ⇒ 40'000, nicht 80'000), feste Rate und Guardrails | neu `withdrawal_rate_base_excludes_property` |
| Immobilien: Bestand, Kauf, Verkauf, Objektwechsel, Notverkauf | neu + vorh. |
| Hypotheken: fest 2/5/10 J., SARON + Marge, Amortisation, Tragbarkeit | neu (Laufzeiten, SARON), Amortisation/Tragbarkeit vorh. |
| Steuern: Einkommen, Vermögen, Kapital (unter/in/über Stufe), Grundstückgewinn, Ehepaar | neu + vorh. |
| Historisch: 1929, 1973, 2000, 2007/08; alle Startmonate | neu (Renditen werden Monat für Monat nachgerechnet; Anzahl Läufe = mögliche Startmonate) |
| CSV-Import: fehlende Spalten, Datum, leer, doppelt, Lücke, Text | neu; Währung vorh. (`api::currency_issue`) |
| Monte Carlo: Reproduzierbarkeit, NaN/∞, Plausibilität, 1'000 vs. 10'000 Pfade | neu; Blockbildung/Korrelation vorh. (`data.rs`) |
| Szenarien 1D/2D, Konsistenz mit Einzellauf | neu (3×3×3×3 = 81 Kombinationen, 4 Punkte gegen separaten Lauf) |
| Ergebnisse: Perzentile = Pfade, Jahreswerte, Vorschau = Simulation | neu |
| Dateien: Roundtrip, Fehler beim Öffnen | neu; Neu/Öffnen/Speichern unter in der Oberfläche **nicht automatisch** |
| CLI: validate, run, export, import | vorh. (`crates/cli/tests/cli.rs`) |
| UI: Ansichten, Konsolenfehler | vorh. (`tools/ui-smoke.sh`); neu Playwright (`e2e/`): Navigation, Eingaben und Prüfung, Live-Vorschau = Simulation, Meilensteine verschieben, Undo/Redo mit 100 Aktionen, Datei Neu/Öffnen/Speichern/Roundtrip, Ergebnisansichten, Tooltips, Responsiveness (4 Breiten), Dunkelmodus, gespeicherte Simulationen |
| Performance und Stresstest | neu, `cargo nextest run --cargo-profile release -p vivaplan-engine --run-ignored only -E 'test(/^qa_matrix::/)' --no-capture` (nextest wie `make test`, siehe `TESTS.md`) |
| Golden Master (10 Fälle) | neu, `crates/engine/tests/golden.rs` mit Ergebnissen in `crates/engine/tests/golden/` |

## Gefundene Fehler

| Nr. | Schwere | Befund | Stand |
|---|---|---|---|
| F1 | mittel | Geburtsdatum nach dem Simulationsstart (z. B. 2999) wurde nicht beanstandet; das Alter wurde negativ gerechnet. | behoben (`validate.rs`) |
| F2 | mittel | Alter über 120 Jahre am Simulationsende (z. B. Geburt 1850) wurde nicht beanstandet; ein Alter-Meilenstein war nur bis 130 begrenzt. | behoben (Grenze 120 am Simulationsende) |
| F3 | niedrig | Kopfzeile mit 11 Schaltflächen brach erst ab 760 px um: zwischen 761 und rund 1'300 px scrollte die ganze Seite seitwärts (Playwright, Responsiveness). | behoben (`index.html`, `flex-wrap`) |
| F4 | mittel | Einkommenssteuer wird im Dezember des Steuerjahrs bezahlt; nach Schweizer Praxis einmal jährlich veranlagt und im Mai des Folgejahres fällig. Test `income_tax_is_assessed_yearly_and_paid_in_may_of_next_year`. | behoben (`sim.rs`): Veranlagung im Dezember, Zahlung im Mai des Folgejahres, offene Steuer im letzten Planmonat |
| F5 | mittel | Teuerung wirkt als Sprung im Januar (Vorjahresrate); erwartet ist monatliche Anwendung (1 + i)^(1/12). Test `inflation_is_applied_every_month`. | behoben (`sim.rs`): monatlich (1 + i)^(1/12) |
| F6 | mittel | Hypothekarzinsen wurden monatlich bezahlt; nach Praxis fallen Zins und Amortisation quartalsweise an (erste Zahlung bei Start im April im Juni). Tests `mortgage_interest_is_paid_quarterly_first_payment_in_june`, `accrued_interest_is_settled_on_sale_and_at_plan_end`, `living_costs_exclude_rent_and_mortgage_interest`. | behoben (`sim.rs`): Zins läuft monatlich auf, Zahlung am Quartalsende; bei Verkauf und Planende Restzahlung |

AHV-Anpassung nur im Januar: geprüft und in Ordnung (`indexed_ahv_changes_only_in_january`). Wegen F4 bis F6 wurden die zehn Golden Masters neu erzeugt und fünf ältere Tests an die neuen Regeln angepasst (Teuerung monatlich, Steuer im Mai).

Kritische oder hohe Fehler (Rechenfehler, verletzte Bilanz, ungültige Zahlen) wurden in keinem Lauf gefunden.
Die übrigen 9 anfänglichen Fehlschläge waren falsche Annahmen in den neuen Tests (Nebenkosten sind monatlich,
Überschüsse werden angelegt, Monatsformat), keine Programmfehler.

**Folge von F2:** Ein 100-Jahre-Plan ist nur noch für eine Person möglich, die höchstens 20 Jahre vor Start
geboren ist (Alter am Ende ≤ 120). Das entspricht dem Plan («Alter > 120 → Validierungsfehler»).

## Referenzrechnung «Test Retirement.ods»

`crates/engine/tests/golden/Test Retirement.ods` ist die Basisrechnung des Altprogramms als Tabelle (Blatt 1: Rendite 0 %,
Blatt 2: 5 %, PK-Zins 10 %, Blatt 3 = Blatt 1). `crates/engine/tests/ods_reference.rs` liest die berechneten Werte der Tabelle
und rechnet dieselbe Planung im Programm (Zeile r = Monat r − 3 ab Januar 2030; Pensionierung, AHV-Beginn, Amortisation
und Verkauf aus den Ereignissen der Tabelle).

| Prüfung | Ergebnis |
|---|---|
| Blatt 1, Monate 0 bis zur Amortisation (270 Monate) | Vermögen auf den Rappen gleich |
| Blatt 2, Monate 0 bis zur Pensionierung (161 Monate, mit 5 % und halbierter Rendite ab Zeile 153) | auf den Rappen gleich |
| Blatt 2, PK-Kapitalbezug (PK-Zins 10 %) | Tabelle 845'142, Programm 832'147 (−1.5 %): andere Zinskonvention |
| Blatt 1, alle 604 Monate (inkl. Amortisation und Hausverkauf, nach Korrektur T1 bis T4) | Vermögen auf den Rappen gleich |

**Beurteilung der Tabelle.** Die Grundrechnung (Rendite, Sparen, PK-Beiträge, Kapitalbezug 50 %, Rente = Restkapital × 5 % / 12,
AHV, Ausgabenwechsel, Einmalbetrag) war plausibel und stimmt mit dem Programm überein. Vier Fehler der Tabelle sind korrigiert
(Zellformeln und berechnete Werte in allen drei Blättern):

| Nr. | Befund | Korrektur |
|---|---|---|
| T1 | Das Vermögen wurde bei 0 abgeschnitten (`E = IF(F<=0;0;F)`): bei der Amortisation von 500'000 mit 50'623 Vermögen verschwanden 454'029 Schuld; der Hausverkauf brachte nur das Eigenkapital. | `E = F` (Schuld bleibt sichtbar); der Verkauf nach der Amortisation bringt den vollen Marktwert (`T2 + S2`), vor der Amortisation das Eigenkapital. |
| T2 | Dasselbe Haus wurde ein zweites Mal verkauft (Zeile 454, +1'000'000, Miete +650), weil das Flag «Sold» nur bei Gewinn + Amortisation gesetzt wurde; Sonderformel `T+S` nur in Zeile 348. | `Sold` wird bei jedem Verkauf gesetzt; nach dem Verkauf entfällt die Amortisation (`S`); einheitliche Formel in allen Zeilen. |
| T3 | Rendite ab Zeile 153 hart halbiert, Ausgaben nach AHV-Beginn (8350) hart in Zeile 186, Verkaufsschwelle 50'000 in der Formel. | Neue Eingaben `V2` (Faktor Rendite, 0.5), `W2` (Ausgaben ab AHV, 8350), `X2` (Verkaufsschwelle, 50'000); Ereignis «Derisk» in Spalte A (Zeile 153). |
| T4 | Rente = Kapital × `M2` × `N2` / 12 mit dem Kapitalbezugsanteil `M2` als Rentenanteil (nur bei 50 % richtig). | Rente = Kapital × (1 − `M2`) × `N2` / 12. |

Nicht geändert (bewusste Vereinfachungen der Tabelle): PK-Zins und Teuerung nur im Januar (Blatt 2 hat 10 % PK-Zins als Testwert,
BVG-Minimum 1.25 %), keine Steuern, keine Hypothekarzinsen (in den 10'000 enthalten), Alter in Spalte C um einen Monat zu tief.
Die korrigierte Datei wurde mit LibreOffice (headless) neu berechnet: alle rund 6'600 Zahlenzellen je Blatt stimmen mit den in
der Datei gespeicherten Werten überein, keine Fehlerwerte. (Ein erster Wurf ohne die ursprünglichen Namensraum-Deklarationen
zeigte in LibreOffice überall «Err:5xx»; behoben.)

## Golden Masters

Zehn feste Planungen (Einzelperson, Ehepaar, PK-Rente, PK-Kapitalbezug, Immobilie mit Hypothek, historisch ab 1973,
Bootstrap, parametrisch, Szenarien, Notverkauf). Gespeichert sind Kennzahlen, Ereignisse, Perzentile bzw. Szenario-
Ergebnisse und die Monatswerte jedes Januars und Julis sowie des letzten Monats. Zahlen gelten mit 1e-9 relativer
Toleranz. Der Test schlägt bei jeder Abweichung fehl und nennt die Stelle. Eine gewollte Änderung der Rechenregeln
wird bewusst übernommen mit `GOLDEN_UPDATE=1 cargo nextest run -p vivaplan-engine -E 'test(/^golden::/)'`; danach die Änderung der JSON-Dateien in
Git prüfen. Die Ergebnisse sind über mehrere Läufe stabil (parallele Berechnung ändert nichts).

## Messungen (Release-Build, Apple-Rechner des Entwicklers)

| Fall | Zeit |
|---|---|
| parametrisch, 30 Jahre, 1'000 / 10'000 / 50'000 Pfade | 0.07 s / 0.34 s / 1.8 s |
| Bootstrap, 30 Jahre, 1'000 / 10'000 / 50'000 Pfade | 0.01 s / 0.12 s / 0.7 s |
| Stresstest: Ehepaar, 4 Objekte mit je 2 Tranchen, 100 Jahre, 50'000 Pfade | 7.5 s, höchstens 0.8 GB Speicher, Bilanzen der Perzentil-Läufe stimmen |

CPU-Auslastung wurde nicht separat erfasst (rayon nutzt alle Kerne; `time`: 101 s Benutzerzeit auf 37 s Wanduhr
inklusive Kompilieren).

## Offene Punkte

1. **Cashflow-Invariante mit Immobilien, Pensionskasse und 3a:** Die Monatsgleichung mit Renditen ist für Planungen ohne
   diese Bestandteile geprüft. Mit ihnen fehlen im Monatsdatensatz die Einzelposten (PK-Zins, Hypothekarzins,
   Kauf-/Verkaufsvorgänge), die Vermögensgleichung gilt dort aber (`random_plans_keep_books_balanced`).
2. **Oberfläche:** Die Dialoge der Desktop-App (Datei öffnen/speichern über das Betriebssystem, Menüleiste) lassen sich im
   Browser nicht prüfen; dort gelten Dateiauswahl und Download. Die Darstellung der Grafiken wird nur auf Vorhandensein
   und Tooltips geprüft, nicht pixelgenau.
3. **Statistik bei 50'000 Pfaden** wird nur über die Laufzeit-/Absturzprüfung abgedeckt, die Stabilitätsprüfung
   vergleicht 1'000 mit 10'000 Pfaden.
4. Das Geburtsdatum «in der Zukunft» wird relativ zum Simulationsstart geprüft, nicht zum heutigen Datum, damit
   der Rechenkern ohne Uhr deterministisch bleibt.
