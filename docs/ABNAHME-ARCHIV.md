# Abnahmeliste: Archiv

Abgeschlossene Issues aus [ABNAHME.md](ABNAHME.md): geschlossen, alle Kriterien geprüft (`ok`) oder überholt. Spalten und Vorgehen wie dort. Nur gezielt lesen (`grep -n '^### #<Nr> '`).

## Issues

### #1 Steuern 2029→2030 steigen stärker als die Einnahmen (False Positive)

[Issue #1](https://github.com/ingen63/VivaPlan/issues/1) · geschlossen, nicht geplant · Umsetzung: nicht vermerkt

Fehlalarm (`invalid`), keine Kriterien.

### #2 Kauf nach Verkauf einer Immobilie erfolgt erst Jahre später

[Issue #2](https://github.com/ingen63/VivaPlan/issues/2) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/reference.rs:369](../crates/engine/tests/reference.rs#L369), [engine/tests/simulation.rs:961](../crates/engine/tests/simulation.rs#L961)

Keine Kriterien im Issue; der Entscheid zum Verhalten steht im Issue-Text.

### #3 Zeitleiste: Immobilienereignisse beschriften

[Issue #3](https://github.com/ingen63/VivaPlan/issues/3) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #3/AK1 | Mit `examples/Test 1.rplan` hat jedes Kauf-, Verkaufs- und Notverkaufsereignis eine Beschriftung. | ☑ | **gilt**: Die Planung `examples/Test 1.rplan` gibt es nicht mehr (#26); prüfen mit der Beispielplanung `example.vplan` oder einer erfundenen Planung. | [ui-start.spec.js:10](../e2e/ui-start.spec.js#L10) (prüft alle fünf Kriterien, nennt die Kennungen nicht) | 15.11.2026 ok (e2e, grün) |
| #3/AK2 | Keine zwei Beschriftungen überdecken sich. | ☑ | **gilt** | [ui-start.spec.js:10](../e2e/ui-start.spec.js#L10) (prüft alle fünf Kriterien, nennt die Kennungen nicht) | 15.11.2026 ok (e2e, grün) |
| #3/AK3 | Ereignisse und Meilensteine sind ohne Farbunterscheidung auseinanderzuhalten. | ☑ | **gilt** | [ui-start.spec.js:10](../e2e/ui-start.spec.js#L10) (prüft alle fünf Kriterien, nennt die Kennungen nicht) | 15.11.2026 ok (e2e, grün) |
| #3/AK4 | Bei mehr als 15 Ereignissen erscheinen nur Symbole, Text beim Hover. | ☑ | **gilt** | [ui-start.spec.js:10](../e2e/ui-start.spec.js#L10) (prüft alle fünf Kriterien, nennt die Kennungen nicht) | 15.11.2026 ok (e2e, grün) |
| #3/AK5 | Ohne Ergebnis keine Ereignisse. | ☑ | **gilt** | [ui-start.spec.js:10](../e2e/ui-start.spec.js#L10) (prüft alle fünf Kriterien, nennt die Kennungen nicht) | 15.11.2026 ok (e2e, grün) |

### #4 Beim Start die letzte Planung öffnen

[Issue #4](https://github.com/ingen63/VivaPlan/issues/4) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/example_neu.rs:1](../crates/engine/tests/example_neu.rs#L1), [ui-optimierung.spec.js:98](../e2e/ui-optimierung.spec.js#L98) (+3)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #4/AK1 | Planung öffnen, App beenden, neu starten: dieselbe Planung ist geladen, der Dateiname steht im Kopf. | ☑ | **gilt** | [app/main.rs:1964](../crates/app/src/main.rs#L1964), [app/main.rs:2005](../crates/app/src/main.rs#L2005), [ui-optimierung.spec.js:162](../e2e/ui-optimierung.spec.js#L162) | 15.11.2026 ok (Tests grün; echter Neustart vom Anwender geprüft) |
| #4/AK2 | Speichern unter neuem Namen und Neustart: die neue Datei wird geöffnet. | ☑ | **gilt** | [app/main.rs:2020](../crates/app/src/main.rs#L2020), [app/main.rs:2005](../crates/app/src/main.rs#L2005) | 15.11.2026 ok (Tests grün; echter Neustart vom Anwender geprüft) |
| #4/AK3 | Erststart ohne Merkung: Beispielplanung, kein Hinweis. | ☑ | **überholt**: durch #70 und #61: Beim ersten Start erscheinen Haftungsausschluss und Willkommensdialog; ohne gemerkte Datei lädt die Desktop-App `new.vplan`. | [app/main.rs:1919](../crates/app/src/main.rs#L1919), [ui-optimierung.spec.js:162](../e2e/ui-optimierung.spec.js#L162) | 15.11.2026 ok (Tests grün); ohne Hinweis, dazu Willkommensdialog (#70) |
| #4/AK4 | Datei zwischen den Starts gelöscht: Beispielplanung und Hinweis «Zuletzt geöffnete Datei nicht gefunden». | ☑ | **gilt** | [app/main.rs:1977](../crates/app/src/main.rs#L1977), [ui-optimierung.spec.js:205](../e2e/ui-optimierung.spec.js#L205) | 15.11.2026 ok (Tests grün) |
| #4/AK5 | «Neue Planung» ohne Speichern, dann Neustart: die zuletzt gespeicherte Planung wird geöffnet. | ☑ | **gilt** | kein Test (Code: «Neue Planung» ruft `remember_plan` nicht auf) | 15.11.2026 ok (Code gelesen) |
| #4/AK6 | Beenden mit ungespeicherten Änderungen: Dialog mit Speichern, Verwerfen, Abbrechen; Abbrechen lässt die App offen. | ☑ | **gilt** | [app/main.rs:2042](../crates/app/src/main.rs#L2042), [ui-optimierung.spec.js:213](../e2e/ui-optimierung.spec.js#L213) | 15.11.2026 ok (Tests grün) |
| #4/AK7 | Die Merkung enthält nur den Dateipfad und bleibt lokal. | ☑ | **gilt** | [app/main.rs:2005](../crates/app/src/main.rs#L2005) | 15.11.2026 ok (Code gelesen: `lastPlan` ist der Pfad) |
| #4/AK8 | Die Beispielplanung enthält keine persönlichen Daten und ist ohne Fehler simulierbar. | ☑ | **gilt**: Beispielplanung heute `crates/engine/assets/example.vplan`. | [app/main.rs:1094](../crates/app/src/main.rs#L1094), [app/main.rs:1486](../crates/app/src/main.rs#L1486), [app/main.rs:2061](../crates/app/src/main.rs#L2061) | 15.11.2026 ok (Tests grün) |

### #5 Ergebnisse: Reiter «Parameter»

[Issue #5](https://github.com/ingen63/VivaPlan/issues/5) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #5/AK1 | Nach einem Lauf zeigt der Reiter alle Werte des Laufs; danach im Editor eine Zahl ändern: Der Reiter des bereits berechneten Ergebnisses bleibt unverändert. | ☑ | **gilt** | [ui-ergebnisse.spec.js:420](../e2e/ui-ergebnisse.spec.js#L420) (prüft mehrere Kriterien, nennt die Kennungen nicht) | 15.11.2026 ok (e2e grün) |
| #5/AK2 | Ein gespeichertes Ergebnis zeigt nach dem Öffnen dieselbe Zusammenfassung wie beim Berechnen. | ☑ | **gilt** | [ui-ergebnisse.spec.js:465](../e2e/ui-ergebnisse.spec.js#L465) (prüft mehrere Kriterien, nennt die Kennungen nicht) | 15.11.2026 ok (e2e grün; Gleichheit nicht Wert für Wert getestet, Code: gespeichert wird die Planung des Laufs) |
| #5/AK3 | Nichts im Reiter ist editierbar. | ☑ | **gilt** | [ui-ergebnisse.spec.js:420](../e2e/ui-ergebnisse.spec.js#L420) | 15.11.2026 ok (e2e grün) |
| #5/AK4 | Test: je einen Parameter pro Editorabschnitt ändern, neu rechnen: Die beiden Zusammenfassungen unterscheiden sich an dieser Stelle. | ☑ | **gilt** | [ui-ergebnisse.spec.js:420](../e2e/ui-ergebnisse.spec.js#L420) | 15.11.2026 ok (e2e grün) |
| #5/AK5 | Ein altes Ergebnis ohne Parameter zeigt «Nicht verfügbar» und keinen Fehler. | ☑ | **gilt** | [ui-ergebnisse.spec.js:420](../e2e/ui-ergebnisse.spec.js#L420) | 15.11.2026 ok (e2e grün) |

### #6 Guardrails und 3-Topf gemeinsam anwenden

[Issue #6](https://github.com/ingen63/VivaPlan/issues/6) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/src/optimize.rs:1566](../crates/engine/src/optimize.rs#L1566), [engine/tests/buckets_guardrails.rs:1](../crates/engine/tests/buckets_guardrails.rs#L1) (+2)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #6/AK1 | Auf der Karte «3-Topf» kann «Ausgaben ab Pensionierung» zwischen «wie geplant» und «mit Guardrails steuern» gewählt werden; bei «mit Guardrails» erscheinen die Karten «Guardrails» und «Grenzen der Lebenshaltung». | ☑ | **gilt** | [ui-gespeichert.spec.js:282](../e2e/ui-gespeichert.spec.js#L282) | 15.11.2026 ok (Test grün) |
| #6/AK2 | Bei «mit Guardrails» bestimmen Startrate, Teuerung, Leitplanken und Grenzen die Lebenshaltung genau wie bei der Strategie «Guardrails»; bei gleichen Eingaben und gleichem Vermögensverlauf ist die Entnahme in beiden identisch. | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:99](../crates/engine/tests/buckets_guardrails.rs#L99) | 15.11.2026 ok (Test grün) |
| #6/AK3 | Die Guardrails (rein und kombiniert) starten mit der Pensionierung der ersten Person; in der Kombination gemeinsam mit den Töpfen. | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:121](../crates/engine/tests/buckets_guardrails.rs#L121) | 15.11.2026 ok (Test grün) |
| #6/AK4 | Ein für einen Kauf reservierter Verkaufserlös zählt für die Guardrails-Rate (rein und kombiniert) nicht zum Vermögen. | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:156](../crates/engine/tests/buckets_guardrails.rs#L156) | 15.11.2026 ok (Test grün) |
| #6/AK5 | Nach einem Zufluss rechnen die Guardrails (rein und kombiniert) die Entnahme neu; in der Kombination geht das Geld in Topf 3, und die Topfziele bleiben bis zum nächsten Januar. | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:187](../crates/engine/tests/buckets_guardrails.rs#L187) | 15.11.2026 ok (Test grün) |
| #6/AK6 | Netto-Bedarf und Topfziele werden aus der Guardrails-Lebenshaltung berechnet; die Tabelle «3-Topf-Strategie» nennt diese Quelle. | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:228](../crates/engine/tests/buckets_guardrails.rs#L228) | 15.11.2026 ok (Test grün) |
| #6/AK7 | Nach einer Kürzung durch die Leitplanke sinken die Ziele von Topf 1 und 2 in der nächsten Januar-Auffüllung entsprechend. | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:276](../crates/engine/tests/buckets_guardrails.rs#L276) | 15.11.2026 ok (Test grün) |
| #6/AK8 | Die Topf-Regeln (Januar-Auffüllung, A, B, C, Renditekriterium, Notverkauf, Phase 8) gelten unverändert. | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:292](../crates/engine/tests/buckets_guardrails.rs#L292) | 15.11.2026 ok (Test grün) |
| #6/AK9 | Planungen ohne das neue Feld rechnen unverändert, ausser mit der Strategie «Guardrails» bei zwei Personen mit verschiedenem Pensionierungsdatum oder mit reserviertem Verkaufserlös (Golden-Dateien unverändert, keine verwendet Guardrails). | ☑ | **gilt** | [ui-gespeichert.spec.js:282](../e2e/ui-gespeichert.spec.js#L282) | 15.11.2026 ok (Test grün) |
| #6/AK10 | Der Strategiename «3-Topf + Guardrails» erscheint in Ergebnissen, gespeicherten Ergebnissen, Vergleich und Tooltip (#11). | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:347](../crates/engine/tests/buckets_guardrails.rs#L347), [ui-gespeichert.spec.js:282](../e2e/ui-gespeichert.spec.js#L282) | 15.11.2026 ok (Test grün) |
| #6/AK11 | Die Optimierung bietet «3-Topf + Guardrails» als eigene Strategie an (standardmässig nicht angewählt; untere Leitplanke und Anpassung standardmässig fix). | ☑ | **gilt** | [optimize_ranges.rs:177](../crates/engine/tests/optimize_ranges.rs#L177) | 15.11.2026 ok (Test: eigene Strategie, Leitplanke und Anpassung fix; nicht angewählt: ui.spec.js «#6») |
| #6/AK12 | Hilfe, `BESCHREIBUNG.md` und Spezifikation beschreiben die Kombination. | ☑ | **gilt** | kein Test | 15.11.2026 ok (Durchsicht: `BESCHREIBUNG.md:100`, `hilfe/30-entnahme.md:104`; die Spezifikation entfällt mit #99) |

### #8 Jahreswerte: Steuerspalten einklappbar

[Issue #8](https://github.com/ingen63/VivaPlan/issues/8) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui-ergebnisse.spec.js:16](../e2e/ui-ergebnisse.spec.js#L16)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #8/AK1 | Beim ersten Öffnen (ohne gespeicherte Wahl) zeigt die Jahrestabelle keine Spalte mit «Steuer». | ☑ | **gilt** | [ui-ergebnisse.spec.js:16](../e2e/ui-ergebnisse.spec.js#L16) | 15.11.2026 ok |
| #8/AK2 | «Steuern anzeigen» blendet die fünf Spalten mit denselben Werten wie bisher ein, ein zweiter Klick blendet sie aus. | ☑ | **gilt** | [ui-ergebnisse.spec.js:16](../e2e/ui-ergebnisse.spec.js#L16) | 15.11.2026 ok (fünf Spalten, zweiter Klick blendet aus) |
| #8/AK3 | Nach Neustart der App gilt die zuletzt gewählte Einstellung. | ☑ | **gilt** | [ui-ergebnisse.spec.js:400](../e2e/ui-ergebnisse.spec.js#L400) | 15.11.2026 ok (Wahl bleibt nach dem Neuladen) |
| #8/AK4 | Kennzahl «Steuern total» bleibt sichtbar. | ☑ | **gilt** | [ui-einstieg.spec.js:161](../e2e/ui-einstieg.spec.js#L161) | 15.11.2026 ok (Test) |
| #8/AK5 | Die Berechnung und ein allfälliger Export sind unverändert. | ☑ | **gilt** | [ui-einstieg.spec.js:161](../e2e/ui-einstieg.spec.js#L161) | 15.11.2026 ok (Test) |
| #8/AK6 | Das Umschalten verschiebt keine anderen Spalten. | ☑ | **gilt** | [ui-einstieg.spec.js:161](../e2e/ui-einstieg.spec.js#L161) | 15.11.2026 ok (Test) |
| #8/AK7 | Steht die gespeicherte Einstellung nicht zur Verfügung (zum Beispiel Datenverzeichnis nicht lesbar), gilt eingeklappt. | ☑ | **gilt** | [ui-ergebnisse.spec.js:400](../e2e/ui-ergebnisse.spec.js#L400) | 15.11.2026 ok (Speicher nicht lesbar: eingeklappt) |

### #9 3-Topf-Herleitung aufklappbar in den Jahreswerten

[Issue #9](https://github.com/ingen63/VivaPlan/issues/9) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui-ergebnisse.spec.js:375](../e2e/ui-ergebnisse.spec.js#L375)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #9/AK1 | Ein 3-Topf-Lauf hat keine Karte «3-Topf-Strategie, Herleitung …». | ☑ | **gilt** | [ui-ergebnisse.spec.js:362](../e2e/ui-ergebnisse.spec.js#L362) | 15.11.2026 ok (keine Karte) |
| #9/AK2 | In den Jahreswerten lässt sich zu einem Jahr mit Auffüllung die Herleitung aufklappen, mit allen Angaben nach Abschnitt 10. | ☑ | **gilt** | [ui-ergebnisse.spec.js:362](../e2e/ui-ergebnisse.spec.js#L362) | 15.11.2026 ok (Herleitung mit Netto-Bedarf, PK-Rente, Kriterium, Umbuchungen) |
| #9/AK3 | Jahre ohne Auffüllung haben keine Aufklapp-Zeile. | ☑ | **gilt** | [ui-einstieg.spec.js:185](../e2e/ui-einstieg.spec.js#L185) | 15.11.2026 ok (Test) |
| #9/AK4 | Läufe ohne 3-Topf-Strategie zeigen keine Aufklapp-Zeilen. | ☑ | **gilt** | [ui-einstieg.spec.js:185](../e2e/ui-einstieg.spec.js#L185) | 15.11.2026 ok (Test) |
| #9/AK5 | Der UI-Test zur Herleitung (`ui-tests/ui.spec.js:728`), die 3-Topf-Spezifikation (Abschnitt 10), die Hilfe und die Beschreibung sind nachgeführt. | ☑ | **gilt**: Einmalige Nachführung; der Test liegt heute in `e2e/ui-*.spec.js` (#48). | [ui-ergebnisse.spec.js:362](../e2e/ui-ergebnisse.spec.js#L362) | 15.11.2026 ok (Test, Hilfe und Beschreibung vorhanden) |
| #9/AK6 | Beim Wechsel des gewählten Einzellaufs zeigt die Tabelle die Herleitung des neuen Laufs. | ☑ | **gilt** | [ui-einstieg.spec.js:216](../e2e/ui-einstieg.spec.js#L216) | 15.11.2026 ok (Test) |

### #10 Ereignisliste ohne 3-Topf-Meldungen

[Issue #10](https://github.com/ingen63/VivaPlan/issues/10) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui-ergebnisse.spec.js:375](../e2e/ui-ergebnisse.spec.js#L375)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #10/AK1 | Ein 3-Topf-Lauf zeigt in der Karte «Ereignisse» keine Zeile zu Auffüllung, Zufluss oder Reservierung. | ☑ | **gilt** | [ui-ergebnisse.spec.js:362](../e2e/ui-ergebnisse.spec.js#L362) | 15.11.2026 ok (Ereignisliste ohne «Topf 1») |
| #10/AK2 | Pensionierung, Kapitalbezug, Käufe, Verkäufe, Notverkäufe, Defizitphase und Geld aufgebraucht stehen weiterhin in der Liste. | ☑ | **gilt** | [ui-einstieg.spec.js:185](../e2e/ui-einstieg.spec.js#L185) | 15.11.2026 ok (Test) |
| #10/AK3 | Ein Lauf ohne 3-Topf-Strategie zeigt dieselbe Liste wie bisher. | ☑ | **gilt** | [ui-einstieg.spec.js:185](../e2e/ui-einstieg.spec.js#L185) | 15.11.2026 ok (Test) |
| #10/AK4 | Jeder reservierte Verkaufserlös ist in den Jahreswerten unter der Herleitung (#9) zu finden. | ☑ | **gilt** | kein Test | 15.11.2026 ok (Sichtprüfung vom Anwender) |
| #10/AK5 | Zeitleiste und Grafikmarken sind unverändert. | ☑ | **gilt** | kein Test | 15.11.2026 ok (Sichtprüfung vom Anwender) |
| #10/AK6 | Der UI-Test, der 3-Topf-Ereignisse in der Liste erwartet, ist angepasst. | ☑ | **gilt** | [ui-ergebnisse.spec.js:362](../e2e/ui-ergebnisse.spec.js#L362) | 15.11.2026 ok (Test erwartet keine 3-Topf-Ereignisse mehr) |

### #12 Gespeicherte Ergebnisse: Spalte «Zeitreihe» statt «Art»

[Issue #12](https://github.com/ingen63/VivaPlan/issues/12) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #12/AK1 | Liste und Vergleich haben keine Spalte «Art», aber eine Spalte «Zeitreihe». | ☑ | **gilt** | [ui-gespeichert.spec.js:32](../e2e/ui-gespeichert.spec.js#L32) | 15.11.2026 ok (Spalte Zeitreihe, keine Spalte Art) |
| #12/AK2 | Ein historisches oder Bootstrap-Ergebnis zeigt den Reihennamen ohne Pfad und «.csv». | ☑ | **gilt**: in der neuen Fassung (Kommentar in #12 vom 15.11.2026): Die Spalte «Zeitreihe» zeigt den Namen der Anlagestrategie bzw. der Reihe | [ui-einstieg.spec.js:262](../e2e/ui-einstieg.spec.js#L262) | 15.11.2026 ok (Test, in der neuen Fassung vom 15.11.2026) |
| #12/AK3 | Feste und parametrische Ergebnisse zeigen «feste Renditen» bzw. «parametrisch» in Grau. | ☑ | **gilt** | [ui-einstieg.spec.js:262](../e2e/ui-einstieg.spec.js#L262) | 15.11.2026 ok (Test, auch Grau) |
| #12/AK4 | Ein Ergebnis zeigt auch nach Änderung der Zeitreihe in der aktuellen Planung weiterhin die Zeitreihe von damals. | ☑ | **gilt**: Sinngemäss: Änderung der Anlagestrategie statt der Zeitreihe (#69). | [ui-gespeichert.spec.js:32](../e2e/ui-gespeichert.spec.js#L32) | 15.11.2026 ok (Ergebnis behält die Werte von damals) |
| #12/AK5 | Einzellauf, Szenarien und Anzahl Läufe sind weiterhin erkennbar (zweite Zeile in «Ergebnis»). | ☑ | **gilt** | [ui-gespeichert.spec.js:32](../e2e/ui-gespeichert.spec.js#L32) | 15.11.2026 ok («Einzellauf» in der zweiten Zeile) |
| #12/AK6 | Alte Einträge zeigen «–» ohne Fehler. | ☑ | **gilt** | [ui-gespeichert.spec.js:32](../e2e/ui-gespeichert.spec.js#L32) | 15.11.2026 ok (alte Einträge «–») |

### #14 Pflichteingaben prüfen und markieren

[Issue #14](https://github.com/ingen63/VivaPlan/issues/14) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/mandatory_inputs.rs:1](../crates/engine/tests/mandatory_inputs.rs#L1), [ui-start.spec.js:397](../e2e/ui-start.spec.js#L397) (+1)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #14/AK1 | Leerer Name: Fehler am Feld, Start blockiert. | ☑ | **gilt** | [ui-start.spec.js:397](../e2e/ui-start.spec.js#L397) | 15.11.2026 ok |
| #14/AK2 | Leeres oder ungültiges Geburtsdatum: Fehler am Feld, Start blockiert. | ☑ | **gilt** | [ui-start.spec.js:397](../e2e/ui-start.spec.js#L397) | 15.11.2026 ok |
| #14/AK3 | Jede Zeile der Pflichtliste hat einen Test, der den Fehler auslöst und die Meldung prüft. | ☑ | **gilt** | [tests/mandatory_inputs.rs:1](../crates/engine/tests/mandatory_inputs.rs#L1) | 15.11.2026 ok |
| #14/AK4 | Sind alle Pflichtfelder gültig, startet die Simulation wie bisher. | ☑ | **gilt** | [ui-start.spec.js:397](../e2e/ui-start.spec.js#L397) | 15.11.2026 ok |
| #14/AK5 | Beim Blockieren führt der Verweis in den ersten Abschnitt mit Fehler. | ☑ | **gilt** | [ui-start.spec.js:397](../e2e/ui-start.spec.js#L397) | 15.11.2026 ok (Meldung nennt «Haushalt») |
| #14/AK6 | Verkaufsdatum vor dem Simulationsstart: Warnung im Abschnitt Immobilien. | ☑ | **gilt** | [src/validate.rs:731](../crates/engine/src/validate.rs#L731) | 15.11.2026 ok (Warnung «Verkaufszeitpunkt liegt vor dem Simulationsstart» (Test in validate.rs:1295)) |
| #14/AK7 | Die Liste in der Spezifikation stimmt mit den Prüfungen in der Engine überein. | ☑ | **gilt** | [tests/mandatory_inputs.rs:1](../crates/engine/tests/mandatory_inputs.rs#L1) | 15.11.2026 ok (jede Zeile der Liste hat einen Test) |

### #16 «Altersguthaben heute» → «per Simulationsstart»

[Issue #16](https://github.com/ingen63/VivaPlan/issues/16) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui-start.spec.js:74](../e2e/ui-start.spec.js#L74)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #16/AK1 | Bei Start 11.2026 heisst das Feld «Altersguthaben per Simulationsstart (11.2026)»; nach Änderung des Starts auf 03.2027 ändert sich der Text sofort. | ☑ | **gilt** | [ui-start.spec.js:74](../e2e/ui-start.spec.js#L74) | 15.11.2026 ok |
| #16/AK2 | Die Bezeichnung gilt für beide Personen. | ☑ | **gilt** | [ui-start.spec.js:74](../e2e/ui-start.spec.js#L74) | 15.11.2026 ok (zwei Personen) |
| #16/AK3 | «Altersguthaben heute» kommt in Editor, Hilfe und Beschreibung nicht mehr vor. | ☑ | **gilt** | [ui-start.spec.js:74](../e2e/ui-start.spec.js#L74) | 15.11.2026 ok (alte Bezeichnungen in Editor, Hilfe und Beschreibung nicht mehr gefunden (grep)) |
| #16/AK4 | Oberflächentests sind nachgeführt. | ☑ | **gilt** | [ui-start.spec.js:74](../e2e/ui-start.spec.js#L74) | 15.11.2026 ok |

### #17 «Bestände heute» → «per Simulationsstart»

[Issue #17](https://github.com/ingen63/VivaPlan/issues/17) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #17/AK1 | Karte und Einleitung tragen die neue Bezeichnung mit dem aktuellen Start. | ☑ | **gilt** | [ui-start.spec.js:74](../e2e/ui-start.spec.js#L74) | 15.11.2026 ok |
| #17/AK2 | Die Felder 3a-Guthaben und Marktwert tragen die neue Bezeichnung. | ☑ | **gilt** | [ui-start.spec.js:74](../e2e/ui-start.spec.js#L74) | 15.11.2026 ok |
| #17/AK3 | «Bestände heute», «Aktuelle Bestände», «Guthaben heute» und «Marktwert heute» kommen in Editor, Hilfe und Beschreibung nicht mehr vor. | ☑ | **gilt** | [ui-start.spec.js:74](../e2e/ui-start.spec.js#L74) | 15.11.2026 ok (alte Bezeichnungen nicht mehr gefunden (grep)) |
| #17/AK4 | Oberflächentests sind nachgeführt. | ☑ | **gilt** | [ui-start.spec.js:74](../e2e/ui-start.spec.js#L74) | 15.11.2026 ok |

### #19 Zeitreihen-Erklärungen in ein Info-Popup

[Issue #19](https://github.com/ingen63/VivaPlan/issues/19) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui-start.spec.js:98](../e2e/ui-start.spec.js#L98)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #19/AK1 | Die Karte «Zeitreihe» enthält unter der Tabelle keine festen Erklärungstexte mehr. | ☑ | **überholt**: durch #69: Die Karte «Zeitreihe» gibt es nicht mehr; die Erklärung steht im Info-Popup der Anlagestrategie. | kein Test | 15.11.2026 überholt, nicht geprüft |
| #19/AK2 | Maus auf ⓘ öffnet das Popup mit dem bisherigen Text; Tastaturfokus und Klick öffnen es ebenfalls, Esc schliesst es. | ☑ | **gilt**: Heute für das Info-Popup der Anlagestrategie (Test #19/#69). | [ui-start.spec.js:98](../e2e/ui-start.spec.js#L98) | 15.11.2026 ok |
| #19/AK3 | Das Popup bleibt offen, wenn die Maus hineinbewegt wird. | ☑ | **gilt**: Heute für das Info-Popup der Anlagestrategie. | [ui-start.spec.js:98](../e2e/ui-start.spec.js#L98) | 15.11.2026 ok |
| #19/AK4 | Die Info zur gewählten Reihe bleibt sichtbar. | ☑ | **überholt**: durch den Kommentar im Issue (Info in der Tabellenzeile) und danach #69 (keine Tabelle der Reihen mehr). | kein Test | 15.11.2026 überholt, nicht geprüft |
| #19/AK5 | Der Ordnerpfad lässt sich aus dem Popup kopieren. | ☑ | **überholt**: durch #69: Seit #69 gibt es keine eigenen Dateien und keinen Ordnerpfad mehr (Kommentar in #19 vom 15.11.2026) | kein Test | 15.11.2026 überholt, nicht geprüft |
| #19/AK6 | Der Inhalt der Texte ist unverändert. | ☑ | **überholt**: durch den Kommentar im Issue (Texte gekürzt) und #69 (Texte zur Anlagestrategie). | kein Test | 15.11.2026 überholt, nicht geprüft |
| #19/K1 (Kommentar) | Unter der Tabelle steht keine Zeile «Gewählt:» mehr; ein Fehler zur Reihe wird weiterhin angezeigt. | ☑ | **überholt**: durch #69: keine Tabelle der Reihen mehr. | kein Test | – |
| #19/K2 (Kommentar) | Jede Reihe hat ein ⓘ; Maus, Tastaturfokus und Klick öffnen das Popup, Esc schliesst es. | ☑ | **überholt**: durch #69: keine Tabelle der Reihen mehr; das Popup gibt es je Anlagestrategie. | kein Test | – |
| #19/K3 (Kommentar) | Jeder Popup-Text hat höchstens 2 Zeilen. | ☑ | **unklar**: Gilt die Grenze von 2 Zeilen auch für die Texte der Anlagestrategie? | kein Test | – |
| #19/K4 (Kommentar) | Die Texte der vier mitgelieferten Reihen entsprechen dem Vorschlag oder der abgestimmten Fassung. | ☑ | **überholt**: durch #69 und #90: Die vier Reihen sind nicht mehr einzeln wählbar, Pictet ist entfernt. | kein Test | – |
| #19/K5 (Kommentar) | Die vollständigen Quellenangaben (mit Stand der Daten) bleiben in den Dateien erhalten. | ☑ | **gilt** | kein Test | – |
| #19/K6 (Kommentar) | Test und Hilfe nachgeführt. | ☑ | **gilt** | kein Test | – |

### #21 Variationen ohne feste Obergrenze, Warnung ab 60 s

[Issue #21](https://github.com/ingen63/VivaPlan/issues/21) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui-ergebnisse.spec.js:542](../e2e/ui-ergebnisse.spec.js#L542)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #21/AK1 | Mit 1'001 Kombinationen (bisher abgelehnt) wird die Berechnung angeboten. | ☑ | **gilt** | [ui-ergebnisse.spec.js:542](../e2e/ui-ergebnisse.spec.js#L542) | 15.11.2026 ok |
| #21/AK2 | Bei geschätzten 90 s erscheint die Warnung mit Dauer und Läufen; «Abbrechen» startet nichts, «Trotzdem starten» startet. | ☑ | **gilt** | [ui-ergebnisse.spec.js:542](../e2e/ui-ergebnisse.spec.js#L542) | 15.11.2026 ok |
| #21/AK3 | Bei geschätzten 30 s keine Warnung. | ☑ | **gilt** | [ui-ergebnisse.spec.js:542](../e2e/ui-ergebnisse.spec.js#L542) | 15.11.2026 ok |
| #21/AK4 | Die Zusammenfassung der Variationen nennt keine Grenze mehr ausser der technischen. | ☑ | **gilt** | [ui-ergebnisse.spec.js:542](../e2e/ui-ergebnisse.spec.js#L542) | 15.11.2026 ok |
| #21/AK5 | 1'000'001 Kombinationen werden mit der technischen Meldung abgelehnt, ohne die App zu belasten. | ☑ | **gilt** | [ui-ergebnisse.spec.js:542](../e2e/ui-ergebnisse.spec.js#L542) | 15.11.2026 ok |
| #21/AK6 | Abbrechen bleibt während der Berechnung möglich. | ☑ | **gilt** | [ui-cancel.spec.js:41](../e2e/ui-cancel.spec.js#L41) | 15.11.2026 ok (Abbrechen läuft (ui-cancel.spec.js:41)) |

### #22 Legenden: gestrichelte Linien

[Issue #22](https://github.com/ingen63/VivaPlan/issues/22) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui-ergebnisse.spec.js:625](../e2e/ui-ergebnisse.spec.js#L625)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #22/AK1 | In «Bandbreite des Gesamtvermögens» zeigt die Legende «1. und 99. Perzentil» als gestricheltes Linien-Symbol in der Farbe des Diagramms. | ☑ | **gilt** | [ui-ergebnisse.spec.js:625](../e2e/ui-ergebnisse.spec.js#L625) | 15.11.2026 ok |
| #22/AK2 | Die Legende der Verteilung des Endvermögens nennt die fünf Perzentil-Linien mit gestricheltem Symbol. | ☑ | **gilt** | [ui-ergebnisse.spec.js:625](../e2e/ui-ergebnisse.spec.js#L625) | 15.11.2026 ok |
| #22/AK3 | Übersicht und Grafiken zeigen dieselben Legenden. | ☑ | **gilt** | [ui-ergebnisse.spec.js:625](../e2e/ui-ergebnisse.spec.js#L625) | 15.11.2026 ok |
| #22/AK4 | Jede Linie in den beiden Diagrammen hat einen Legendeneintrag und umgekehrt. | ☑ | **gilt** | [ui-ergebnisse.spec.js:625](../e2e/ui-ergebnisse.spec.js#L625) | 15.11.2026 ok (fünf gestrichelte Linien, Legende dazu) |
| #22/AK5 | Die Legenden sind im hellen und dunklen Erscheinungsbild lesbar. | ☑ | **gilt** | kein Test | 15.11.2026 ok (Sichtprüfung vom Anwender, hell und dunkel) |

### #23 Übersicht: Immobilienverkäufe statt Notverkäufe und Einmalzahlungen

[Issue #23](https://github.com/ingen63/VivaPlan/issues/23) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui-ergebnisse.spec.js:113](../e2e/ui-ergebnisse.spec.js#L113), [ui-ergebnisse.spec.js:510](../e2e/ui-ergebnisse.spec.js#L510)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #23/AK1 | Die Übersicht hat keine Karte und keine Spalte «Einmalzahlungen». | ☑ | **gilt** | [ui-ergebnisse.spec.js:510](../e2e/ui-ergebnisse.spec.js#L510) | 15.11.2026 ok |
| #23/AK2 | Die Spalte heisst «Immobilienverkäufe» und zeigt pro Verkauf eine Zeile «Haus: 05.2034». | ☑ | **gilt** | [ui-ergebnisse.spec.js:510](../e2e/ui-ergebnisse.spec.js#L510) | 15.11.2026 ok |
| #23/AK3 | Ein Notverkauf trägt das Warnsymbol, ein geplanter Verkauf nicht. | ☑ | **gilt** | [ui-ergebnisse.spec.js:510](../e2e/ui-ergebnisse.spec.js#L510) | 15.11.2026 ok |
| #23/AK4 | Ein Lauf mit zwei Verkäufen zeigt zwei Zeilen. | ☑ | **gilt** | [ui-ergebnisse.spec.js:510](../e2e/ui-ergebnisse.spec.js#L510) | 15.11.2026 ok |
| #23/AK5 | Die Kennzahlkarte «Notverkäufe» zeigt die Zahlen wie bisher. | ☑ | **gilt** | [ui-abnahme.spec.js:20](../e2e/ui-abnahme.spec.js#L20) | 15.11.2026 ok (Test) |
| #23/AK6 | Planungen ohne Immobilien zeigen die Spalte nicht. | ☑ | **gilt** | [ui-ergebnisse.spec.js:510](../e2e/ui-ergebnisse.spec.js#L510) | 15.11.2026 ok |
| #23/AK7 | Zahlen und Daten stimmen mit den Immobilienereignissen des jeweiligen Perzentil-Laufs überein. | ☑ | **gilt** | [ui-ergebnisse.spec.js:510](../e2e/ui-ergebnisse.spec.js#L510) | 15.11.2026 ok |

### #24 Berechnete Variationen in «Einzelne Läufe» und «Jahreswerte» wählbar

[Issue #24](https://github.com/ingen63/VivaPlan/issues/24) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/rerun_paths.rs:1](../crates/engine/tests/rerun_paths.rs#L1), [ui-optimierung.spec.js:264](../e2e/ui-optimierung.spec.js#L264)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #24/AK1 | Ein Monte-Carlo-Ergebnis mit zwei Variationen zeigt in «Einzelne Läufe» und «Jahreswerte» zwei Auswahlfelder mit den berechneten Stufen. | ☑ | **gilt** | [ui-optimierung.spec.js:265](../e2e/ui-optimierung.spec.js#L265) | 15.11.2026 ok |
| #24/AK2 | Die Auswahl «Aktienanteil 30 %, Kapitalbezug 60 %» zeigt den 50.-Perzentil-Lauf dieser Stufe; sein Endvermögen stimmt mit dem Wert in der Szenarientabelle überein. | ☑ | **gilt** | [ui-optimierung.spec.js:265](../e2e/ui-optimierung.spec.js#L265) | 15.11.2026 ok |
| #24/AK3 | Der Wechsel dauert wenige Sekunden und löst keinen vollständigen Durchlauf aus. | ☑ | **gilt** | [ui-optimierung.spec.js:265](../e2e/ui-optimierung.spec.js#L265) | 15.11.2026 ok (nur «rerun_paths», kein vollständiger Lauf) |
| #24/AK4 | Die Auswahl wirkt in beiden Reitern; die anderen Reiter zeigen Planwerte mit Vermerk. | ☑ | **gilt** | [ui-optimierung.spec.js:265](../e2e/ui-optimierung.spec.js#L265) | 15.11.2026 ok |
| #24/AK5 | Bei gespeicherten Ergebnissen funktioniert die Auswahl; bei abweichendem Endvermögen erscheint der Hinweis. | ☑ | **gilt** | [ui-abnahme.spec.js:31](../e2e/ui-abnahme.spec.js#L31) | 15.11.2026 ok (Test: Hinweis bei abweichendem Endvermögen; gespeicherte Ergebnisse laufen über dieselbe Funktion) |
| #24/AK6 | Der Einzellauf mit Variationen funktioniert wie bisher. | ☑ | **gilt** | [ui-abnahme.spec.js:31](../e2e/ui-abnahme.spec.js#L31) | 15.11.2026 ok (Test) |

### #26 Repository öffentlich machen (Daten- und Plandateien bereinigen)

[Issue #26](https://github.com/ingen63/VivaPlan/issues/26) · geschlossen · Umsetzung: nicht vermerkt

Keine Kriterien im Issue; die Aufgaben stehen in den Kommentaren.

### #33 Mixed findings

[Issue #33](https://github.com/ingen63/VivaPlan/issues/33) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [optimum.spec.js:50](../e2e/optimum.spec.js#L50), [optimum.spec.js:55](../e2e/optimum.spec.js#L55) (+1)

Sammelbefund ohne Kriterien; die Punkte sind in #37 aufgenommen (dort AK3).

### #45 Verwaltung, nicht öffentlich

[Issue #45](https://github.com/ingen63/VivaPlan/issues/45) · geschlossen, nicht geplant · Umsetzung: nicht vermerkt

Platzhalter «Verwaltung, nicht öffentlich», kein Inhalt.

### #47 Fixe Rate, VPW und Endowment ab erster Pensionierung

[Issue #47](https://github.com/ingen63/VivaPlan/issues/47) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/buckets_guardrails.rs:1](../crates/engine/tests/buckets_guardrails.rs#L1), [engine/tests/buckets_guardrails.rs:413](../crates/engine/tests/buckets_guardrails.rs#L413)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #47/AK1 | Bei zwei Personen mit verschiedenem Pensionierungsdatum legen «Fixe Rate», «VPW» und «Endowment» die jährliche Entnahme im Monat der ersten Pensionierung fest (Rate × liquides Vermögen; bei VPW mit dem Alter der Hauptperson in diesem Monat). Ab diesem Monat gilt die Lebenshaltung der Strategie. | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:416](../crates/engine/tests/buckets_guardrails.rs#L416) | 15.11.2026 ok (Test grün) |
| #47/AK2 | Ein noch laufendes Erwerbseinkommen der zweiten Person zählt zu «Einkommen und Renten»: Lebenshaltung = Entnahme + Einkommen und Renten − Wohnkosten, wie bei Guardrails. | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:416](../crates/engine/tests/buckets_guardrails.rs#L416) | 15.11.2026 ok (Test grün) |
| #47/AK3 | Die Unter- und die Obergrenze der Lebenshaltung gelten ab demselben Monat. | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:416](../crates/engine/tests/buckets_guardrails.rs#L416) | 15.11.2026 ok (Test grün) |
| #47/AK4 | Bei einer Person und bei zwei Personen mit gleichem Pensionierungsdatum ändert sich nichts (Regressionstest, Golden-Dateien unverändert). | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:452](../crates/engine/tests/buckets_guardrails.rs#L452) | 15.11.2026 ok (Test grün) |
| #47/AK5 | Die Hilfe (Absatz «Wann gilt die Strategie?» und die Stellen der drei Strategien) und `BESCHREIBUNG.md` nennen den neuen Start. | ☑ | **gilt** | kein Test (Datei geprüft: `crates/app/ui/hilfe/30-entnahme.md`) | 15.11.2026 ok («Wann gilt die Strategie?») |

### #51 Meilensteine AHV-Bezug und Bezug Pensionskasse: immer vorhanden, nicht löschbar, Grenzen und Standard 65

[Issue #51](https://github.com/ingen63/VivaPlan/issues/51) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/src/optimize.rs:1180](../crates/engine/src/optimize.rs#L1180), [engine/src/resolve.rs:185](../crates/engine/src/resolve.rs#L185) (+7)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #51/AK1 | Jede Person hat die Meilensteine «AHV-Bezug» und «Bezug Pensionskasse». Beide lassen sich in der Oberfläche nicht löschen (keine Löschen-Schaltfläche, Hinweis warum). | ☑ | **gilt** | [engine/src/upgrade.rs:496](../crates/engine/src/upgrade.rs#L496), [engine/tests/milestones_limits.rs:108](../crates/engine/tests/milestones_limits.rs#L108) (+2) | 15.11.2026 ok (Test grün) |
| #51/AK2 | Beide lassen sich ändern (Alter oder Datum, wie die übrigen Meilensteine). Das Alter muss innerhalb der Grenzen liegen: AHV-Bezug 63 bis 70 Jahre, Bezug Pensionskasse 58 bis 65 Jahre. Die Grenzen selbst sind gültig (63 und 70, 58 und 65), ein Wert ausserhalb wird mit klarer Meldung abgelehnt (Oberfläche und Validierung im Rechenkern). | ☑ | **gilt**: Die Grenzen sind seit #25 einstellbar; 63 bis 70 und 58 bis 65 sind die Standardwerte. | [engine/tests/milestones_limits.rs:59](../crates/engine/tests/milestones_limits.rs#L59), [ui-eingaben.spec.js:126](../e2e/ui-eingaben.spec.js#L126) (+1) | 15.11.2026 ok (Test grün) |
| #51/AK3 | Neue Planung: Beide Meilensteine stehen auf 65 Jahren. | ☑ | **gilt** | [ui-abnahme.spec.js](../e2e/ui-abnahme.spec.js) (#51/AK3) | 15.11.2026 ok |
| #51/AK4 | Beispiel: Geburt 1980-06. AHV-Bezug mit 65 liegt im Monat 2045-06, mit 63 im Monat 2043-06, mit 70 im Monat 2050-06. Bezug Pensionskasse mit 58 liegt im Monat 2038-06. | ☑ | **gilt** | [engine/tests/milestones_limits.rs:95](../crates/engine/tests/milestones_limits.rs#L95) | 15.11.2026 ok (Test grün) |
| #51/AK5 | Der Meilenstein «Pensionierung» heisst überall «Bezug Pensionskasse» (Oberfläche, Hilfe, Berichte). Die Id `retire` bleibt, damit Planungsdateien gültig bleiben. | ☑ | **gilt** | [engine/src/upgrade.rs:439](../crates/engine/src/upgrade.rs#L439), [ui-optimierung.spec.js:470](../e2e/ui-optimierung.spec.js#L470) | 15.11.2026 ok (Test grün) |
| #51/AK6 | Ältere Planungsdateien ohne einen der beiden Meilensteine erhalten ihn beim Laden mit dem Standard 65. Liegt ein vorhandenes Alter ausserhalb der Grenzen, wird es auf die nächste Grenze gesetzt, mit Hinweis beim Laden (kein stiller Datenverlust). | ☑ | **gilt** | [engine/src/upgrade.rs:439](../crates/engine/src/upgrade.rs#L439), [engine/src/upgrade.rs:459](../crates/engine/src/upgrade.rs#L459) (+2) | 15.11.2026 ok (Test grün) |
| #51/AK7 | Bis #25 umgesetzt ist, gelten die Grenzen fest. Die Stelle, die später die Einstellung liest, ist an einer Stelle gebündelt. | ☑ | **überholt**: durch #25: Die Grenzen kommen aus den Einstellungen. | kein Test | 15.11.2026 überholt, nicht geprüft |
| #51/AK8 | Prüfung (Warnung, keine Sperre): Beträgt das Erwerbseinkommen im Monat des Bezugs der Pensionskasse oder danach mehr als 0, meldet die Prüfung eine Warnung mit dem Betrag, z. B. «Das Erwerbseinkommen beträgt nach dem Bezug der Pensionskasse (06.2040) noch 12'500 pro Monat.» Bei Einkommen 0 ab dem Bezug erscheint keine Warnung. Die Warnung steht in der Prüfung des Rechenkerns (`Severity::Warning`) und in der Oberfläche. | ☑ | **gilt** | [engine/tests/milestones_limits.rs:123](../crates/engine/tests/milestones_limits.rs#L123), [ui-optimierung.spec.js:489](../e2e/ui-optimierung.spec.js#L489) | 15.11.2026 ok (Test grün) |
| #51/AK9 | Beispiel mit erfundenen Zahlen: Einkommen 12'500 ab 2035-02 ohne spätere Zeile mit 0, Bezug Pensionskasse 2040-06: Es erscheint die Warnung mit 12'500. Mit einer Zeile «ab Bezug Pensionskasse: 0» erscheint keine Warnung. Mit einer Zeile «ab 2040-06: 3'000» lautet die Warnung 3'000. | ☑ | **gilt** | [engine/tests/milestones_limits.rs:123](../crates/engine/tests/milestones_limits.rs#L123) | 15.11.2026 ok (Test grün) |
| #51/AK10 | Hilfe, `BESCHREIBUNG.md` und Spezifikation sind nachgeführt; Regressionstest im Rechenkern. | ☑ | **gilt** | kein Test | 15.11.2026 ok (Hilfe, Beschreibung und Spezifikation stimmen überein) |

### #53 Lebenshaltungskosten inklusive Wohnkosten, Simulation mit den echten Wohnkosten

[Issue #53](https://github.com/ingen63/VivaPlan/issues/53) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/src/scenarios.rs:257](../crates/engine/src/scenarios.rs#L257), [engine/tests/cash_housing.rs:1](../crates/engine/tests/cash_housing.rs#L1) (+5)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #53/AK1 | Beschriftung und Hilfe der Lebenshaltungskosten sagen «inklusive Miete bzw. Wohnkosten (Zins, Amortisation, Nebenkosten)». | ☑ | **gilt** | [ui-planung.spec.js:92](../e2e/ui-planung.spec.js#L92) | 15.11.2026 ok (Test grün) |
| #53/AK2 | Der Wohnanteil ist die Summe der simulierten Wohnkosten im Startmonat (Miete bzw. Hypothekarzins, Amortisation, Nebenkosten), mit der Teuerung fortgeschrieben. Er wird von jeder Zeile einer zeitabhängigen Lebenshaltung gleich abgezogen. | ☑ | **gilt** | [engine/src/housing.rs:99](../crates/engine/src/housing.rs#L99), [ui-planung.spec.js:92](../e2e/ui-planung.spec.js#L92) | 15.11.2026 ok (Test grün) |
| #53/AK3 | Beispiel mit erfundenen Zahlen: Eingabe 8'000, Wohnkosten im Startmonat 2'000 (Zins 1'000, Amortisation 500, Nebenkosten 500), Teuerung 0. Die Simulation rechnet mit 6'000 Lebenshaltung plus den echten Wohnkosten; solange sich nichts ändert, sind die «Ausgaben total» 8'000. Steigt der Zins bei einer Erneuerung um 300, sind sie 8'300; bei einem Wechsel in eine Miete von 2'500 sind sie 8'500. | ☑ | **gilt** | [engine/src/housing.rs:99](../crates/engine/src/housing.rs#L99), [engine/tests/cash_housing.rs:70](../crates/engine/tests/cash_housing.rs#L70) | 15.11.2026 ok (Test grün) |
| #53/AK4 | Ist der Wohnanteil grösser als eine Zeile der Lebenshaltung, meldet die Prüfung einen Fehler mit beiden Beträgen. | ☑ | **gilt** | [engine/tests/cash_housing.rs:85](../crates/engine/tests/cash_housing.rs#L85), [ui-planung.spec.js:100](../e2e/ui-planung.spec.js#L100) | 15.11.2026 ok (Test grün) |
| #53/AK5 | Ohne Wohnkosten im Startmonat (keine Immobilie, keine Miete) ändert sich nichts. | ☑ | **gilt** | [engine/src/housing.rs:122](../crates/engine/src/housing.rs#L122), [engine/tests/cash_housing.rs:85](../crates/engine/tests/cash_housing.rs#L85) | 15.11.2026 ok (Test grün) |
| #53/AK6 | Die Einzahlung in die Säule 3a (#52) ist nicht in der Eingabe enthalten und kommt wie bisher separat zu den Ausgaben. | ☑ | **gilt** | [engine/tests/cash_housing.rs:108](../crates/engine/tests/cash_housing.rs#L108) | 15.11.2026 ok (Test grün) |
| #53/AK7 | Ältere Planungen (Lebenshaltung bisher ohne Wohnkosten) werden beim Öffnen umgerechnet (Wohnanteil zu jeder Zeile addiert), mit Hinweis, damit sich das Ergebnis nicht ändert. | ☑ | **gilt** | [engine/tests/cash_housing.rs:118](../crates/engine/tests/cash_housing.rs#L118), [ui-planung.spec.js:121](../e2e/ui-planung.spec.js#L121) | 15.11.2026 ok (Test grün) |
| #53/AK8 | Jahreswerte («Ausgaben total», «Abweichung vom Plan»), Netto-Bedarf der 3-Topf-Strategie, Ausgabenregeln (Guardrails & Co.), Grenzen der Lebenshaltung und Ausgaben-Faktor der Variation verwenden die neue Aufteilung. Regressionstest im Rechenkern; Golden-Dateien ändern nur mit Begründung im Issue. Hilfe, `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt** | [engine/tests/cash_housing.rs:158](../crates/engine/tests/cash_housing.rs#L158) | 15.11.2026 ok (Test grün) |

### #55 Säule 3a: Auszahlungsplan mit Datum oder Alter, Guthaben anzeigen, Restbetrag vorabfüllen

[Issue #55](https://github.com/ingen63/VivaPlan/issues/55) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/pillar3a_plan.rs:1](../crates/engine/tests/pillar3a_plan.rs#L1), [engine/tests/pillar3a_plan.rs:119](../crates/engine/tests/pillar3a_plan.rs#L119) (+3)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #55/AK1 | Jede Zeile des Auszahlungsplans hat die Auswahl «Datum» oder «Alter». Beim Wechsel bleibt der Zeitpunkt gleich (wie bei Meilensteinen, #37). Daneben steht der berechnete Monat und das Alter. | ☑ | **gilt** | [ui-planung.spec.js:157](../e2e/ui-planung.spec.js#L157) | 15.11.2026 ok (Test grün) |
| #55/AK2 | Beispiel mit erfundenen Zahlen: Geburt 1980-06, Zeile «Alter 61.5» ergibt 12.2041; als «Datum» umgestellt steht 12.2041 im Feld. | ☑ | **gilt** | [engine/tests/pillar3a_plan.rs:44](../crates/engine/tests/pillar3a_plan.rs#L44), [ui-planung.spec.js:157](../e2e/ui-planung.spec.js#L157) | 15.11.2026 ok (Test grün) |
| #55/AK3 | Die Regel «frühestens 5 Jahre vor dem AHV-Bezug» (#52) gilt auch für eine Zeile nach Alter; die Meldung steht am Feld. | ☑ | **gilt** | [engine/tests/pillar3a_plan.rs:63](../crates/engine/tests/pillar3a_plan.rs#L63), [ui-planung.spec.js:175](../e2e/ui-planung.spec.js#L175) | 15.11.2026 ok (Test grün) |
| #55/AK4 | Jede Zeile zeigt «Guthaben vor Auszahlung» und «Rest danach» aus der Vorschau; unter dem Plan steht das Guthaben, das im AHV-Bezug ausbezahlt wird. Ohne Vorschau (Fehler in der Planung) steht «–». | ☑ | **gilt** | [engine/tests/pillar3a_plan.rs:94](../crates/engine/tests/pillar3a_plan.rs#L94), [ui-planung.spec.js:175](../e2e/ui-planung.spec.js#L175) (+1) | 15.11.2026 ok (Test grün) |
| #55/AK5 | Beispiel: Guthaben 120'000, Zins 0, keine Einzahlung, Zeile 2041 mit 50'000: vor 120'000, Rest 70'000; Rest im AHV-Bezug 70'000. | ☑ | **gilt** | [engine/tests/pillar3a_plan.rs:94](../crates/engine/tests/pillar3a_plan.rs#L94), [ui-planung.spec.js:185](../e2e/ui-planung.spec.js#L185) | 15.11.2026 ok (Test grün) |
| #55/AK6 | «+ Auszahlung» legt die neue Zeile ein Jahr nach der letzten an (ohne Zeile: frühester erlaubter Monat, 5 Jahre vor dem AHV-Bezug, nicht vor dem Start, höchstens im AHV-Bezug). Der Betrag ist das Guthaben laut Vorschau zu diesem Zeitpunkt nach den früheren Auszahlungen, abgerundet auf 1'000. Er bleibt danach frei änderbar. | ☑ | **gilt** | [ui-planung.spec.js:185](../e2e/ui-planung.spec.js#L185) | 15.11.2026 ok (Test grün) |
| #55/AK7 | Beispiel: wie oben; «+ Auszahlung» ergibt eine Zeile 2042 mit 70'000. | ☑ | **gilt** | [ui-planung.spec.js:185](../e2e/ui-planung.spec.js#L185) | 15.11.2026 ok (Test grün) |
| #55/AK8 | Ältere Planungen (Zeile nur mit Datum) bleiben gültig und erscheinen als «Datum». | ☑ | **gilt** | [engine/tests/pillar3a_plan.rs:130](../crates/engine/tests/pillar3a_plan.rs#L130), [ui-planung.spec.js:157](../e2e/ui-planung.spec.js#L157) | 15.11.2026 ok (Test grün) |
| #55/AK9 | Regressionstest im Rechenkern für die Zeile nach Alter; Oberflächentest für Umschalten, Anzeige und Vorabfüllen. Hilfe, `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt** | [tests/pillar3a_plan.rs:46](../crates/engine/tests/pillar3a_plan.rs#L46) | 15.11.2026 ok (Regressionstests und Oberflächentest vorhanden) |

### #56 Ganzes Projekt mit rustfmt formatieren, Formatierung prüfen, Agentenanleitung anpassen

[Issue #56](https://github.com/ingen63/VivaPlan/issues/56) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #56/AK1 | Im Wurzelverzeichnis liegt eine `rustfmt.toml` mit den gewählten Einstellungen (z. B. Zeilenlänge); sie gilt für alle Pakete. | ☑ | **gilt** | kein Test (Datei geprüft: `rustfmt.toml`) | 15.11.2026 ok (max_width = 120) |
| #56/AK2 | `cargo fmt --all --check` meldet im ganzen Workspace keine Abweichung. | ☑ | **gilt** | kein Test | 15.11.2026 ok (cargo fmt --all --check meldet nichts) |
| #56/AK3 | Die Umformatierung ist ein eigener Commit ohne inhaltliche Änderung; alle Rust-Tests (`make test`) laufen danach unverändert grün, die Golden-Dateien bleiben gleich. | ☑ | **gilt** | kein Test | 15.11.2026 vom Anwender geschlossen: nicht mehr belegbar (das öffentliche Repository hat nur einen Commit; die Tests laufen grün) |
| #56/AK4 | Der Commit der Umformatierung steht in `.git-blame-ignore-revs`, damit `git blame` die fachlichen Änderungen zeigt. | ☑ | **überholt**: durch #26: Das öffentliche Repository hat die alte Historie nicht; `.git-blame-ignore-revs` ist entfernt. | kein Test | 15.11.2026 überholt, nicht geprüft |
| #56/AK5 | Ein GitHub-Workflow (oder ein bestehender Job) prüft bei jedem Push `cargo fmt --all --check` und schlägt bei Abweichungen fehl. | ☑ | **gilt** | kein Test (Datei geprüft: `.github/workflows/test-quality.yml`) | 15.11.2026 ok (Job format) |
| #56/AK6 | `CLAUDE.md` beschreibt die neue Regel: Code wird mit `cargo fmt` formatiert, vor dem Commit läuft `cargo fmt --all --check`. | ☑ | **gilt** | kein Test (Datei geprüft: `CLAUDE.md`) | 15.11.2026 ok |
| #56/AK7 | Die Sperre von `cargo fmt` in `.claude/settings.json` ist aufgehoben (bzw. auf das Nötige beschränkt). | ☑ | **gilt** | kein Test | 15.11.2026 ok (Durchsicht: `.claude/settings.json` sperrt `cargo fmt` nicht mehr) |
| #56/AK8 | Die Agentenanleitung `.claude/agents/rust-test-agent.md` und die Aufträge in `.claude/hooks/after-commit.sh` und `scripts/git-hooks/pre-push` sagen nicht mehr «Kein cargo fmt (das Projekt ist nicht rustfmt-formatiert)», sondern verlangen formatierten Code. | ☑ | **gilt**: Der Hook liegt heute unter `tools/git-hooks/pre-push`. | kein Test (Datei geprüft: `.claude/agents/rust-test-agent.md`) | 15.11.2026 ok (verlangt cargo fmt) |
| #56/AK9 | Keine anderen Dokumente (`TESTS.md`, `QA.md`, `README.md`) beschreiben die alte Regel noch. | ☑ | **gilt** | kein Test | 15.11.2026 ok (Durchsicht: `TESTS.md`, `QA.md`, `README.md`, `ENTWICKLUNG.md` nennen keine alte Regel; sie gilt in `CLAUDE.md`) |

### #58 Erwerbseinkommen optional mit der Teuerung

[Issue #58](https://github.com/ingen63/VivaPlan/issues/58) · geschlossen · Umsetzung: `ca1a939` (altes Repository) · Tests zum Issue ohne Kriterium: [engine/tests/income_indexed.rs:1](../crates/engine/tests/income_indexed.rs#L1), [engine/tests/income_indexed.rs:113](../crates/engine/tests/income_indexed.rs#L113) (+4)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #58/AK1 | Beim Erwerbseinkommen jeder Person gibt es eine Auswahl «steigt mit der Teuerung» (ja/nein). Ohne Angabe gilt «nein»; bestehende Planungen rechnen unverändert. | ☑ | **gilt** | [engine/tests/income_indexed.rs:52](../crates/engine/tests/income_indexed.rs#L52), [engine/tests/income_indexed.rs:189](../crates/engine/tests/income_indexed.rs#L189) (+1) | 15.11.2026 ok (Test grün) |
| #58/AK2 | Mit «ja» wird jeder Wert, auch jede Zeile eines zeitabhängigen Einkommens, als Betrag in Preisen des Simulationsstarts verstanden und mit dem Preisindex der Simulation fortgeschrieben (wie die Lebenshaltung). | ☑ | **gilt** | [engine/tests/income_indexed.rs:72](../crates/engine/tests/income_indexed.rs#L72) | 15.11.2026 ok (Test grün) |
| #58/AK3 | Beispiel mit erfundenen Zahlen: Einkommen 8'000 pro Monat, Teuerung 2 % pro Jahr, «ja»: nach 12 Monaten rund 8'160, nach 5 Jahren rund 8'830. Mit «nein» bleiben es 8'000. | ☑ | **gilt** | [engine/tests/income_indexed.rs:52](../crates/engine/tests/income_indexed.rs#L52) | 15.11.2026 ok (Test grün) |
| #58/AK4 | Die Auswahl wirkt in allen Renditequellen (fix, historisch, Monte Carlo) mit dem jeweiligen Preisindex des Pfades. | ☑ | **gilt** | [engine/tests/income_indexed.rs:87](../crates/engine/tests/income_indexed.rs#L87) | 15.11.2026 ok (Test grün) |
| #58/AK5 | Jahreswerte, Grafiken, Einkommenssteuer und die Warnung «Erwerbseinkommen nach dem Bezug der Pensionskasse» (#51) verwenden den angepassten Betrag. | ☑ | **gilt** | [engine/tests/income_indexed.rs:139](../crates/engine/tests/income_indexed.rs#L139) | 15.11.2026 ok (Test grün) |
| #58/AK6 | Regressionstest im Rechenkern; Golden-Dateien ändern nicht (Vorgabe «nein»). Oberflächentest für die Auswahl. Hilfe, `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test | 15.11.2026 ok (Regressionstests und Oberflächentest vorhanden (E2633)) |

### #62 App mehrsprachig: Deutsch, Englisch, Italienisch, Französisch (Schweiz), Zahlen immer im Schweizer Format

[Issue #62](https://github.com/ingen63/VivaPlan/issues/62) · geschlossen · Sammel-Issue, Umsetzung in #165–#176, #179–#182

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #62/AK1 | Beim ersten Start gilt die Sprache des Betriebssystems (de, en, it, fr, unabhängig von der Region); sonst Deutsch. | ☑ | **gilt** | #170/AK2; [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «#172», «#174», «#175» «AK1» | 21.11.2026 ok: Playwright mit Systemsprache en-US, fr-CH, it-CH |
| #62/AK2 | Auswahl «Sprache» in den Einstellungen, lokal gespeichert, wirkt sofort, auch auf die nativen Menüs. | ☑ | **gilt** (geändert per Kommentar vom 19.11.2026: sofort statt nach Neustart) | #170/AK3; [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «#170» | 21.11.2026 ok; natives Menü nur Code und Rauchtest, nicht von Hand |
| #62/AK3 | Alle sichtbaren Texte in der gewählten Sprache; Test auf fehlende Übersetzungen. | ☑ | **gilt** (geändert per Kommentar vom 19.11.2026: Vollständigkeitstest) | #170/AK4; `make i18n-check` (#180); [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) | 21.11.2026 ok: 0 Befunde, Playwright 18 grün |
| #62/AK4 | Meldungen des Rechenkerns in der gewählten Sprache; Berechnung in allen Sprachen identisch. | ☑ | **gilt** (geändert per Kommentar vom 19.11.2026: Golden-Dateien einmalig in #176 neu, Zahlen unverändert) | #171, #176/AK2; Golden-Tests | 21.11.2026 ok: `issue_171_`, `issue_176_` und Golden-Tests grün |
| #62/AK5 | Zahlen, Beträge, Prozente und Daten in allen vier Sprachen im Schweizer Format. | ☑ | **gilt** | #170/AK5; [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) je Sprache | 21.11.2026 ok: Stichproben je Sprache |
| #62/AK6 | Eine Planung lässt sich in jeder Sprache öffnen und unverändert speichern; keine sprachabhängigen Texte in der Datei. | ☑ | **gilt** | #170/AK6 | 21.11.2026 ok: `issue_170_` grün |
| #62/AK7 | Oberflächentests je Sprache, Rust-Tests für übersetzte Meldungen, Doku nachgeführt. | ☑ | **gilt** | [ui-sprache.spec.js](../e2e/ui-sprache.spec.js); Rust `issue_171_`, `issue_176_` | 21.11.2026 ok: README, BESCHREIBUNG «Sprache» (Durchsicht) |

### #67 Eigener Welt-Aktienindex aus den JST-Daten nach BIP gewichtet, Vergleich mit DMS

[Issue #67](https://github.com/ingen63/VivaPlan/issues/67) · geschlossen · Umsetzung: `2ccfc9e` (altes Repository) · Tests zum Issue ohne Kriterium: [engine/tests/jst.rs:1](../crates/engine/tests/jst.rs#L1), [engine/tests/jst.rs:63](../crates/engine/tests/jst.rs#L63) (+1)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #67/AK1 | Der Rechenweg ist beschrieben und aus den Rohdaten nachrechenbar: BIP in USD über den Wechselkurs, Gewicht = BIP-Anteil des Vorjahres, Umrechnung der Länderrenditen in CHF, Gewichte jährlich angepasst. | ☑ | **gilt** | [engine/tests/jst.rs:91](../crates/engine/tests/jst.rs#L91) | 15.11.2026 ok (Test grün) |
| #67/AK2 | Fehlende Länder oder Jahre (zum Beispiel Deutschland 1945 ohne Wechselkurs) sind benannt; die Gewichte der übrigen Länder werden neu auf 100 % gerechnet. | ☑ | **gilt** | [engine/tests/jst.rs:91](../crates/engine/tests/jst.rs#L91) | 15.11.2026 ok (Test grün) |
| #67/AK3 | Ein Test rechnet drei Jahre von Hand nach (Beispielwerte im Test). | ☑ | **gilt** | [engine/tests/jst.rs:61](../crates/engine/tests/jst.rs#L61) | 15.11.2026 ok (Test grün) |
| #67/AK4 | Vergleich mit den DMS-Kennzahlen (Rendite, Schwankung, schlechtestes Jahr) mit einem Absatz Bewertung. Die DMS-Zahlen dienen nur dem Vergleich und werden nicht ausgeliefert. | ☑ | **gilt** | [engine/tests/jst.rs:138](../crates/engine/tests/jst.rs#L138) | 15.11.2026 ok (Test grün) |
| #67/AK5 | Der Entscheid über Japan und Australien ist mit Begründung dokumentiert. | ☑ | **gilt** | kein Test | 15.11.2026 ok (Durchsicht: `docs/Weltindex JST.md`, Abschnitt «Entscheid Japan und Australien») |
| #67/AK6 | Ist die Abweichung zu DMS unerwartet gross, ist die Ursache beschrieben, bevor #68 beginnt. | ☑ | **gilt** | kein Test | 15.11.2026 ok (Durchsicht: dieselbe Datei, «Bewertung» zum Vergleich mit DMS) |

### #68 Zeitreihen «Welt (JST)» und Schweizer Staatsanleihen aus SNB und JST

[Issue #68](https://github.com/ingen63/VivaPlan/issues/68) · geschlossen · Umsetzung: `2ccfc9e` (altes Repository) · Tests zum Issue ohne Kriterium: [engine/tests/jst.rs:1](../crates/engine/tests/jst.rs#L1), [engine/tests/jst.rs:63](../crates/engine/tests/jst.rs#L63)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #68/AK1 | Die Reihe «Welt (JST)» beginnt 1900 und endet mit dem letzten JST-Jahr; Teuerung aus JST bzw. Landesindex (BFS). | ☑ | **gilt** | [engine/tests/jst.rs:200](../crates/engine/tests/jst.rs#L200) | 15.11.2026 ok (Test grün) |
| #68/AK2 | Obligationen: jährlich aus JST vor 1988, monatlich aus der SNB ab 1988; der Übergang ist im Test ohne Sprung geprüft. | ☑ | **gilt** | [engine/tests/jst.rs:170](../crates/engine/tests/jst.rs#L170) | 15.11.2026 ok (Test grün) |
| #68/AK3 | `data/QUELLEN.md`, Dateikopf und Hilfe nennen die Quelle, die Zitate (Jordà, Schularick, Taylor 2017; Jordà et al. 2019), das Abrufdatum und die Lizenz CC BY-NC-SA 4.0. | ☑ | **gilt**: Die Datei liegt heute unter `crates/engine/data/QUELLEN.md`. | [engine/tests/jst.rs:200](../crates/engine/tests/jst.rs#L200) | 15.11.2026 ok (Test grün) |
| #68/AK4 | Die Reihen sind als Datei mit derselben Lizenz gekennzeichnet; in `QUELLEN.md` steht, dass die App kostenlos bleiben muss (nicht kommerziell). | ☑ | **gilt**: Wie AK3. | [engine/tests/jst.rs:200](../crates/engine/tests/jst.rs#L200) | 15.11.2026 ok (Test grün) |
| #68/AK5 | Ein Golden-Test hält die Werte fest. | ☑ | **gilt** | [engine/tests/jst.rs:37](../crates/engine/tests/jst.rs#L37) | 15.11.2026 ok (Test grün) |

### #73 Vermögen & Anlage: Quote von Aktien und Obligationen als Info neben den Beständen

[Issue #73](https://github.com/ingen63/VivaPlan/issues/73) · geschlossen · Umsetzung: `c5167d7` (altes Repository) · Tests zum Issue ohne Kriterium: [ui-optimierung.spec.js:21](../e2e/ui-optimierung.spec.js#L21)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #73/AK1 | Aktien 80 000, Obligationen 20 000, Cash 50 000: neben «Aktien» steht «80 %», neben «Obligationen» «20 %». | ☑ | **gilt** | [ui-optimierung.spec.js:21](../e2e/ui-optimierung.spec.js#L21) | 15.11.2026 ok |
| #73/AK2 | Die Quote wird ganzzahlig gerundet (Aktien 1, Obligationen 2: «33 %» und «67 %»). | ☑ | **gilt** | [ui-optimierung.spec.js:21](../e2e/ui-optimierung.spec.js#L21) | 15.11.2026 ok |
| #73/AK3 | Nach einer Änderung von Aktien oder Obligationen stimmt die Anzeige, sobald der Wert übernommen ist (beim Verlassen des Feldes, wie die übrigen Textfelder nach #57). | ☑ | **gilt** | [ui-optimierung.spec.js:21](../e2e/ui-optimierung.spec.js#L21) | 15.11.2026 ok |
| #73/AK4 | Sind Aktien und Obligationen beide 0 oder leer, erscheint keine Quote (kein «NaN», kein «0 %»). | ☑ | **gilt** | [ui-optimierung.spec.js:21](../e2e/ui-optimierung.spec.js#L21) | 15.11.2026 ok |
| #73/AK5 | Cash allein ändert die Quoten nicht (Beispiel AK1 mit Cash 0 oder 500 000: weiterhin 80 % / 20 %). | ☑ | **gilt** | [ui-optimierung.spec.js:21](../e2e/ui-optimierung.spec.js#L21) | 15.11.2026 ok |
| #73/AK6 | Die Quote wird nicht in der Planungsdatei gespeichert; Rechenergebnisse und Golden-Dateien bleiben unverändert. | ☑ | **gilt** | [ui-optimierung.spec.js:21](../e2e/ui-optimierung.spec.js#L21) | 15.11.2026 ok |
| #73/AK7 | Die Hilfe zu den Beständen erwähnt die angezeigte Quote. | ☑ | **gilt** | kein Test (Datei geprüft: `crates/app/ui/hilfe/10-editor.md`) | 15.11.2026 ok (Hilfe nennt die Quote) |

### #81 Invarianten- und Robustheitstests rechnen nachweislich genug Planungen

[Issue #81](https://github.com/ingen63/VivaPlan/issues/81) · geschlossen · Umsetzung: siehe Kommentar im Issue

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #81/AK1 | `cash_flows_are_conserved_without_returns` und `buying_and_selling_at_book_value_keeps_total` schlagen fehl, wenn weniger als die Hälfte der Planungen gerechnet wurde. | ☑ | **gilt** | [invariants.rs:252](../crates/engine/tests/invariants.rs#L252) | 15.11.2026 ok (Test grün) |
| #81/AK2 | `mutated_plans_never_crash` verlangt einen begründeten Mindestanteil (statt 40 von 600). | ☑ | **gilt** | [robustness.rs](../crates/engine/tests/robustness.rs) | 15.11.2026 ok (Test grün) |
| #81/AK3 | Ist der Mindestanteil heute nicht erreicht, ist der Generator so angepasst, dass er erreicht wird. | ☑ | **gilt** | [robustness.rs](../crates/engine/tests/robustness.rs) | 15.11.2026 ok (Test grün) |

### #82 Golden Masters erkennen Änderungen in jedem Monat

[Issue #82](https://github.com/ingen63/VivaPlan/issues/82) · geschlossen · Umsetzung: siehe Kommentar im Issue

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #82/AK1 | Je Fall wird zusätzlich eine Prüfsumme (oder Summe) über alle Monatswerte gespeichert und verglichen. | ☑ | **gilt** | [golden.rs](../crates/engine/tests/golden.rs) | 15.11.2026 ok (Test grün) |
| #82/AK2 | `golden_cases_exercise_their_feature` prüft das frisch gerechnete Ergebnis. | ☑ | **gilt** | [golden.rs](../crates/engine/tests/golden.rs) | 15.11.2026 ok (Test grün) |
| #82/AK3 | Beispiel: Ein künstlich veränderter Wert im März lässt den Test fehlschlagen. | ☑ | **gilt** | [golden.rs](../crates/engine/tests/golden.rs) | 15.11.2026 ok (Test grün) |

### #83 Tests ohne feste Zeitgrenzen und feste Wartezeiten

[Issue #83](https://github.com/ingen63/VivaPlan/issues/83) · geschlossen · Umsetzung: siehe Kommentar im Issue

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #83/AK1 | Rust-Tests prüfen keine absoluten Laufzeiten mehr, ausser in `#[ignore]`-Messungen; oder die Grenzen sind mit Begründung so gross, dass sie auf einem 2-Kern-Runner unter Abdeckungsmessung halten. | ☑ | **gilt** | kein Test (Durchsicht der Grenzen) | 15.11.2026 ok |
| #83/AK2 | In `e2e/` ersetzt ein Warten auf einen Zustand jedes `waitForTimeout`, wo das möglich ist; die übrigen sind begründet. | ☑ | **gilt** | kein Test (Durchsicht von `e2e/ui-*.spec.js`) | 15.11.2026 ok (fünf begründete Pausen) |

### #84 Testdoku auf den Stand bringen

[Issue #84](https://github.com/ingen63/VivaPlan/issues/84) · geschlossen · Umsetzung: siehe Kommentar im Issue

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #84/AK1 | `README.md`, `docs/TESTS.md`, `docs/QA.md` nennen `make test` (nextest) statt `cargo test`, mit Grund. | ☑ | **gilt** | kein Test (Dokumente geprüft) | 15.11.2026 ok |
| #84/AK2 | Zahl der Oberflächentests, Abdeckung und Schichtentabelle (alle Dateien in `crates/engine/tests/`) stimmen. | ☑ | **gilt** | kein Test (Dokumente geprüft) | 15.11.2026 ok (Zahlen nicht neu gemessen) |
| #84/AK3 | `tools/test-quality.sh` schreibt den Stand nur nach erfolgreichem Lauf (Fehlercode 3 von cargo-mutants = nicht erkannte Mutanten ausgenommen, mit Kommentar). | ☑ | **gilt** | kein Test (Skript gelesen) | 15.11.2026 ok (nicht ausgeführt) |

### #87 Unit-Tests für die Hilfsfunktionen der Oberfläche

[Issue #87](https://github.com/ingen63/VivaPlan/issues/87) · geschlossen · Umsetzung: siehe Kommentar im Issue

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #87/AK1 | Reine Funktionen liegen in einem eigenen Modul. | ☑ | **gilt** | kein Test (Datei `crates/app/ui/util.js`) | 15.11.2026 ok |
| #87/AK2 | `node --test` prüft sie, ohne Browser; Laufzeit unter 10 s. | ☑ | **gilt** | [util.test.js](../e2e/unit/util.test.js) | 15.11.2026 ok (14 Tests in 0.12 s) |
| #87/AK3 | Der Lauf ist in `test-quality.yml` eingebunden. | ☑ | **gilt** | kein Test (Datei geprüft: `test-quality.yml`) | 15.11.2026 ok (GitHub nicht gelaufen) |

### #99 Ordner docs/spezifikation auflösen

[Issue #99](https://github.com/ingen63/VivaPlan/issues/99) · geschlossen · Umsetzung: 9f23216

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #99/AK1 | `Pflichteingaben.md`, `Weltindex JST.md` und `VivaPlan in Claude (MCP).md` liegen unter gleichem Namen in `docs/` (mit `git mv`, damit die Historie folgt). | ☑ | **gilt** | kein Test (`git show --stat 9f23216`: Umbenennung) | 15.11.2026 ok |
| #99/AK2 | Alle Verweise auf die verschobenen Dateien zeigen auf den neuen Pfad (Code, Tests, Hilfe, Doku, `CLAUDE.md`). `git grep "docs/spezifikation"` findet ausserhalb von `docs/ABNAHME.md` nichts mehr. | ☑ | **gilt** | kein Test (`git grep`) | 15.11.2026 ok |
| #99/AK3 | Die vier entbehrlichen Dateien sind gelöscht. | ☑ | **gilt** | kein Test (`git show --stat 9f23216`) | 15.11.2026 ok |
| #99/AK4 | Die Rechenregeln aus `3-topf-strategie-spezifikation.md` sind mit «Rechenregeln» in `docs/ENTWICKLUNG.md` abgeglichen; jede Abweichung ist im Issue aufgeführt und nachgetragen oder begründet verworfen. | ☑ | **gilt** | [engine/tests/buckets_need.rs](../crates/engine/tests/buckets_need.rs) (`issue_99_ak4_…`, `a_simulation_younger_than_12_months_…`) | 15.11.2026 ok (zwei Abweichungen entschieden: der Code gilt, in `ENTWICKLUNG.md` Punkt 8 nachgetragen) |
| #99/AK5 | Vor dem Löschen steht im Issue je Datei der Beschreibungen umgesetzter Issues, wo ihr Inhalt steht oder dass er entfällt. | ☑ | **gilt** | kein Test (Kommentar im Issue) | 15.11.2026 ok |
| #99/AK6 | `docs/spezifikation/` existiert nicht mehr. `CLAUDE.md` und `docs/ENTWICKLUNG.md` nennen den Ordner nicht mehr; der Satz zu `tools/github-issues/create.sh` in `CLAUDE.md` stimmt weiterhin. | ☑ | **gilt** | kein Test (Durchsicht) | 15.11.2026 ok (Satz zu `create.sh` sinngemäss, nicht wortgleich) |
| #99/AK7 | `make test` und `make ui-unit` sind grün; `tools/ui-smoke.sh` meldet keine kaputten Hilfeverweise. | ☑ | **gilt** | `make test`, `make ui-unit`, `tools/ui-smoke.sh` | 15.11.2026 ok (636 Rust, 18 UI-Unit grün; `#helpcheck` 0 Probleme; voller Rauchtest unter Last mit Zeitüberschreitungen, einzeln grün) |

### #103 Renditekriterium der 3-Topf-Strategie bei Simulationen unter 12 Monaten

[Issue #103](https://github.com/ingen63/VivaPlan/issues/103) · geschlossen · Umsetzung: cdbe449, e3a53ba

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #103/AK1 | Die Regel für Simulationen unter 12 Monaten ist entschieden und im Issue als Kommentar mit Begründung festgehalten. | ☑ | **gilt** (Entscheid «letzte 12 Monate mit Vorlauf», Kommentar vom 16.11.2026) | kein Test (Kommentar im Issue) | 16.11.2026 ok |
| #103/AK2 | Ein Test mit Start im Juli und bekannten Monatsrenditen belegt die gewählte Regel; der bisherige Test «seit Januar» ist ersetzt (angepasst im Kommentar vom 16.11.2026). | ☑ | **gilt** | [buckets_need.rs](../crates/engine/tests/buckets_need.rs) `issue_103_ak2_start_in_july_measures_the_last_twelve_months_with_the_pre_run`, [sim_rules.rs](../crates/engine/tests/sim_rules.rs) `issue_103_the_pre_run_supplies_the_months_before_the_start` | 16.11.2026 ok |
| #103/AK3 | `docs/ENTWICKLUNG.md` und die Hilfe zur 3-Topf-Strategie nennen dieselbe Regel. | ☑ | **gilt** | kein Test (Dateien geprüft: `docs/ENTWICKLUNG.md`, `hilfe/30-entnahme.md`) | 16.11.2026 ok |
| #103/AK4 | Der Geltungsbereich der Hochrechnung im Startjahr ist festgelegt, getestet und in `docs/ENTWICKLUNG.md` beschrieben. | ☑ | **gilt** | [buckets_need.rs](../crates/engine/tests/buckets_need.rs) `issue_99_ak4_a_pension_fund_withdrawal_recomputes_the_need_from_the_withdrawal_month` | 16.11.2026 ok |
| #103/AK5 | Ändert sich ein Ergebnis der Golden-Dateien, steht die Begründung im Issue. | ☑ | **gilt** | `golden::golden_masters` (nur 08 geändert, Begründung im Issue) | 16.11.2026 ok |

### #111 Oberflächentests unabhängig von der Rechnerlast

[Issue #111](https://github.com/ingen63/VivaPlan/issues/111) · geschlossen · Umsetzung: a334c91, 200fc64

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #111/AK1 | Der Testzugang `#results/…` rechnet Bootstrap und Parametrisch mit 500 Pfaden und dem Zufallsstartwert 42; zwei Läufe mit derselben Adresse ergeben dieselben Kennzahlen. | ☑ | **gilt** | [ui-abnahme.spec.js](../e2e/ui-abnahme.spec.js) «#111/AK1: …» | 16.11.2026 ok |
| #111/AK2 | Mit dem Testzugang `#nobench` misst die App die Rechenzeit nicht; Anzeige «Dauer wird geschätzt…», keine Rückfrage «Lange Berechnung». | ☑ | **gilt** | [ui-abnahme.spec.js](../e2e/ui-abnahme.spec.js) «#111/AK2: …» (mit Gegenprobe) | 16.11.2026 ok |
| #111/AK3 | `open()` setzt `#nobench` standardmässig; nur Tests, die die echte Messung prüfen, schalten sie ein. | ☑ | **gilt** | kein Test (Durchsicht von `e2e/`) | 16.11.2026 ok |
| #111/AK4 | Kein Test wartet fest (`waitForTimeout`, `setTimeout` in der Seite); #57-Prüfungen mit `page.clock` und Zählung der Anfragen. | ☑ | **gilt** | [ui-abnahme.spec.js](../e2e/ui-abnahme.spec.js) «#57/…», «#18: Cash-Reserve …» | 16.11.2026 ok (Nacharbeit 200fc64) |
| #111/AK5 | Ausser den Abbruchtests hat kein Test einen eigenen Timeout. | ☑ | **gilt** | kein Test (Durchsicht von `e2e/`) | 16.11.2026 ok |
| #111/AK6 | Tests, die auf bestimmte Ereignisse angewiesen sind, finden diese mit 500 Pfaden. | ☑ | **gilt** | ganzer Satz `e2e/` | 16.11.2026 ok (ohne Anpassung) |
| #111/AK7 | Die Oberflächentests laufen auf GitHub dreimal hintereinander grün. | ☑ | **gilt** | Läufe 43850404495, 43851543289, 43852527223 (beim Aufräumen der Runs am 16.11.2026 gelöscht, Ergebnis im Issue) | 16.11.2026 ok |
| #111/AK8 | `docs/TESTS.md` beschreibt die Regel; `#nobench` und `page.clock` mit Zweck. | ☑ | **gilt** | kein Test (Datei geprüft) | 16.11.2026 ok |

### #112 Klick ins Menü geht verloren, wenn gleichzeitig eine Vorschau eintrifft

[Issue #112](https://github.com/ingen63/VivaPlan/issues/112) · geschlossen · Umsetzung: 9383d82

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #112/AK1 | Das Menü wird beim Eintreffen einer Vorschau nicht neu aufgebaut; nachgeführt werden nur Markierung und Zahlen. | ☑ | **gilt** | [ui-abnahme.spec.js](../e2e/ui-abnahme.spec.js) «#112/AK1, AK2: …» | 16.11.2026 ok |
| #112/AK2 | Ein Klick ins Menü mit Vorschau zwischen Drücken und Loslassen öffnet den Abschnitt. | ☑ | **gilt** | [ui-abnahme.spec.js](../e2e/ui-abnahme.spec.js) «#112/AK1, AK2: …» (Gegenprobe mit altem Verhalten rot) | 16.11.2026 ok |
| #112/AK3 | Die Prüfliste wird nur neu aufgebaut, wenn sich ihre Meldungen ändern; ein Klick auf eine Meldung geht nicht verloren. | ☑ | **gilt** | [ui-abnahme.spec.js](../e2e/ui-abnahme.spec.js) «#112/AK3: …» | 16.11.2026 ok |
| #112/AK4 | Zahlen am Menü und Markierung des aktiven Abschnitts stimmen weiterhin. | ☑ | **gilt** | [ui-abnahme.spec.js](../e2e/ui-abnahme.spec.js) «#112/AK4: …», ganzer Satz `e2e/` | 16.11.2026 ok |

### #113 Anforderungsbezug erkennt Prüfungen, describe-Blöcke und Nummern mit Null

[Issue #113](https://github.com/ingen63/VivaPlan/issues/113) · geschlossen · Umsetzung: 3f6b3c7

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #113/AK1 | Kennzeichnungen werden unabhängig von führenden Nullen zugeordnet. | ☑ | **gilt** | [test_report.py](../tools/report/test_report.py) `test_issue_113_ak1_…` | 16.11.2026 ok (auch auf echten Daten: #8 findet seine Tests) |
| #113/AK2 | Ein Test mit `AKn` in einem `describe`-Block mit Issue-ID belegt das Kriterium n. | ☑ | **gilt** | [test_report.py](../tools/report/test_report.py) `test_issue_113_ak2_…` | 16.11.2026 ok |
| #113/AK3 | Ein in einer Prüfung als erfüllt festgehaltenes Kriterium zählt in der Spalte «geprüft». | ☑ | **gilt** | [test_report.py](../tools/report/test_report.py) `test_issue_113_ak3_…` (2 Tests) | 16.11.2026 ok (237 Kriterien «geprüft») |
| #113/AK4 | Spalten «mit Test», «geprüft», «nur manuell», «ohne»; Aufklappliste mit Art und Datum der Prüfung. | ☑ | **gilt** | kein Test (Seite aus `report.py` durchgesehen) | 16.11.2026 ok |
| #113/AK5 | Unittests in `tools/report/test_report.py` decken die drei Fälle ab. | ☑ | **gilt** | [test_report.py](../tools/report/test_report.py) (18 Tests grün) | 16.11.2026 ok |
| #113/AK6 | `docs/TESTS.md` beschreibt die Zuordnung. | ☑ | **gilt** | kein Test (Datei geprüft) | 16.11.2026 ok |

### #114 Bezugslinie (0 bzw. 100) in den Ergebnisgrafiken dicker zeichnen

[Issue #114](https://github.com/ingen63/VivaPlan/issues/114) · geschlossen · Umsetzung: 77a6423

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #114/AK1 | Nulllinie dicker in jeder Ergebnisgrafik mit 0 im Bereich, auch im Vergleich und in der Minigrafik. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#114/AK1, AK3, AK4: …» | 16.11.2026 ok |
| #114/AK2 | Aktien, Obligationen und Inflation: dicke Linie bei 100 (linear, logarithmisch), bei 0 % in «Änderung in %». | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#114/AK2: …» | 16.11.2026 ok |
| #114/AK3 | Ausserhalb des gezeigten Bereichs keine Bezugslinie. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#114/AK1, AK3, AK4: …» | 16.11.2026 ok |
| #114/AK4 | Bezugslinie hinter den Daten, hell und dunkel gut sichtbar. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#114/AK1, AK3, AK4: …» (Reihenfolge) | 16.11.2026 ok (hell und dunkel von Hand in der Vorschau) |
| #114/AK5 | Ein Oberflächentest prüft die dicke Linie. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «Bezugslinie in den Grafiken (#114)» | 16.11.2026 ok |

### #115 Vergleich der Ergebnisse an das Einzelergebnis angleichen

[Issue #115](https://github.com/ingen63/VivaPlan/issues/115) · geschlossen · Umsetzung: f036687, c48e278

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #115/AK1 | Umschalter Nominal / Heutige Kaufkraft für Kennzahlen, Grafiken und Jahreswerte, je Ergebnis mit seiner Inflation. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#115/AK1: …» | 16.11.2026 ok |
| #115/AK2 | Übersicht: Endvermögen nach Perzentil, Wahrscheinlichkeit, Alter bei Erschöpfung, Bandbreite je Ergebnis. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#115/AK2: …» | 16.11.2026 ok |
| #115/AK3 | Verläufe je Ergebnis überlagert (erweitert um «Lauf wählen», Kommentar zu c48e278). | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#115/AK3: …», «#115/AK3, AK4: Lauf wählen …» | 16.11.2026 ok |
| #115/AK4 | Jahreswerte mit einer Spalte je Ergebnis, gleich wie die Jahreswerte des Einzelergebnisses. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#115/AK4: …», «#115/AK3, AK4: Lauf wählen …» | 16.11.2026 ok |
| #115/AK5 | Sonderfälle: Start verschieden, fehlende Grössen mit Vermerk. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#115/AK5: …» | 16.11.2026 ok |
| #115/AK6 | Zoom und Registerkarten wie im Einzelergebnis. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#115/AK6: …» | 16.11.2026 ok |
| #115/AK7 | Hilfe «Ergebnisse vergleichen» nachgeführt; Oberflächentests für Umschalter, Grafiken und Jahreswerte. | ☑ | **gilt** | kein Test (Hilfe durchgesehen), Tests oben | 16.11.2026 ok |

### #117 Issues nach GitHub-Konvention: Nummer statt eigener ID

[Issue #117](https://github.com/ingen63/VivaPlan/issues/117) · geschlossen · Umsetzung: 4bd1a25, 4c29de1, 2eccce0

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #117/AK1 | Kein Titel mit altem Präfix; genau ein Art-Label (`bug`, `enhancement`, `task`). | ☑ | **gilt** | kein Test (Abfrage mit `gh`) | 17.11.2026 ok |
| #117/AK2 | `docs/ISSUE-IDS.md` nennt jede alte ID mit Nummer, doppelte vermerkt. | ☑ | **gilt** | kein Test (Abgleich mit den Titeln vor der Umstellung) | 17.11.2026 ok |
| #117/AK3 | `git grep` findet alte IDs nur in `docs/ISSUE-IDS.md`. | ☑ | **gilt** | kein Test (`git grep`) | 17.11.2026 ok |
| #117/AK4 | Dateinamen ohne alte ID, `tools/github-issues/` entfernt. | ☑ | **gilt** | kein Test (Durchsicht) | 17.11.2026 ok |
| #117/AK5 | Bericht erkennt `#Nr/AKn`, `issue_Nr_akn_…` und `AKn` im `describe`; nicht weniger belegte Kriterien. | ☑ | **gilt** | [test_report.py](../tools/report/test_report.py) «test_issue_117_ak5_…» | 17.11.2026 ok (508 belegt vorher und nachher) |
| #117/AK6 | Dokumente und Vorlagen beschreiben die neue Konvention. | ☑ | **gilt** (Präzisierung 1: kein `Fixes`/`Closes`, Label `implemented`) | kein Test (Durchsicht) | 17.11.2026 ok |
| #117/AK7 | Labels migriert, Anzahl je Label gleich, englische Beschreibungen. | ☑ | **gilt** | kein Test (Abfrage mit `gh`) | 17.11.2026 ok (Abweichung: `task` 6 → 4, damit jedes Issue genau ein Art-Label hat) |
| #117/AK8 | Label `accepted` angelegt, `umgesetzt` gelöscht. | ☑ | **überholt** durch Präzisierung 1: `implemented` statt `accepted` | kein Test (Abfrage mit `gh`) | 17.11.2026 ok (`implemented` ersetzt `umgesetzt`) |
| #117/AK9 | Keine alten Labels mehr im Repo. | ☑ | **gilt** (Präzisierung 2: Labels in Backticks, `gefunden:`) | kein Test (`git grep`) | 17.11.2026 ok |
| #117/AK10 | `docs/ABNAHME.md` nach Nummer, Auswahl-Abfrage funktioniert. | ☑ | **gilt** | kein Test (Abfrage ausgeführt) | 17.11.2026 ok |
| #117/AK11 | Alle Tests grün, `cargo fmt` sauber, Golden-Dateien unverändert. | ☑ | **gilt** | alle Testsuiten | 17.11.2026 ok |
### #118 Vergleich: Kennzahlen nach Spalte sortieren (auf- und absteigend)

[Issue #118](https://github.com/ingen63/VivaPlan/issues/118) · geschlossen · Umsetzung: db2e666

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #118/AK1 | Klick auf den Spaltenkopf sortiert aufsteigend, zweiter Klick absteigend, andere Spalte wieder aufsteigend. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#118/AK1, AK2, AK3: …» | 16.11.2026 ok |
| #118/AK2 | Pfeil ▲/▼ in der sortierten Spalte, ↕ in den übrigen. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#118/AK1, AK2, AK3: …» | 16.11.2026 ok |
| #118/AK3 | Zahlen nach Wert, Daten nach Datum («keine» zuletzt), Texte alphabetisch. | ☑ | **gilt** | [util.test.js](../e2e/unit/util.test.js) «#118/AK3, AK4: …» | 16.11.2026 ok |
| #118/AK4 | Fehlende Werte in beiden Richtungen am Ende. | ☑ | **gilt** | [util.test.js](../e2e/unit/util.test.js) «#118/AK3, AK4: …» | 16.11.2026 ok |
| #118/AK5 | Ohne Klick Reihenfolge der Liste; Sortierung bleibt bei Registerkarte und Kaufkraft; Farben bleiben. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#118/AK5: …» | 16.11.2026 ok |
| #118/AK6 | Spaltenköpfe per Tastatur bedienbar, mit `aria-sort`. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#118/AK6: …» | 16.11.2026 ok |
| #118/AK7 | Hilfe beschreibt die Sortierung. | ☑ | **gilt** | kein Test (Hilfe durchgesehen) | 16.11.2026 ok |

### #120 Vergleich auf 10 Ergebnisse erweitern und begrenzen

[Issue #120](https://github.com/ingen63/VivaPlan/issues/120) · geschlossen · Umsetzung: 2f6e114

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #120/AK1 | «Vergleichen» aktiv bei 2 bis 10, sonst gesperrt; Tooltip nennt die Grenze. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#120/AK1, AK2: …» | 16.11.2026 ok |
| #120/AK2 | Ab dem 11. markierten Ergebnis Hinweis; Markieren und Löschen bleiben möglich. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#120/AK1, AK2: …» | 16.11.2026 ok |
| #120/AK3 | Zehn unterscheidbare Farben, hell und dunkel, überall gleich. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#120/AK3: …» (light, dark) | 16.11.2026 ok (Bildschirmfotos hell und dunkel) |
| #120/AK4 | Mit 10 Ergebnissen lesbar: Tabellen und Balken scrollen waagrecht. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#120/AK4: …» | 16.11.2026 ok (Bildschirmfoto) |
| #120/AK5 | Ein Oberflächentest vergleicht 10 Ergebnisse. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «Vergleich mit bis zu 10 Ergebnissen (#120)» | 16.11.2026 ok |
| #120/AK6 | Hilfe nennt die Grenze von 10. | ☑ | **gilt** | kein Test (Hilfe durchgesehen) | 16.11.2026 ok |

### #121 Wahrscheinlichkeit, dass das Geld reicht: Achse 80–100 %, umschaltbar auf 0–100 %

[Issue #121](https://github.com/ingen63/VivaPlan/issues/121) · geschlossen · Umsetzung: 9b24b38

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #121/AK1 | Standard 80–100 % im Einzelergebnis und im Vergleich. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#121/AK1, AK2: …» | 16.11.2026 ok |
| #121/AK2 | Schalter 80–100 % / 0–100 % gilt für beide Ansichten und bleibt. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#121/AK1, AK2: …» | 16.11.2026 ok |
| #121/AK3 | Unter 80 % abgeschnitten, Hinweis mit Verweis auf 0–100 %. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#121/AK3, AK4: …», «#121/AK3: im Vergleich …» | 16.11.2026 ok |
| #121/AK4 | Tooltip zeigt den echten Wert. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#121/AK3, AK4: …» | 16.11.2026 ok |
| #121/AK5 | Hilfe beschreibt den Schalter. | ☑ | **gilt** | kein Test (Hilfe durchgesehen) | 16.11.2026 ok |

### #122 Vergleich: Aktien, Obligationen und Inflation linear, logarithmisch oder in %

[Issue #122](https://github.com/ingen63/VivaPlan/issues/122) · geschlossen · Umsetzung: 773898d

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #122/AK1 | Dieselben Schalter wie im Einzelergebnis. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#122/AK1, AK3: …» | 16.11.2026 ok |
| #122/AK2 | Änderung in % je Ergebnis wie im Einzelergebnis; Bezugslinie bei 0 bzw. 100. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#122/AK2, AK5, AK6: …» | 16.11.2026 ok |
| #122/AK3 | Logarithmische Achse wie im Einzelergebnis. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#122/AK1, AK3: …» | 16.11.2026 ok |
| #122/AK4 | Wahl gilt im Einzelergebnis und im Vergleich gemeinsam. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#122/AK4: …» | 16.11.2026 ok |
| #122/AK5 | Tooltip je Ergebnis; Kennzeichnung der Linien bleibt. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#122/AK2, AK5, AK6: …» | 16.11.2026 ok (Linienarten im Bildschirmfoto) |
| #122/AK6 | Heutige Kaufkraft gilt in allen Darstellungen. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#122/AK2, AK5, AK6: …» | 16.11.2026 ok |
| #122/AK7 | Hilfe nachgeführt. | ☑ | **gilt** | kein Test (Hilfe durchgesehen) | 16.11.2026 ok |

### #123 Vergleich: Unterschiede in der Planung mit Bezeichnungen statt internen Feldnamen

[Issue #123](https://github.com/ingen63/VivaPlan/issues/123) · geschlossen · Umsetzung: 31c1d0f

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #123/AK1 | Bezeichnung wie im Editor mit Abschnitt, keine internen Feldnamen. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#123/AK1, AK2, AK3: …» | 16.11.2026 ok (Bildschirmfoto) |
| #123/AK2 | Werte wie im Editor (Prozente, Beträge, Auswahltext, zeitabhängige Werte lesbar). | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#123/AK1, AK2, AK3: …», «#123/AK2, AK4: …» | 16.11.2026 ok |
| #123/AK3 | Gruppiert und sortiert nach den Abschnitten des Editors. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#123/AK1, AK2, AK3: …» | 16.11.2026 ok |
| #123/AK4 | Listen mit Namen statt Index. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#123/AK2, AK4: …» | 16.11.2026 ok |
| #123/AK5 | Felder ohne Bezeichnung erscheinen nicht; Test über alle Felder einer Beispielplanung. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#123/AK5: …» | 16.11.2026 ok |
| #123/AK6 | Hilfe nachgeführt. | ☑ | **gilt** | kein Test (Hilfe durchgesehen) | 16.11.2026 ok |

### #124 Nicht erkannte Mutanten aus dem vollen Mutationslauf

[Issue #124](https://github.com/ingen63/VivaPlan/issues/124) · geschlossen · Umsetzung: 181eed4

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #124/AK1 | 3a-Konto der zweiten Person (`account_of`, `try_buy`). | ☑ | **gilt** | [pillar3a.rs](../crates/engine/tests/pillar3a.rs) «issue_124_ak1_indirect_amortization_goes_to_the_second_person» | 17.11.2026 ok |
| #124/AK2 | 3a-Einzahlungen im Ruhestand. | ☑ | **gilt** | [pillar3a.rs](../crates/engine/tests/pillar3a.rs) «issue_124_ak2_…» | 17.11.2026 ok |
| #124/AK3 | Hypothek: Einmalzahlung, Zinsen, `after_sales`, Amortisation beim AHV-Bezug. | ☑ | **gilt** | [simulation.rs](../crates/engine/tests/simulation.rs), [pillar3a.rs](../crates/engine/tests/pillar3a.rs) «issue_124_ak3_…» | 17.11.2026 ok |
| #124/AK4 | `progress::cancel` bricht eine Berechnung ab. | ☑ | **gilt** | [cancel.rs](../crates/engine/tests/cancel.rs) «issue_124_ak4_cancel_stops_every_running_job» | 17.11.2026 ok |
| #124/AK5 | Kalibrierung: `corr`, `fit_moments`, `joint`. | ☑ | **gilt** | [calibrate.rs](../crates/engine/src/calibrate.rs) «issue_124_ak5_…»; `corr` begründet ausgeschlossen (gleichwertig) | 17.11.2026 ok (Abweichung begründet) |
| #124/AK6 | Import alter Dateien: Standardalter 3a (`upgrade.rs`). | ☑ | **gilt** | [upgrade.rs](../crates/engine/src/upgrade.rs) «issue_124_ak6_…»; unerreichbarer Zweig begründet ausgeschlossen | 17.11.2026 ok (Abweichung begründet) |
| #124/AK7 | Prüfung der Planung: Warnungen 3a, `PayoutAt::is_date`. | ☑ | **gilt** | [validate.rs](../crates/engine/src/validate.rs) «issue_124_ak7_…» | 17.11.2026 ok |
| #124/AK8 | Übrige Mutanten getötet oder begründet ausgeschlossen. | ☑ | **gilt** | `issue_124_ak8_…`, `.cargo/mutants.toml`, `docs/TESTS.md` | 17.11.2026 ok |
| #124/AK9 | Voller Lauf ohne unbegründete nicht erkannte Mutanten. | ☑ | **gilt** | kein Test (voller Lauf vom 16.11.2026: 0 nicht erkannt) | 17.11.2026 ok (Lauf nicht wiederholt) |

### #125 Kopfzeile des Ergebnisses mit dem Namen der Berechnung vor der Anzahl Läufe

[Issue #125](https://github.com/ingen63/VivaPlan/issues/125) · geschlossen · Umsetzung: e37f43c

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #125/AK1 | Mehrere Läufe: «<Name> · <Anzahl> Läufe», Name des gespeicherten Ergebnisses. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#125/AK1, AK3: …» | 16.11.2026 ok |
| #125/AK2 | Einzellauf: Name vorne, danach «Einzellauf · …». | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#125/AK2: …» | 16.11.2026 ok |
| #125/AK3 | Gilt für frisch berechnete und aus der Liste geöffnete Ergebnisse. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#125/AK1, AK3: …», «#125/AK3, AK4: …» | 16.11.2026 ok |
| #125/AK4 | Nach dem Umbenennen zeigt die Kopfzeile den neuen Namen. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#125/AK3, AK4: …» | 16.11.2026 ok |
| #125/AK5 | Langer Name gekürzt, Anzahl sichtbar, voller Name im Tooltip. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#125/AK5: …» | 16.11.2026 ok |
| #125/AK6 | Ohne gespeicherten Namen bleibt die Kopfzeile wie heute. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#125/AK6: …» | 16.11.2026 ok |
| #125/AK7 | Oberflächentest für frisches und geöffnetes Ergebnis; Hilfe nachgeführt. | ☑ | **gilt** | Tests oben; Hilfe `@r-overview` durchgesehen | 16.11.2026 ok |

### #126 Vergleich: Endvermögen nach Perzentil in heutiger Kaufkraft zeigt nur sechs Perzentile

[Issue #126](https://github.com/ingen63/VivaPlan/issues/126) · geschlossen · Umsetzung: 23d9807

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #126/AK1 | Der Rechenkern liefert zu jedem Perzentil den realen Endwert (`endTotalReal`), Rust-Test gegen den Lauf des Perzentils. | ☑ | **gilt** | [simulation.rs](../crates/engine/tests/simulation.rs) «issue_126_ak1_…» | 17.11.2026 ok (Test vergleicht die sechs Perzentile mit Detaillauf; die übrigen haben den Wert, entstehen im Code aber im selben Ausdruck) |
| #126/AK2 | Karte «Endvermögen nach Perzentil» zeigt in heutiger Kaufkraft für jedes Perzentil einen Wert. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#126/AK2, #126/AK3, #129/AK2: …» | 17.11.2026 ok |
| #126/AK3 | Die Karte trägt in heutiger Kaufkraft den Hinweis der Karte «Kennzahlen». | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#126/AK2, #126/AK3, #129/AK2: …» | 17.11.2026 ok |
| #126/AK4 | Perzentil-Zeilen der Kennzahlen-Karte unverändert. | ☑ | **gilt** | [golden.rs](../crates/engine/tests/golden.rs); #115/#118/93 in [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) | 17.11.2026 ok (Golden-Dateien nur Zeilen hinzugefügt: 120, keine entfernt) |
| #126/AK5 | Hilfe und Doku nachgeführt. | ☑ | **gilt** | kein Test (Durchsicht `hilfe/50-ergebnisse.md`) | 17.11.2026 ok |

### #128 Vergleich: fehlende Perzentile älterer Ergebnisse ohne Vermerk, Hilfe zur Perzentil-Tabelle zu allgemein

[Issue #128](https://github.com/ingen63/VivaPlan/issues/128) · geschlossen · Umsetzung: 8c3b4de

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #128/AK1 | Vermerk im Vergleich für ältere Ergebnisse ohne reale Werte je Perzentil; Detailläufe weiter mit Wert. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#128/AK1, #128/AK2: …» | 17.11.2026 ok |
| #128/AK2 | Oberflächentest mit Ergebnis ohne reale Werte je Perzentil: Vermerk und Rückfall. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#128/AK1, #128/AK2: …» | 17.11.2026 ok |
| #128/AK3 | Hilfe beschreibt die reale Perzentil-Tabelle nur im Thema «Vergleich». | ☑ | **gilt** | kein Test (Durchsicht `hilfe/50-ergebnisse.md`) | 17.11.2026 ok |
| #128/AK4 | Hinweistext in `app.js` einmal definiert. | ☑ | **gilt** | kein Test (Durchsicht `REAL_PCT_HINT`) | 17.11.2026 ok |

### #129 Tests zum realen Endvermögen je Perzentil schärfen

[Issue #129](https://github.com/ingen63/VivaPlan/issues/129) · geschlossen · Umsetzung: c9ea087, a140f88

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #129/AK1 | e2e-Test zu #126 im Bericht #126/AK2 und #126/AK3 zugeordnet. | ☑ | **gilt** | kein Test (`report.test_marks`) | 17.11.2026 ok |
| #129/AK2 | e2e-Test prüft jede Ergebnisspalte, ein Laden der Ergebnisse. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#126/AK2, #126/AK3, #129/AK2: …» | 17.11.2026 ok |
| #129/AK3 | Rust-Test prüft alle Perzentile gegen ihren Lauf. | ☑ | **gilt** | [simulation.rs](../crates/engine/tests/simulation.rs) «issue_126_ak1_…» | 17.11.2026 ok |
| #129/AK4 | Endlichkeitsprüfungen umfassen `end_total_real`. | ☑ | **gilt** | [robustness.rs](../crates/engine/tests/robustness.rs), [qa_matrix.rs](../crates/engine/tests/qa_matrix.rs) «stress_couple_four_properties_100_years», Fuzz-Ziel `run_plan` | 17.11.2026 ok (Stresstest mit `QA_PATHS=2000`; mit 50'000 Pfaden Abbruch an der Speichergrenze #116, unabhängig) |

### #130 Szenarien ohne reales Endvermögen je Perzentil

[Issue #130](https://github.com/ingen63/VivaPlan/issues/130) · geschlossen · Umsetzung: df40918

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #130/AK1 | Szenario-Perzentile ohne `endTotalReal`, Zusammenfassung mit. | ☑ | **gilt** | [simulation.rs](../crates/engine/tests/simulation.rs) «issue_130_ak1_…» | 17.11.2026 ok |
| #130/AK2 | Szenarientabelle, Optimum und Vergleich zeigen dieselben Werte. | ☑ | **gilt** | [golden.rs](../crates/engine/tests/golden.rs); alle Oberflächentests | 17.11.2026 ok (Golden 09: nur 90 Zeilen `endTotalReal` entfernt) |

### #133 Geldfluss-Erhaltung über alle Bestandteile prüfen

[Issue #133](https://github.com/ingen63/VivaPlan/issues/133) · geschlossen · Umsetzung: b5640b1

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #133/AK1 | Der Monatsdatensatz (oder ein Testmodus des Rechenkerns) weist alle Geldflüsse und Bewertungsänderungen eines Monats einzeln aus: … | ☑ | **gilt** (Entscheid vom 18.11.2026: Umbuchungen und zusammengefasste Positionen angenommen) | kein Test (Durchsicht `sim::Ledger`) | 18.11.2026 ok (Entscheid des Anwenders) |
| #133/AK2 | Ein Eigenschaftstest (proptest) prüft für zufällige Planungen mit Immobilien, Hypotheken, PK, 3a, Renditen ungleich 0 und allen … | ☑ | **gilt** | [golden.rs](../crates/engine/tests/golden.rs), [proptest_plans.rs](../crates/engine/tests/proptest_plans.rs), [qa_matrix.rs](../crates/engine/tests/qa_matrix.rs) | 18.11.2026 ok |
| #133/AK3 | Beispiel: Ein absichtlich doppelt gebuchter Verkaufserlös (Mutant in `sell`) lässt den Test fehlschlagen. | ☑ | **gilt** | Gegenprobe von Hand (Arbeitskopie, `sell` doppelt) | 18.11.2026 ok |
| #133/AK4 | Die Golden-Dateien bleiben unverändert; | ☑ | **gilt** | kein Test (Git-Verlauf Golden) | 18.11.2026 ok |
| #133/AK5 | `docs/TESTS.md` und `docs/QA.md` (Offene Punkte 1) sind nachgeführt. | ☑ | **gilt** | kein Test (Durchsicht) | 18.11.2026 ok |

### #134 Steuer nie negativ und monoton

[Issue #134](https://github.com/ingen63/VivaPlan/issues/134) · geschlossen · Umsetzung: edc1f0c

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #134/AK1 | Eigenschaftstest: Für beliebige Tabellen mit nicht fallender Steuer gilt `tax::tax(t, a + d, m) >= tax::tax(t, a, m) >= 0` für alle a, d … | ☑ | **gilt** | [proptest_tax.rs](../crates/engine/tests/proptest_tax.rs) | 18.11.2026 ok |
| #134/AK2 | Eigenschaftstest: `property_gain_tax_held` ist für jeden Gewinn, jede Besitzdauer (auch unbekannt) und beide Modelle (Zürich, Aargau) ≥ 0. | ☑ | **gilt** | [proptest_tax.rs](../crates/engine/tests/proptest_tax.rs) | 18.11.2026 ok |
| #134/AK3 | Beispiel: Kapitaltabelle [[0, 0.08], [1'000'000, 0.02]], zwei Kapitalbezüge im selben Jahr (200'000 und 300'000): Die Steuer auf keinem … | ☑ | **gilt** (Beispiel per Kommentar vom 17.11.2026 korrigiert: [[0, 0.10], [300'000, 0.01]]) | [household.rs](../crates/engine/tests/household.rs) | 18.11.2026 ok |
| #134/AK4 | Die Prüfung meldet eine Einkommens-, Vermögens- oder Kapitaltabelle, bei der die Steuer zwischen zwei Stufen fällt, mit Feldbezug; | ☑ | **gilt** (Präzisierung: Warnung, kein Fehler) | [proptest_tax.rs](../crates/engine/tests/proptest_tax.rs), [validate_extra.rs](../crates/engine/tests/validate_extra.rs) | 18.11.2026 ok |
| #134/AK5 | Hilfe (`crates/app/ui/hilfe/10-editor.md`, Steuern) nennt die neue Meldung. | ☑ | **gilt** | kein Test (Durchsicht `hilfe/10-editor.md`) | 18.11.2026 ok |

### #135 Absturz bei leerem Batch-Ergebnis nach frühem Abbruch

[Issue #135](https://github.com/ingen63/VivaPlan/issues/135) · geschlossen · Umsetzung: b5a03dc

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #135/AK1 | `summarize` und `Bands::from_columns` liefern mit 0 Läufen ein Ergebnis ohne Panik; | ☑ | **gilt** | [batch.rs](../crates/engine/src/batch.rs) | 18.11.2026 ok |
| #135/AK2 | Ein Test bricht eine Simulation ab, bevor der erste Block rechnet (`cancel_job` vor `run_as`), und erhält die Meldung «abgebrochen» ohne … | ☑ | **gilt** | [cancel.rs](../crates/engine/tests/cancel.rs) | 18.11.2026 ok |
| #135/AK3 | Dasselbe gilt für Szenarien und Optimierung (Test oder begründet im Issue). | ☑ | **gilt** | kein Test (Durchsicht `api.rs`, `optimize.rs`) | 18.11.2026 ok |

### #136 Planung atomar speichern

[Issue #136](https://github.com/ingen63/VivaPlan/issues/136) · geschlossen · Umsetzung: 9153c90

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #136/AK1 | Die Planung wird zuerst in eine temporäre Datei im selben Ordner geschrieben und auf den Datenträger gebracht; | ☑ | **gilt** | [files.rs](../crates/engine/tests/files.rs) | 18.11.2026 ok |
| #136/AK2 | Test: Schlägt das Schreiben fehl (z. | ☑ | **gilt** | [files.rs](../crates/engine/tests/files.rs) | 18.11.2026 ok |
| #136/AK3 | Test: Nach einem Fehler bleibt keine temporäre Datei im Ordner zurück. | ☑ | **gilt** | [files.rs](../crates/engine/tests/files.rs) | 18.11.2026 ok |
| #136/AK4 | In der Desktop-App erscheint bei einem Fehler beim Speichern eine Meldung, und die Planung gilt weiter als ungespeichert (Test in … | ☑ | **gilt** | [main.rs](../crates/app/src/main.rs), [ui-optimierung.spec.js](../e2e/ui-optimierung.spec.js) | 18.11.2026 ok |
| #136/AK5 | Gilt auch für «Speichern unter» und für die Kommandozeile, soweit sie Planungen schreibt. | ☑ | **gilt** | kein Test (Durchsicht `crates/cli/src/main.rs`) | 18.11.2026 ok |

### #137 Fuzzing und Eigenschaftstests für Upgrade und Altimport

[Issue #137](https://github.com/ingen63/VivaPlan/issues/137) · geschlossen · Umsetzung: 0701f24

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #137/AK1 | Neues Fuzz-Ziel `upgrade_plan`: beliebiges JSON → `api::upgrade_plan` → `api::validate`; | ☑ | **gilt** | Fuzz-Ziel `upgrade_plan` (Build) | 18.11.2026 ok |
| #137/AK2 | Neues Fuzz-Ziel `import_legacy`: beliebiger Text → `api::import_legacy`; | ☑ | **gilt** | Fuzz-Ziel `import_legacy` (Build) | 18.11.2026 ok |
| #137/AK3 | Beide Ziele laufen im nächtlichen Fuzz-Job von `test-quality.yml` mit; | ☑ | **gilt** | kein Test (Durchsicht `test-quality.yml`) | 18.11.2026 ok (Schleife über `cargo fuzz list`; neue Ziele laufen ab der nächsten Nacht) |
| #137/AK4 | proptest: Für zufällige Planungen gilt `upgrade_plan(upgrade_plan(x).plan)` ergibt dieselbe Planung und keine Hinweise. | ☑ | **gilt** | [proptest_upgrade.rs](../crates/engine/tests/proptest_upgrade.rs) | 18.11.2026 ok |
| #137/AK5 | Test: Die Beispielplanung und jede Golden-Planung ergeben beim Nachführen keine Hinweise und keine Änderung. | ☑ | **gilt** | [proptest_upgrade.rs](../crates/engine/tests/proptest_upgrade.rs) | 18.11.2026 ok (Abweichung begründet: Golden-Planungen erst ab dem zweiten Nachführen ohne Änderung) |
| #137/AK6 | `docs/TESTS.md` (Abschnitt «Fuzzing» und Schicht 5f) ist nachgeführt. | ☑ | **gilt** | kein Test (Durchsicht) | 18.11.2026 ok |

### #138 Grundstückgewinnsteuer-Vorlagen gegen amtliche Beispiele prüfen

[Issue #138](https://github.com/ingen63/VivaPlan/issues/138) · geschlossen · Umsetzung: feb956c

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #138/AK1 | Die Werte jeder Vorlage sind an einer einzigen Stelle definiert; | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) | 18.11.2026 ok |
| #138/AK2 | Je Vorlage gibt es mindestens drei Referenzfälle mit verschiedenen Besitzdauern, gerechnet mit `tax::property_gain_tax_held`. | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) | 18.11.2026 ok |
| #138/AK3 | Beispiel Zürich: Gewinn 100'000 CHF, 10 volle Jahre Besitz, Steuerfuss-Option aus → Stufensteuer 29'400, Abzug 20 % → 23'520.00 CHF. | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) | 18.11.2026 ok (Abweichung begründet: kein amtlicher Rechner, gegen amtlichen Tarif nachgerechnet) |
| #138/AK4 | Zürich: Fälle für den Zuschlag (unter 1 Jahr: +50 %, unter 2 Jahren: +25 %), die Freigrenze (Gewinn 4'999 → 0) und den höchsten Abzug … | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) «issue_138_ak3_zurich_…» | 18.11.2026 ok |
| #138/AK5 | Aargau: Fälle für 0, 10, 11 und 25 volle Jahre (40 %, 20 %, 19 %, 5 %). | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) | 18.11.2026 ok |
| #138/AK6 | Ein Test prüft, dass die Vorlage, wie sie die Oberfläche übernimmt, dieselben Beträge ergibt (Parität Oberfläche ↔ Rechenkern). | ☑ | **gilt** | [ui-gespeichert.spec.js](../e2e/ui-gespeichert.spec.js) | 18.11.2026 ok |

### #139 Speichergrenze bei Monte Carlo

[Issue #139](https://github.com/ingen63/VivaPlan/issues/139) · geschlossen · Umsetzung: 4128e09

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #139/AK1 | Die Prüfung meldet einen Fehler, wenn Pfade × Monate eine Grenze überschreitet. | ☑ | **gilt** | [validate_extra.rs](../crates/engine/tests/validate_extra.rs) | 18.11.2026 ok |
| #139/AK2 | Beispiel: 1'000'000 Pfade, Planungsdauer 100 Jahre → Fehler mit Feldbezug auf die Anzahl Pfade und einem Vorschlag für die höchste … | ☑ | **gilt** | [validate_extra.rs](../crates/engine/tests/validate_extra.rs) | 18.11.2026 ok |
| #139/AK3 | Beispiel: 50'000 Pfade, Planungsdauer 30 Jahre → kein Fehler (heute übliche Rechnung). | ☑ | **gilt** | [validate_extra.rs](../crates/engine/tests/validate_extra.rs) | 18.11.2026 ok |
| #139/AK4 | Ein Test misst den höchsten Speicherverbrauch (über `engine::test_alloc`) einer Rechnung genau an der Grenze, und dieser bleibt unter … | ☑ | **gilt** | [qa_matrix.rs](../crates/engine/tests/qa_matrix.rs) | 18.11.2026 ok |
| #139/AK5 | Der Stresstest `stress_couple_four_properties_100_years` läuft wieder grün (mit einer Pfadzahl unter der Grenze) oder ist begründet … | ☑ | **gilt** | [qa_matrix.rs](../crates/engine/tests/qa_matrix.rs) «stress_couple_four_properties_100_years» | 18.11.2026 ok |
| #139/AK6 | Die Hilfe (Abschnitt Monte Carlo) nennt die Grenze. | ☑ | **gilt** | kein Test (Durchsicht `hilfe/20-renditen.md`) | 18.11.2026 ok |

### #140 Zahleneingabe mit Punkt als Tausendertrennzeichen wird still um den Faktor 1000 falsch übernommen

[Issue #140](https://github.com/ingen63/VivaPlan/issues/140) · geschlossen · Umsetzung: 0c4d328

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #140/AK1 | In Feldern mit einem Betrag in Währung wird eine Eingabe mit Punkt und genau drei folgenden Ziffern («1.234», «12.500», «1.234.567») … | ☑ | **gilt** | [ui-planung.spec.js](../e2e/ui-planung.spec.js) | 18.11.2026 ok |
| #140/AK2 | «0x10», «1e5» und «Infinity» werden in allen Zahlenfeldern abgelehnt. | ☑ | **gilt** | [ui-planung.spec.js](../e2e/ui-planung.spec.js) | 18.11.2026 ok |
| #140/AK3 | Unverändert gültig sind «1'234.50», «1’234,5», «12,5», «1 234», «-3» und in Prozent- und Jahresfeldern «1.25». | ☑ | **gilt** | [util.test.js](../e2e/unit/util.test.js) «#140/AK3» | 18.11.2026 ok |
| #140/AK4 | Unit-Tests in `e2e/unit/util.test.js` decken alle Beispiele dieser Liste ab; | ☑ | **gilt** | [ui-planung.spec.js](../e2e/ui-planung.spec.js) | 18.11.2026 ok |
| #140/AK5 | Die Hilfe (Eingabe von Zahlen) nennt die erlaubten Schreibweisen. | ☑ | **gilt** | kein Test (Durchsicht) | 18.11.2026 ok |

### #141 Tests und Rauchtest unter Windows und macOS

[Issue #141](https://github.com/ingen63/VivaPlan/issues/141) · geschlossen · Umsetzung: 1ddc59d

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #141/AK1 | Ein nächtlicher Job (nur nach neuem Commit, wie die übrigen nächtlichen Jobs) führt `cargo nextest run --workspace` unter … | ☑ | **gilt** | kein Test (Durchsicht `test-quality.yml`) | 18.11.2026 ok |
| #141/AK2 | Die Golden Masters und `ods_reference` bestehen auf allen drei Betriebssystemen mit der heutigen Toleranz; | ☑ | **gilt** | Jobs `tests-os` (Windows, macOS) und `tests-coverage` (Linux), [Lauf 44274654652](https://github.com/ingen63/VivaPlan/actions/runs/44274654652) | 18.11.2026 ok: `golden::` und `ods_reference::` auf allen drei Systemen grün |
| #141/AK3 | Tests für Pfade unter Windows (Laufwerksbuchstabe, Backslash, Leerzeichen und Umlaute im Namen, `.rplan` → `.vplan`) laufen im Windows-Job. | ☑ | **gilt** | [main.rs](../crates/app/src/main.rs) | 18.11.2026 ok: `issue_141_ak3_windows_paths` grün in `tests-os (windows-latest)` |
| #141/AK4 | Vor dem Release-Entwurf startet jedes Paket in einem Rauchtest: Die App öffnet die Beispielplanung, rechnet einen festen Lauf und … | ☑ | **gilt** | [main.rs](../crates/app/src/main.rs), [Lauf 44270687642](https://github.com/ingen63/VivaPlan/actions/runs/44270687642) | 18.11.2026 ok: Rauchtest auf Linux, macOS und Windows «ok – 472 Monate gerechnet» |
| #141/AK5 | `docs/TESTS.md` nennt die Plattformen und die Jobs. | ☑ | **gilt** | kein Test (Durchsicht) | 18.11.2026 ok |

### #142 Vertragstest zwischen Dev-Server und Tauri-Commands

[Issue #142](https://github.com/ingen63/VivaPlan/issues/142) · geschlossen · Umsetzung: b203250

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #142/AK1 | Eine gemeinsame Tabelle (eine Datei) nennt jeden Rechenbefehl, den beide Verteiler anbieten, mit Beispielargumenten, wie sie die … | ☑ | **gilt** | [main.rs](../crates/app/src/main.rs) | 18.11.2026 ok |
| #142/AK2 | Ein Test schickt jeden Eintrag durch die Tauri-IPC der Mock-App (`get_ipc_response`) und durch `devserver::call` und verlangt dieselbe … | ☑ | **gilt** | [main.rs](../crates/app/src/main.rs) | 18.11.2026 ok |
| #142/AK3 | Ein Test schlägt fehl, wenn ein Befehl in `generate_handler!` oder im Dev-Server dazukommt und in der Tabelle fehlt; | ☑ | **gilt** | [dispatch.rs](../crates/engine/src/dispatch.rs), [main.rs](../crates/app/src/main.rs) | 18.11.2026 ok |
| #142/AK4 | Beispiel: `run_simulation` mit `jobId` und anschliessend `progress_get` mit derselben Kennung liefern in beiden Verteilern denselben … | ☑ | **gilt** | [main.rs](../crates/app/src/main.rs) «issue_142_ak2_…» | 18.11.2026 ok |
| #142/AK5 | `docs/TESTS.md` (Schicht 6) nennt den Test. | ☑ | **gilt** | kein Test (Durchsicht) | 18.11.2026 ok |

### #144 Unbekannte Formatversion beim Öffnen melden

[Issue #144](https://github.com/ingen63/VivaPlan/issues/144) · geschlossen · Umsetzung: 4678dc0

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #144/AK1 | `"format": "vivaplan/1"`, `"pensionsplaner/1"` (alte Kennung) und eine fehlende Kennung öffnen wie heute. | ☑ | **gilt** | [legacy_format.rs](../crates/engine/tests/legacy_format.rs) | 18.11.2026 ok |
| #144/AK2 | `"format": "vivaplan/2"` (und jede andere unbekannte Kennung) ergibt beim Öffnen eine Fehlermeldung, die die Kennung nennt und sagt, … | ☑ | **gilt** | [legacy_format.rs](../crates/engine/tests/legacy_format.rs), [main.rs](../crates/app/src/main.rs), [ui-eingaben.spec.js](../e2e/ui-eingaben.spec.js) | 18.11.2026 ok |
| #144/AK3 | Dasselbe gilt für die Kommandozeile (`vivaplan run`, `vivaplan validate`): Fehlercode ungleich 0 und dieselbe Meldung. | ☑ | **gilt** | [cli.rs](../crates/cli/tests/cli.rs) | 18.11.2026 ok |
| #144/AK4 | Rust-Tests für alle Fälle oben; | ☑ | **gilt** | [ui-eingaben.spec.js](../e2e/ui-eingaben.spec.js) | 18.11.2026 ok |
| #144/AK5 | `docs/ENTWICKLUNG.md` (Dateiformat) beschreibt die Regel. | ☑ | **gilt** | kein Test (Durchsicht) | 18.11.2026 ok |

### #145 AHV-Plafond des Ehepaars steigt mit der doppelten Teuerung

[Issue #145](https://github.com/ingen63/VivaPlan/issues/145) · geschlossen · Umsetzung: d6c9a73

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #145/AK1 | Der Plafond steigt im Januar um dieselbe Teuerung wie eine einzelne angepasste Rente, unabhängig davon, wie viele Personen eine Rente … | ☑ | **gilt** | [household.rs](../crates/engine/tests/household.rs) | 18.11.2026 ok |
| #145/AK2 | Referenztest mit dem Beispiel oben: Januar 2031 = 3'893.40, Januar 2033 = 4'130.51 (Toleranz 0.01 CHF). | ☑ | **gilt** | [household.rs](../crates/engine/tests/household.rs) | 18.11.2026 ok |
| #145/AK3 | Referenztest: nur eine der beiden Renten mit Teuerung, beide laufend: Plafond und Summe wie von Hand nachgerechnet (Rechnung im Kommentar). | ☑ | **gilt** | [household.rs](../crates/engine/tests/household.rs) | 18.11.2026 ok |
| #145/AK4 | Referenztest: Die zweite Rente beginnt später als die erste; | ☑ | **gilt** | [household.rs](../crates/engine/tests/household.rs) | 18.11.2026 ok |
| #145/AK5 | Ändern sich Golden-Dateien (z. | ☑ | **gilt** | kein Test (Git-Verlauf Golden) | 18.11.2026 ok |

### #146 Golden Masters nächtlich im Release-Profil

[Issue #146](https://github.com/ingen63/VivaPlan/issues/146) · geschlossen · Umsetzung: 53dbf09

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #146/AK1 | Ein nächtlicher Job in `test-quality.yml` (nur nach neuem Commit, wie die übrigen nächtlichen Jobs) führt die Golden Masters … | ☑ | **gilt** | kein Test (Durchsicht `test-quality.yml`) | 18.11.2026 ok |
| #146/AK2 | Die Golden Masters bestehen dort mit der heutigen Toleranz (1e-9 relativ), ohne `GOLDEN_UPDATE`. | ☑ | **gilt** | Job `release-golden`, [Lauf 44274654652](https://github.com/ingen63/VivaPlan/actions/runs/44274654652) | 18.11.2026 ok: 9 Tests im Release-Profil grün, ohne `GOLDEN_UPDATE` |
| #146/AK3 | Ein Fehlschlag lässt den Lauf fehlschlagen und ist in der Zusammenfassung sichtbar. | ☑ | **gilt** | kein Test (Durchsicht) | 18.11.2026 ok |
| #146/AK4 | `docs/TESTS.md` nennt den Job und den Grund. | ☑ | **gilt** | kein Test (Durchsicht) | 18.11.2026 ok |

### #147 Gespeicherte Ergebnisse in IndexedDB testen

[Issue #147](https://github.com/ingen63/VivaPlan/issues/147) · geschlossen · Umsetzung: 089585d

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #147/AK1 | Unit-Tests ohne Browser (`node --test` in `e2e/unit/`, mit einer IndexedDB-Nachbildung als reine Testabhängigkeit) prüfen `add`, `list` … | ☑ | **gilt** | [runs.test.js](../e2e/unit/runs.test.js) «#147/AK1» | 18.11.2026 ok |
| #147/AK2 | Test: Bricht eine Transaktion ab (Kontingent erschöpft, nachgebildet), wird das Versprechen mit einem Fehler abgelehnt; | ☑ | **gilt** | [ui-gespeichert.spec.js](../e2e/ui-gespeichert.spec.js) | 18.11.2026 ok |
| #147/AK3 | Test: Ist IndexedDB nicht verfügbar, fällt die Liste wie heute auf den Arbeitsspeicher zurück. | ☑ | **gilt** | [runs.test.js](../e2e/unit/runs.test.js) «#147/AK3» | 18.11.2026 ok |
| #147/AK4 | Test: Ein gespeichertes Ergebnis einer älteren Version (ohne Felder, die später dazukamen, z. | ☑ | **gilt** | [runs.test.js](../e2e/unit/runs.test.js) «#147/AK4» | 18.11.2026 ok |
| #147/AK5 | Der Job `ui-unit` in `test-quality.yml` führt die neuen Tests aus. | ☑ | **gilt** | Job `ui-unit` (GitHub, c702e99) | 18.11.2026 ok |

### #148 Statistiktests der Renditepfade mit Vertrauensgrenzen

[Issue #148](https://github.com/ingen63/VivaPlan/issues/148) · geschlossen · Umsetzung: 5d8e2ec

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #148/AK1 | Bootstrap: Mittelwert und Streuung der Jahresrenditen von Aktien, Obligationen und Inflation über die Pfade liegen innerhalb von 4 … | ☑ | **gilt** | [statistics.rs](../crates/engine/tests/statistics.rs) | 18.11.2026 ok (Präzisierung: Monatswerte statt Streuung der Jahresrendite, begründet) |
| #148/AK2 | Parametrisch: Mittelwert und Streuung der Jahresrendite der Aktien liegen innerhalb von 4 Standardfehlern der Modellwerte (aus μ, σ und … | ☑ | **gilt** | [statistics.rs](../crates/engine/tests/statistics.rs) | 18.11.2026 ok |
| #148/AK3 | Parametrisch: Die Korrelation zwischen Aktien und Obligationen sowie zwischen Aktien und Inflation liegt im 99-%-Vertrauensintervall … | ☑ | **gilt** | [statistics.rs](../crates/engine/tests/statistics.rs) | 18.11.2026 ok |
| #148/AK4 | Beispiel: Eine künstlich um 1 Prozentpunkt pro Jahr verschobene Aktienrendite lässt den Test fehlschlagen (einmal von Hand gezeigt und … | ☑ | **gilt** | [statistics.rs](../crates/engine/tests/statistics.rs) (in den Tests zu AK1–AK3) | 18.11.2026 ok |
| #148/AK5 | Die Tests bleiben bei festem Startwert reproduzierbar und laufen im Standard unter 10 s; | ☑ | **gilt** | [statistics.rs](../crates/engine/tests/statistics.rs) | 18.11.2026 ok |

### #150 Kapitalbezüge unverheirateter Paare werden für den Haushalt zusammengerechnet

[Issue #150](https://github.com/ingen63/VivaPlan/issues/150) · geschlossen · Umsetzung: a841d27

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #150/AK1 | Bei unverheirateten Paaren rechnet die Kapitalbezugssteuer die Bezüge eines Jahres je Person zusammen, nicht für den Haushalt. | ☑ | **gilt** | [household.rs](../crates/engine/tests/household.rs) «issue_150_ak1_…» | 20.11.2026 ok |
| #150/AK2 | Bei Verheirateten und bei einer Person bleibt die Rechnung unverändert; die Golden-Dateien ändern nicht (oder nur mit Begründung, … | ☑ | **gilt** | [household.rs](../crates/engine/tests/household.rs) «issue_150_ak3_…», «issue_155_…», Golden-Tests; `make golden-numbers` | 20.11.2026 ok (alle Zahlen unverändert) |
| #150/AK3 | Ein Test belegt das Beispiel (zwei Personen, nicht verheiratet, je 500 000 im selben Jahr) mit Handwert, ein zweiter Test den Fall … | ☑ | **gilt** | [household.rs](../crates/engine/tests/household.rs) «issue_150_ak1_…», «issue_150_ak3_…» | 20.11.2026 ok |
| #150/AK4 | Gilt auch für den Modus «Wohnort» (#64), sobald er umgesetzt ist. | ☑ | **nicht anwendbar** (#64 nicht umgesetzt; Anforderung als Kommentar in #64) | kein Test | 20.11.2026 nicht anwendbar |

### #153 Mitgelieferte Planungen sind beim Aktienmodell nicht auf dem aktuellen Stand

[Issue #153](https://github.com/ingen63/VivaPlan/issues/153) · geschlossen · Umsetzung: 0c6a760

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #153/AK1 | `example.vplan` und `new.vplan` enthalten `"auto": true` bei `returns.parametric`; | ☑ | **gilt** | [proptest_upgrade.rs](../crates/engine/tests/proptest_upgrade.rs) «issue_137_ak5_…» | 18.11.2026 ok |
| #153/AK2 | Ändern sich dadurch Ergebnisse von Tests (z. | ☑ | **gilt** | kein Test (Git-Verlauf Golden) | 18.11.2026 ok |

### #154 Nachführen holt die Wohnkosten-Anpassung erst beim zweiten Öffnen nach

[Issue #154](https://github.com/ingen63/VivaPlan/issues/154) · geschlossen · Umsetzung: aa5bde3

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #154/AK1 | Der Schritt `housing` läuft nach den Schritten, die die Planung lesbar machen; | ☑ | **gilt** | [proptest_upgrade.rs](../crates/engine/tests/proptest_upgrade.rs) «proptest_upgrade_is_idempotent» | 18.11.2026 ok |
| #154/AK2 | Regressionstest mit dem verkleinerten Fall. | ☑ | **gilt** | [upgrade.rs](../crates/engine/src/upgrade.rs) «issue_154_…» | 18.11.2026 ok |

### #155 Referenztest: Kapitalbezug beider Partner im selben Jahr

[Issue #155](https://github.com/ingen63/VivaPlan/issues/155) · geschlossen · Umsetzung: d47cb6e

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #155/AK1 | Ein Rust-Test (`household.rs` oder `reference.rs`) rechnet das Beispiel mit Handwerten nach: Steuer je Bezug und Jahressumme, Abzug im … | ☑ | **gilt** | [household.rs](../crates/engine/tests/household.rs) «issue_155_ak1_…» | 19.11.2026 ok |
| #155/AK2 | Ein zweiter Fall mit Bezügen in zwei verschiedenen Jahren zeigt, dass jedes Jahr für sich besteuert wird. | ☑ | **gilt** | [household.rs](../crates/engine/tests/household.rs) «issue_155_ak2_…» | 19.11.2026 ok |
| #155/AK3 | Rechenkern und Golden-Dateien bleiben unverändert. | ☑ | **gilt** | kein Test (Git-Diff d47cb6e) | 19.11.2026 ok |

### #156 Steuerberechnung von Oberfläche und Rechenkern mit gemeinsamen Testfällen prüfen

[Issue #156](https://github.com/ingen63/VivaPlan/issues/156) · geschlossen · Umsetzung: a3cd177

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #156/AK1 | Gemeinsame Testfälle (JSON: Tabelle, Betrag, Steuerfuss, erwartete Steuer) laufen in einem Rust-Test gegen `tax::rate_at`/`tax::tax` … | ☑ | **gilt** | [tax_parity.rs](../crates/engine/tests/tax_parity.rs) «issue_156_ak1_…», [tax-parity.test.js](../e2e/unit/tax-parity.test.js) «#156/AK1» | 20.11.2026 ok (18 Fälle; Gegenprobe mit altem `rateAt` scheitert) |
| #156/AK2 | Die Abweichung beim Steuerfuss 0 ist geklärt: entweder rechnet die Oberfläche wie der Rechenkern, oder die Prüfung der Planung lässt … | ☑ | **gilt** (Oberfläche rechnet wie der Rechenkern) | [tax-parity.test.js](../e2e/unit/tax-parity.test.js) «#156/AK2», [mandatory_inputs.rs](../crates/engine/tests/mandatory_inputs.rs) «issue_156_ak2_…» | 20.11.2026 ok |

### #157 Netzwerkfreiheit automatisch prüfen

[Issue #157](https://github.com/ingen63/VivaPlan/issues/157) · geschlossen · Umsetzung: c6c1ac4

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #157/AK1 | Test oder Workflow-Schritt prüft mit `cargo tree` (App, Engine, Kommandozeile, ohne Entwicklungsabhängigkeiten), dass keine Netzwerk-Bibliothek aus einer Sperrliste enthalten ist; ein Treffer lässt die Prüfung fehlschlagen. | ☑ | **gilt** (sechs Desktop-Ziele statt aller Ziele, Entscheid im Issue) | [check-network-free.sh](../tools/check-network-free.sh), [test_check_network_free.py](../tools/test_check_network_free.py) (#157/AK1), Job `network-free` in `test-quality.yml` | 19.11.2026 ok (Skript und Tests grün; Job auf GitHub vor dem Push noch nicht gelaufen) |
| #157/AK2 | Test prüft die CSP in `crates/app/tauri.conf.json`: `connect-src` erlaubt nur `'self'` und die IPC-Quellen. | ☑ | **gilt** | [app/tests/csp.rs](../crates/app/tests/csp.rs) «issue_157_ak2_…» (2 Tests) | 19.11.2026 ok |
| #157/AK3 | `docs/TESTS.md` nennt die Prüfung. | ☑ | **gilt** | kein Test (Durchsicht `docs/TESTS.md`) | 19.11.2026 ok |

### #158 Gemeinsamer Plan-Builder für Rust-Tests und Aufteilung von ui.spec.js

[Issue #158](https://github.com/ingen63/VivaPlan/issues/158) · geschlossen · Umsetzung: 2a92011 (AK2 bereits in 268d31a)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #158/AK1 | Gemeinsamer Plan-Builder unter `crates/engine/tests/`, in `all.rs` eingebunden, kennt die Pflichtfelder, Bausteine für Person, Paar, Immobilie, Monte Carlo parametrisch/Bootstrap; die mehrfach vorhandenen Planungen nutzen ihn. | ☑ | **gilt** | [plan_builder.rs](../crates/engine/tests/plan_builder.rs), genutzt in `simulation.rs`, `optimize_goal.rs`, `rerun_paths.rs`, `qa_matrix.rs` | 19.11.2026 ok (Paar und Immobilie nur durchgesehen, noch nicht genutzt) |
| #158/AK2 | `e2e/ui.spec.js` nach Bereichen aufgeteilt, gemeinsame Hilfen in eigener Datei. | ☑ | **gilt** | kein Test (Durchsicht: 11 Spec-Dateien, `e2e/helpers.js`) | 19.11.2026 ok |
| #158/AK3 | Anzahl und Ergebnis der Tests bleiben gleich; Zuordnung zu Abnahmekriterien im Bericht ändert sich nicht. | ☑ | **gilt** | kein Test (Vergleich vor/nach dem Commit) | 19.11.2026 ok (454 `#[test]` und 623 nextest-Einträge vorher wie nachher, keine Namen oder Kennungen geändert, Integrationstests grün; `make report` nicht ausgeführt) |

### #159 Nächtliche Läufe zuverlässig machen und von Hand vollständig startbar

[Issue #159](https://github.com/ingen63/VivaPlan/issues/159) · geschlossen · Umsetzung: b46150f

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #159/AK1 | Beim Start von Hand lassen sich alle nächtlichen Jobs ausführen, auch `fuzz` und `mutants-diff`; ohne die Eingabe bleibt der Start von Hand wie heute. | ☑ | **gilt** | [Lauf 44280201847](https://github.com/ingen63/VivaPlan/actions/runs/44280201847) (`nightly=true`) | 18.11.2026 ok |
| #159/AK2 | Ein Push bricht einen laufenden nächtlichen Lauf (oder einen Start von Hand mit allen nächtlichen Jobs) nicht ab; | ☑ | **gilt** | Läufe 44280164629 (Push) und 44280201847 nebeneinander | 18.11.2026 ok (Abbruch des vorherigen Push-Laufs nur in der Konfiguration geprüft) |
| #159/AK3 | Der Cron-Zeitpunkt liegt nicht zur vollen oder halben Stunde. | ☑ | **gilt** | kein Test (Durchsicht `test-quality.yml`) | 18.11.2026 ok |
| #159/AK4 | `mutants-diff` schliesst auch nach zwei ausgefallenen Nächten innerhalb seines Zeitlimits ab; | ☑ | **gilt** | [Lauf 44375155792](https://github.com/ingen63/VivaPlan/actions/runs/44375155792), Job `mutants-diff` | 18.11.2026 ok: 16 Teile, alle grün, längster Teil rund 40 Min. (Limit 120) |
| #159/AK5 | `docs/TESTS.md` beschreibt, wie man die nächtlichen Jobs von Hand startet und dass `schedule` verspätet sein oder ausfallen kann. | ☑ | **gilt** | kein Test (Durchsicht) | 18.11.2026 ok |

### #160 Weniger Kontext bei der Arbeit mit Claude Code

[Issue #160](https://github.com/ingen63/VivaPlan/issues/160) · geschlossen · Umsetzung: 5bdf2d5 (Beschreibung in `docs/TESTS.md` mit a4070d1)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #160/AK1 | Ein Make-Ziel (z. B. `make test-quiet`) führt dieselben Tests wie `make test` aus, gibt aber nur fehlgeschlagene und langsame Tests sowie … | ☑ | **gilt** | kein Test (Lauf `make test-quiet`, Probe mit rotem Test) | 18.11.2026 ok (6 Zeilen statt rund 740) |
| #160/AK2 | `CLAUDE.md` (Abschnitt Tests) nennt: beim Arbeiten gezielte Tests, vor dem Commit einmal der volle Lauf über das leise Ziel. | ☑ | **gilt** | kein Test (Durchsicht `CLAUDE.md`) | 18.11.2026 ok |
| #160/AK3 | `CLAUDE.md` (Abschnitt Regeln) nennt: Dateien über rund 500 Zeilen nur gezielt lesen; Golden-Dateien nie ganz lesen, … | ☑ | **gilt** | kein Test (Durchsicht `CLAUDE.md`) | 18.11.2026 ok |
| #160/AK4 | `CLAUDE.md` nennt: keine neue Abhängigkeit (Crate, npm-Paket) ohne Rückfrage. | ☑ | **gilt** | kein Test (Durchsicht `CLAUDE.md`) | 18.11.2026 ok |
| #160/AK5 | `CLAUDE.md` nennt eine Stopp-Regel: Ist ein Übersetzungs- oder Testfehler nach drei Versuchen nicht behoben, … | ☑ | **gilt** | kein Test (Durchsicht `CLAUDE.md`) | 18.11.2026 ok |
| #160/AK6 | Der Hook nach dem Commit (`.claude/hooks/after-commit.sh`) führt die Tests der geänderten Pakete selbst leise aus und meldet nur das … | ☑ | **gilt** | [test_after_commit.py](../tools/test_after_commit.py) «test_issue_160_ak6_…» | 18.11.2026 ok (dazu echter Lauf auf 2f0d36a: 82 s, 733 Tests, nur Zusammenfassung) |
| #160/AK7 | Die Regeln zu Tests je Abnahmekriterium, Abdeckung (80 %), Mutanten und Dokumentation in `CLAUDE.md` bleiben unverändert. | ☑ | **gilt** | kein Test (Diff `CLAUDE.md` in 5bdf2d5) | 18.11.2026 ok |
| #160/AK8 | `docs/TESTS.md` beschreibt das neue Make-Ziel und den geänderten Hook. | ☑ | **gilt** | kein Test (Durchsicht `docs/TESTS.md`) | 18.11.2026 ok |

### #161 Import alter Konfiguration stürzt bei extremem Startalter ab

[Issue #161](https://github.com/ingen63/VivaPlan/issues/161) · geschlossen · Umsetzung: e483d57

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #161/AK1 | Import mit extremem `General.StartAge` (±, auch ausserhalb `i32`) stürzt nicht ab. | ☑ | **gilt** | `legacy_import.rs` `issue_161_ak1_ak2_…` | 18.11.2026 ok |
| #161/AK2 | Regressionstest liest genau die gefundene Eingabe ein. | ☑ | **gilt** | `legacy_import.rs` `issue_161_ak1_ak2_…` | 18.11.2026 ok |
| #161/AK3 | Eingabe liegt im Startkorpus `fuzz/seeds/import_legacy/`. | ☑ | **gilt** | `issue-161-extremes-startalter` | 18.11.2026 ok; Fuzz-Lauf grün (Lauf 44375155792) |

### #162 Nicht erkannte Mutanten in den Änderungen vom 15. bis 18.11.2026

[Issue #162](https://github.com/ingen63/VivaPlan/issues/162) · geschlossen · Umsetzung: 23201ec

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #162/AK1 | Jeder genannte Mutant ist mit einem Test erkannt oder mit Begründung ausgeschlossen. | ☑ | **gilt** | 7 Tests `issue_162_ak1_…`, `.cargo/mutants.toml` | 18.11.2026 ok |
| #162/AK2 | `cargo mutants --in-diff` meldet keinen dieser Mutanten mehr als nicht erkannt. | ☑ | **gilt** | [Lauf 44375155792](https://github.com/ingen63/VivaPlan/actions/runs/44375155792), `mutants-diff` (16 Teile) | 18.11.2026 ok: alle Teile grün |

### #163 Tests der Desktop-App teilen sich den Datenordner und sind unzuverlässig

[Issue #163](https://github.com/ingen63/VivaPlan/issues/163) · geschlossen · Umsetzung: e81169a, 67f4e4c

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #163/AK1 | Jeder Testprozess hat einen eigenen Datenordner; nach dem Test bleibt keiner zurück. | ☑ | **gilt** | `main.rs` Tests der App | 18.11.2026 ok: 93 Tests grün, keine Ordner übrig |
| #163/AK2 | `issue_106_…` besteht in 20 Läufen hintereinander ohne Fehlversuch. | ☑ | **gilt** | `issue_106_einstellung_…` | 18.11.2026 ok: 20 von 20 |
| #163/AK3 | Lauf auf GitHub (`tests-coverage`) ist ohne `FLKY`-Meldung grün. | ☑ | **gilt** | [Lauf 44375155792](https://github.com/ingen63/VivaPlan/actions/runs/44375155792) | 18.11.2026 ok |

### #165 Code intern englisch: Regel, Glossar und Prüfskript

[Issue #165](https://github.com/ingen63/VivaPlan/issues/165) · geschlossen · Umsetzung: 7b8a175, afbf054

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #165/AK1 | `CLAUDE.md` und `docs/ENTWICKLUNG.md` enthalten die Regel «Code, Kommentare, Bezeichner und Testnamen englisch; Oberflächentexte, Hilfe, Doku, Issues und Commits deutsch». | ☑ | **gilt** | kein Test (Durchsicht `CLAUDE.md`, `ENTWICKLUNG.md` Abschnitt «Sprache im Code») | 19.11.2026 ok |
| #165/AK2 | Ein Glossar (`docs/GLOSSAR.md`) legt die englischen Bezeichner für Schweizer Fachbegriffe fest; Schweizer Kürzel bleiben als Bezeichner erlaubt. | ☑ | **gilt** (vorerst nur Fachbegriff → Bezeichner; Oberflächensprachen mit #172, #174, #175, Kommentar im Issue) | kein Test (Durchsicht `docs/GLOSSAR.md`: Beispiele des Issues und Abschnitt «Schweizer Kürzel» vorhanden) | 19.11.2026 ok |
| #165/AK3 | Ein Prüfskript (`tools/check-english.sh`, Ziel `make english`) meldet deutsche Kommentare und Testnamen mit Datei und Zeile; Oberflächentexte und Golden-Dateien ausgenommen. | ☑ | **gilt** | [test_check_english.py](../tools/test_check_english.py) «test_ak3_…» (10 Tests) | 19.11.2026 ok: Tests grün; `--all` auf dem Repo meldet 3926 Zeilen mit Datei:Zeile |
| #165/AK4 | Übergangsliste noch nicht umgestellter Pfade; ist sie leer, läuft die Prüfung in `make test-quiet` und im Workflow mit. | ☑ | **gilt** | [test_check_english.py](../tools/test_check_english.py) «test_ak4_…» (6 Tests) | 19.11.2026 ok: `make test-quiet` ruft `--gate` auf (übersprungen, 6 Pfade); Job `english` in `test-quality.yml` (YAML gültig, auf GitHub noch nicht gelaufen) |
| #165/AK5 | Tests des Skripts (Treffer, kein Treffer, Ausnahmen) wie bei `check-network-free.sh`. | ☑ | **gilt** | [test_check_english.py](../tools/test_check_english.py) (16 Tests) | 19.11.2026 ok |

### #166 Code intern englisch: Rechenkern

[Issue #166](https://github.com/ingen63/VivaPlan/issues/166) · geschlossen · Umsetzung: 0bd9e13 … 82be2e0 (Kommentare je Datei, 9730888 Bezeichner und Testnamen)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #166/AK1 | `tools/check-english.sh` meldet für `crates/engine/` keine Treffer; die Pfade sind aus der Übergangsliste entfernt. | ☑ | **gilt** | [test_check_english.py](../tools/test_check_english.py) «test_ak4_transition_list_holds_unconverted_paths»; `tools/check-english.sh --all crates/engine` | 19.11.2026 ok: keine Treffer in 88 Dateien; Übergangsliste nur noch App, e2e |
| #166/AK2 | Bezeichner und Testnamen folgen dem Glossar; Kriterien-Bezüge bleiben erhalten (`#27/AK2`, `issue_27_ak2_…`, `AK2` im Testnamen innerhalb von `test.describe("… (#27)")`). | ☑ | **gilt** | kein Test (Vergleich der Kennungen vor und nach der Umstellung) | 19.11.2026 ok: `#Nr/AKn`, `issue_Nr_akn` in `crates/engine` vor 06dd6ba und nach 82be2e0 identisch; Modul `example_neu` nach der Vorlage `new.vplan` benannt |
| #166/AK3 | Keine Verhaltensänderung: `make test-quiet` grün, Golden-Dateien unverändert, Zeilenabdeckung je Paket nicht gesunken. | ☑ | **gilt** | `make test-quiet` (744 Tests), [test_check_english.py](../tools/test_check_english.py) (16 Tests) | 19.11.2026 ok: alles grün, Golden-Dateien unverändert; Abdeckung vorher/nachher Engine 99,06/99,13 %, Kommandozeile 96,07/96,45 %, App 91,38/91,33 % (App unverändert, Unterschied durch lokale `shiller-index.ods`) |
| #166/AK4 | Oberflächentexte, Meldungstexte und Dateischlüssel der `.rplan`-Datei sind unverändert (bleiben deutsch bis #62). | ☑ | **gilt** | kein Test (Vergleich aller String-Literale vor und nach der Umstellung) | 19.11.2026 ok: einziger Unterschied das Formatargument `{ferien:?}` → `{holiday:?}`; Assert-Meldungen in Tests bleiben deutsch |

### #167 Code intern englisch: Desktop-App und Oberfläche

[Issue #167](https://github.com/ingen63/VivaPlan/issues/167) · geschlossen · Umsetzung: 554ac93 … 4f561fe (Kommentare 554ac93, f086a37, aa77996; Bezeichner und Testnamen f1cc28d, 3fa900c)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #167/AK1 | `tools/check-english.sh` meldet für `crates/app/` keine Treffer; die Pfade sind aus der Übergangsliste entfernt. | ☑ | **gilt** | [test_check_english.py](../tools/test_check_english.py) «test_ak4_transition_list_holds_unconverted_paths»; `tools/check-english.sh --all crates/app` | 19.11.2026 ok: keine Treffer in 34 Dateien; Übergangsliste nur noch e2e |
| #167/AK2 | Bezeichner und Testnamen folgen dem Glossar; Kriterien-Bezüge bleiben erhalten (`#27/AK2`, `issue_27_ak2_…`, `AK2` im Testnamen innerhalb von `test.describe("… (#27)")`). | ☑ | **gilt** | kein Test (Vergleich der Kennungen vor und nach der Umstellung) | 19.11.2026 ok: `#Nr/AKn`, `issue_Nr_akn` in `crates/app` vor 3f55d98 und nach 4f561fe identisch; IPC-Befehle, `data-*`, CSS-Klassen/IDs, Ergebnisschlüssel und von `e2e/` aufgerufene Funktionen unverändert |
| #167/AK3 | Keine Verhaltensänderung: `make test-quiet` grün, Golden-Dateien unverändert, Zeilenabdeckung je Paket nicht gesunken. | ☑ | **gilt** | `make test-quiet` (744 Tests), Playwright (221), `e2e/unit` (27), `tools/ui-smoke.sh`, [test_check_english.py](../tools/test_check_english.py) (16) | 19.11.2026 ok: alles grün, Golden-Dateien unverändert; Abdeckung App vorher/nachher 90,96/90,95 %, Produktivcode gleich abgedeckt (Unterschied nur im Testzweig der lokalen `shiller-index.ods`) |
| #167/AK4 | Oberflächentexte, Meldungstexte und Dateischlüssel der `.rplan`-Datei sind unverändert (bleiben deutsch bis #62). | ☑ | **gilt** | kein Test (Durchsicht aller geänderten Zeilen mit String-Literalen) | 19.11.2026 ok: nur Testdaten-Pfade und eine Assert-Meldung in `csp.rs` geändert; übrige Assert-Meldungen in Tests bleiben deutsch |

### #168 Code intern englisch: Oberflächentests

[Issue #168](https://github.com/ingen63/VivaPlan/issues/168) · geschlossen · Umsetzung: aee429e (Kommentare), c98b1d0 (Testtitel, Übergangsliste leer)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #168/AK1 | `tools/check-english.sh` meldet für `e2e/` keine Treffer; die Pfade sind aus der Übergangsliste entfernt. | ☑ | **gilt** | [test_check_english.py](../tools/test_check_english.py) «test_ak4_transition_list_is_empty», «test_ak4_gate_checks_e2e_with_real_list»; `tools/check-english.sh --all e2e`, `--gate` | 19.11.2026 ok: keine Treffer in 18 Dateien (`e2e/`) bzw. 205 Dateien (`--gate`); Übergangsliste leer, `--gate` läuft in `make test-quiet` und im Job `english`; ein Fehlalarm markiert (Suchbegriff «liquidity» in `docs.test.js`) |
| #168/AK2 | Bezeichner und Testnamen folgen dem Glossar; Kriterien-Bezüge bleiben erhalten (`#27/AK2`, `issue_27_ak2_…`, `AK2` im Testnamen innerhalb von `test.describe("… (#27)")`). | ☑ | **gilt** | kein Test (Vergleich der Kennungen vor und nach der Umstellung mit `scan_marks` aus `tools/report/report.py`) | 19.11.2026 ok: 179 Kriterien-Bezüge in `e2e/` vor 4a14360 und nach c98b1d0 identisch (inkl. Anzahl Fundstellen); Bezeichner waren bereits englisch; Zeilennummern unverändert |
| #168/AK3 | Keine Verhaltensänderung: `make test-quiet` grün, Golden-Dateien unverändert, Zeilenabdeckung je Paket nicht gesunken. | ☑ | **gilt** | `make test-quiet` (744 Tests + Englisch-Prüfung), Playwright (221), `e2e/unit` (27), `tools/ui-smoke.sh`, [test_check_english.py](../tools/test_check_english.py) (17) | 19.11.2026 ok: alles grün, gleiche Testanzahl wie vorher; `crates/` und Golden-Dateien unverändert, daher Abdeckung je Paket gleich |
| #168/AK4 | Oberflächentexte, Meldungstexte und Dateischlüssel der `.rplan`-Datei sind unverändert (bleiben deutsch bis #62). | ☑ | **gilt** | kein Test (Vergleich aller Zeilen ohne Kommentare und Testtitel) | 19.11.2026 ok: ausser Kommentaren und Titeln nur drei Assert-Meldungen in `e2e/unit` geändert; Suchen nach Oberflächentexten, `data-*`, CSS-Klassen/IDs, IPC-Namen und Planungsschlüssel unverändert |

### #169 Code intern englisch: Kommandozeile, fuzz und tools

[Issue #169](https://github.com/ingen63/VivaPlan/issues/169) · geschlossen · Umsetzung: da91d7b, 190c6b4

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #169/AK1 | `tools/check-english.sh` meldet für `crates/cli/`, `fuzz/`, `tools/` keine Treffer; die Pfade sind aus der Übergangsliste entfernt. | ☑ | **gilt** | [test_check_english.py](../tools/test_check_english.py) «test_ak4_transition_list_holds_unconverted_paths»; `tools/check-english.sh --all crates/cli fuzz tools` | 19.11.2026 ok: keine Treffer in 65 Dateien; Übergangsliste nur noch Engine, App, e2e |
| #169/AK2 | Bezeichner und Testnamen folgen dem Glossar; Kriterien-Bezüge bleiben erhalten (`#27/AK2`, `issue_27_ak2_…`, `AK2` im Testnamen innerhalb von `test.describe("… (#27)")`). | ☑ | **gilt** | kein Test (Vergleich der Kennungen vor und nach der Umstellung) | 19.11.2026 ok: `#Nr/AKn`, `issue_Nr_akn`, `test_akN` in den drei Pfaden vor 0bfc0e7 und nach 190c6b4 identisch |
| #169/AK3 | Keine Verhaltensänderung: `make test-quiet` grün, Golden-Dateien unverändert, Zeilenabdeckung je Paket nicht gesunken. | ☑ | **gilt** | `make test-quiet` (744 Tests), Python-Tests in `tools/` (27) und `tools/report` (20) | 19.11.2026 ok: alles grün, Golden-Dateien unverändert; im Produktivcode der Kommandozeile nur Kommentare geändert, Abdeckung `vivaplan-cli` 95,8 % |
| #169/AK4 | Oberflächentexte, Meldungstexte und Dateischlüssel der `.rplan`-Datei sind unverändert (bleiben deutsch bis #62). | ☑ | **gilt** | kein Test (Durchsicht des Diffs) | 19.11.2026 ok: keine Ausgabe der Kommandozeile und keine Meldung der Skripte geändert; nur Testdaten und Panik-Meldungen in Tests englisch |

### #170 Mehrsprachigkeit: Texttabellen, Sprachwahl und Menüs

[Issue #170](https://github.com/ingen63/VivaPlan/issues/170) · geschlossen · Umsetzung: 8299d5b (Meilensteine), f361107 (Texttabellen, Sprachwahl), 36ee629 (Menüs, Dialoge)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #170/AK1 | Alle sichtbaren Texte von Oberfläche, Tastenkürzel-Übersicht, Grafiken, Zeitleiste und nativen Menüs (inkl. Dateidialoge) kommen aus der Texttabelle; ein Test findet keine deutschen Texte mehr ausserhalb der Tabelle. | ☑ | **gilt** | [check_ui_texts.py](../tools/check_ui_texts.py) (`make ui-texts`, `make test-quiet`, Job `english`), [test_check_ui_texts.py](../tools/test_check_ui_texts.py); [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «AK1» | 19.11.2026 ok: keine Treffer; Pseudo-Sprache zeigt alle Beschriftungen der Editor-Abschnitte aus der Tabelle; alte und neue Oberfläche byte-gleich (Vergleich aller Ansichten). Texte des Rechenkerns (#171/#176) und Hilfe-Markdown (#173) nicht Teil dieses Schritts |
| #170/AK2 | Beim ersten Start gilt die Sprache des Betriebssystems unabhängig von der Region; ist sie keine von de/en/fr/it oder fehlt die Tabelle noch, gilt Deutsch. | ☑ | **gilt** | [i18n.rs](../crates/app/src/i18n.rs) «issue_170_ak2_…», [main.rs](../crates/app/src/main.rs) «issue_170_ak2_…», [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «AK2» | 19.11.2026 ok |
| #170/AK3 | In den Einstellungen gibt es die Auswahl «Sprache», angeboten werden nur vorhandene Sprachen; lokal gespeichert, wirkt sofort, auch auf die nativen Menüs. | ☑ | **gilt** | [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «AK3, AK9», [main.rs](../crates/app/src/main.rs) «issue_170_ak9_ui_reports_language_and_choice» | 19.11.2026 ok; Neuaufbau des nativen Menüs im Code und im Rauchtest der App geprüft, nicht visuell |
| #170/AK4 | Vollständigkeitstest: jeder Schlüssel der deutschen Tabelle in jeder anderen Tabelle vorhanden und nicht leer, keine überzähligen Schlüssel. | ☑ | **gilt** | [i18n.rs](../crates/app/src/i18n.rs) «issue_170_ak4_tables_are_complete», «every_language_has_a_compiled_table» | 19.11.2026 ok (nur `de.json`; weitere Tabellen werden automatisch erfasst, auch Platzhalter) |
| #170/AK5 | Zahlen, Beträge, Prozente und Daten bleiben in jeder Sprache im Schweizer Format, in Anzeige, Eingabe und Export; ein Test prüft das je Sprache. | ☑ | **gilt** | [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «AK5», «AK6, AK8» | 19.11.2026 ok (de und Pseudo-Sprache) |
| #170/AK6 | Eine gespeicherte Planung lässt sich in jeder Sprache öffnen und unverändert speichern; die Datei enthält keine sprachabhängigen Texte. | ☑ | **gilt** | [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «AK6, AK8» | 19.11.2026 ok |
| #170/AK7 | Die bestehenden Oberflächentests starten ausdrücklich mit Deutsch; ein Test startet mit einer anderen Systemsprache und prüft den Rückfall auf Deutsch. Hilfe, `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt** | [helpers.js](../e2e/helpers.js) (`lang="de-CH"`), [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «AK7» | 19.11.2026 ok; Hilfe, BESCHREIBUNG, ENTWICKLUNG, TESTS nachgeführt |
| #170/AK8 | Feste Meilensteine zeigen ihren Namen aus der Texttabelle; neue Dateien speichern keinen `name`; `upgrade.rs` entfernt die deutschen Standardnamen, ein abweichender Name bleibt; Kennung `vivaplan/1`. | ☑ | **gilt** | [upgrade.rs](../crates/engine/src/upgrade.rs) «issue_170_ak8_…», [milestones_limits.rs](../crates/engine/tests/milestones_limits.rs) «issue_170_ak8_…», [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «AK6, AK8», «AK8» | 19.11.2026 ok; Golden-Dateien unverändert; «Pensionierung» wird wie seit #51 zu «Bezug Pensionskasse» und dann entfernt |
| #170/AK9 | Sprachumschalter im Menü «Ansicht → Sprache» mit Häkchen; Menü und Einstellungen zeigen dieselbe Wahl; Wechsel lässt Planung, Änderungen und Ansicht unverändert. | ☑ | **gilt** | [main.rs](../crates/app/src/main.rs) «issue_170_ak9_language_menu_entries», [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «AK3, AK9», «AK9» | 19.11.2026 ok; natives Menü nicht visuell geprüft |

### #171 Mehrsprachigkeit: Meldungscodes des Rechenkerns

[Issue #171](https://github.com/ingen63/VivaPlan/issues/171) · geschlossen · Umsetzung: 5c01f7b (Rechenkern), d173d62 (Oberfläche)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #171/AK1 | Jede Meldung trägt `code` (stabil, englisch, z. B. `pillar3a_over_max`) und `params` (Zahlen und Namen als Werte, nicht als formatierter Text). | ☑ | **gilt** | [message.rs](../crates/engine/src/message.rs) «issue_171_codes_and_templates_are_valid», «issue_171_message_serializes_code_params_text», [validate.rs](../crates/engine/src/validate.rs) «issue_171_ak1_issue_serializes_code_and_params», [devserver.rs](../crates/cli/src/devserver.rs) | 19.11.2026 ok: 339 Codes; Prüfmeldungen, Ereignisse, Gründe, Laufbezeichnungen, Vorschau-Basis, Hinweise von upgrade/Import, Fehler (auch Befehle der App). Beschreibungen der Zeitreihen und Katalog der Optimierung bewusst ohne Code (keine Meldungen) |
| #171/AK2 | Die Oberfläche zeigt Meldungen aus der Texttabelle anhand von `code` und `params`, formatiert Zahlen im Schweizer Format; ein unbekannter Code zeigt den mitgelieferten `text`. | ☑ | **gilt** | [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «Messages of the engine (#171)» «AK2», [engine-messages.test.js](../e2e/unit/engine-messages.test.js) «#171/AK2» | 19.11.2026 ok (Pseudo-Sprache); Desktop-App nicht von Hand geprüft |
| #171/AK3 | Test: Für jeden Code ergibt die deutsche Tabelle mit den Parametern genau den deutschen `text` des Rechenkerns (Beispiele je Code). | ☑ | **gilt** | [message.rs](../crates/engine/src/message.rs) «issue_171_ak3_examples_file_is_current», [i18n.rs](../crates/app/src/i18n.rs) «issue_171_ak3_engine_codes_render_the_engine_text», [engine-messages.test.js](../e2e/unit/engine-messages.test.js) «#171/AK3» | 19.11.2026 ok |
| #171/AK4 | Die Berechnung ist unverändert: Golden-Dateien unverändert (der Golden-Vergleich ignoriert `code`/`params`), Rust-Tests je Meldungsart. | ☑ | **gilt** | [golden.rs](../crates/engine/tests/golden.rs) «golden_masters» (`without_codes`), bestehende Texttests je Meldungsart | 19.11.2026 ok: Golden-Dateien ohne Änderung, 762 Rust-Tests grün; Laufzeit gleich oder schneller |
| #171/AK5 | Spezifikation und `docs/ENTWICKLUNG.md` beschreiben Codes und Parameter. | ☑ | **gilt** | – | 19.11.2026 ok: ENTWICKLUNG.md «Meldungscodes des Rechenkerns (#171)», Pflichteingaben.md (Durchsicht) |

### #172 Oberfläche auf Englisch

[Issue #172](https://github.com/ingen63/VivaPlan/issues/172) · geschlossen · Umsetzung: 4a1d98f

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #172/AK1 | Die Tabelle `en` ist vollständig (Vollständigkeitstest aus #170 grün); «Englisch» erscheint in der Auswahl «Sprache». | ☑ | **gilt** | [i18n.rs](../crates/app/src/i18n.rs) «issue_170_ak4_tables_are_complete», «every_language_has_a_compiled_table», [main.rs](../crates/app/src/main.rs) «issue_172_ak3_menus_and_dialogs_in_english», [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «English user interface (#172)» «AK1, AK3» | 19.11.2026 ok: 1641 Schlüssel wie `de`, gleiche Platzhalter; «English» in Einstellungen und «Ansicht → Sprache» |
| #172/AK2 | Fachbegriffe folgen dem Glossar (`docs/GLOSSAR.md`, Abschnitt Englisch): englische Umschreibung mit Schweizer Kürzel in Klammern. | ☑ | **gilt** | kein Test (Durchsicht) | 19.11.2026 ok: Abschnitt «Englisch (Oberflächenbegriffe)» mit Regeln und rund 100 Begriffen; Tabelle danach geschrieben (volle Form mit Kürzel in Überschriften, Kurzform in Beschriftungen) |
| #172/AK3 | Oberflächentest in Englisch: Start mit Systemsprache `en`, Stichproben in Editor, Ergebnissen, Einstellungen, Menüs und Meldungen; Zahlen im Schweizer Format. | ☑ | **gilt** | [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «English user interface (#172)» «AK1, AK3», [main.rs](../crates/app/src/main.rs) «issue_172_ak3_menus_and_dialogs_in_english» | 19.11.2026 ok: Playwright (Systemsprache en-US) und Rust (native Menüs, Dateidialoge); Desktop-App nicht von Hand geprüft |
| #172/AK4 | Die Texte sind fachlich geprüft (Entwurf durch Claude, Prüfung durch den Maintainer); Inhalte der Planung (Namen) bleiben unübersetzt. | ☑ | **gilt** | – | 21.11.2026 ok: fachliche Prüfung in #182 (Projektinhaber, Entscheide zu allen Befunden der Vorprüfung #181 und von Gemini, Korrekturen in 2f31420 und 276efa7); Namen der Planung werden nicht übersetzt (Daten der Datei) |
| #172/AK5 | `BESCHREIBUNG.md` und `README.md` nennen die Sprache. | ☑ | **gilt** | kein Test (Durchsicht) | 19.11.2026 ok: BESCHREIBUNG «Sprache», README «Deutsch und Englisch», dazu ENTWICKLUNG und Hilfe (Einstellungen) |

### #173 Hilfe auf Englisch

[Issue #173](https://github.com/ingen63/VivaPlan/issues/173) · geschlossen · Umsetzung: 89ff7e8 … 7021c0c

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #173/AK1 | Je Sprache ein eigener Ordner (`hilfe/de/`, `hilfe/en/`); jede deutsche Datei hat eine englische Entsprechung mit denselben Abschnitten und Sprungmarken (Test). | ☑ | **gilt** | [help.rs](../crates/app/tests/help.rs) «issue_173_ak1_every_language_has_the_files_topics_and_anchors_of_the_german_help», [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «Help in the chosen language (#173)» (`Help.check()`), `tools/ui-smoke.sh` `#helpcheck` | 20.11.2026 ok: 7 Dateien je Sprache, gleiche Themen in gleicher Reihenfolge, Unterüberschriften, Verweise und Stichwortzeilen; Test erfasst jeden Sprachordner (fr, it später) |
| #173/AK2 | Hilfe, kontextbezogene Hilfe (F1) und Suche zeigen die Hilfe der gewählten Sprache; fehlt eine Sprache, die deutsche. | ☑ | **gilt** | [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «Help in the chosen language (#173)» «AK2» (3 Tests) | 20.11.2026 ok: Playwright (Handbuch, Suche, Sprachwechsel bei offener Hilfe, F1 im Feld und im Abschnitt, ⓘ, Rückfall je Kapitel und für eine Sprache ohne Hilfe); Desktop-App nicht von Hand geprüft |
| #173/AK3 | Fachbegriffe wie im Glossar; Bildschirmbegriffe stimmen mit der englischen Oberfläche überein (Test: in der Hilfe in «…» zitierte Beschriftungen kommen in der Tabelle `en` vor). | ☑ | **gilt** | [help.rs](../crates/app/tests/help.rs) «issue_173_ak3_quoted_screen_labels_are_in_the_text_table», «issue_173_ak3_label_matching» | 20.11.2026 ok: rund 130 in “…” zitierte Beschriftungen stehen in `en.json` (Gegenprobe: geänderte Beschriftung lässt den Test fehlschlagen); andere Zitate in ‘…’; Glossar per Durchsicht |
| #173/AK4 | Fachlich geprüft (Entwurf durch Claude, Prüfung durch den Maintainer). | ☑ | **gilt** | – | 21.11.2026 ok: fachliche Prüfung in #182 (Projektinhaber, Entscheide zu allen Befunden der Vorprüfung #181 und von Gemini, Korrekturen in 2f31420 und 276efa7); Namen der Planung werden nicht übersetzt (Daten der Datei) |

### #174 Oberfläche und Hilfe auf Französisch

[Issue #174](https://github.com/ingen63/VivaPlan/issues/174) · geschlossen · Umsetzung: 0a87619 … 0e57c46

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #174/AK1 | Die Tabelle `fr` ist vollständig (Vollständigkeitstest aus #170 grün); «Französisch» erscheint in der Auswahl «Sprache». | ☑ | **gilt** | [i18n.rs](../crates/app/src/i18n.rs) «issue_170_ak4_tables_are_complete», «every_language_has_a_compiled_table», [main.rs](../crates/app/src/main.rs) «issue_174_ak3_menus_and_dialogs_in_french», [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «French user interface (#174)» «AK1, AK3» | 20.11.2026 ok: 1709 Schlüssel wie `de` (inkl. #171, #177), gleiche Platzhalter; «Français» in Einstellungen und «Ansicht → Sprache» |
| #174/AK2 | Fachbegriffe folgen dem Glossar (`docs/GLOSSAR.md`, Abschnitt Französisch): Schweizer Amtsbegriffe (AVS, LPP, caisse de pension, pilier 3a). | ☑ | **gilt** | kein Test (Durchsicht); Hilfe-Beschriftungen in «…»: [help.rs](../crates/app/tests/help.rs) «issue_173_ak3_quoted_screen_labels_are_in_the_text_table» | 20.11.2026 ok: Abschnitt «Französisch (Oberflächenbegriffe)» mit Regeln und rund 110 Begriffen; Tabelle und Hilfe danach geschrieben |
| #174/AK3 | Oberflächentest in Französisch: Start mit Systemsprache `fr`, Stichproben von Texten in Editor, Ergebnissen, Einstellungen, Menüs und Meldungen; Zahlen im Schweizer Format. | ☑ | **gilt** | [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «French user interface (#174)» «AK1, AK3», «Names of the optimization catalog and of the time series (#177)», [main.rs](../crates/app/src/main.rs) «issue_174_ak3_menus_and_dialogs_in_french» | 20.11.2026 ok: Playwright (Systemsprache fr-CH, auch Hilfe und F1) und Rust (native Menüs, Dateidialoge); Desktop-App nicht von Hand geprüft |
| #174/AK4 | Die Texte sind fachlich geprüft (Entwurf durch Claude, Prüfung durch den Maintainer); Inhalte der Planung (Namen) bleiben unübersetzt. | ☑ | **gilt** | – | 21.11.2026 ok: fachliche Prüfung in #182 (Projektinhaber, Entscheide zu allen Befunden der Vorprüfung #181 und von Gemini, Korrekturen in 2f31420 und 276efa7); Namen der Planung werden nicht übersetzt (Daten der Datei) |
| #174/AK5 | `BESCHREIBUNG.md` und `README.md` nennen die Sprache. | ☑ | **gilt** | kein Test (Durchsicht) | 20.11.2026 ok: BESCHREIBUNG «Sprache», README «Deutsch, Englisch und Französisch», dazu ENTWICKLUNG und Hilfe (Sprachwahl) |

### #175 Oberfläche und Hilfe auf Italienisch

[Issue #175](https://github.com/ingen63/VivaPlan/issues/175) · geschlossen · Umsetzung: 4c0af89 … 50f6aec

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #175/AK1 | Die Tabelle `it` ist vollständig (Vollständigkeitstest aus #170 grün); «Italienisch» erscheint in der Auswahl «Sprache». | ☑ | **gilt** | [i18n.rs](../crates/app/src/i18n.rs) «issue_170_ak4_tables_are_complete», «every_language_has_a_compiled_table», [main.rs](../crates/app/src/main.rs) «issue_175_ak3_menus_and_dialogs_in_italian», [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «Italian user interface (#175)» «AK1, AK3» | 20.11.2026 ok: 1709 Schlüssel wie `de` (inkl. #171, #177), gleiche Platzhalter; «Italiano» in Einstellungen und «Ansicht → Sprache» |
| #175/AK2 | Fachbegriffe folgen dem Glossar (`docs/GLOSSAR.md`, Abschnitt Italienisch): Schweizer Amtsbegriffe (AVS, LPP, cassa pensione, pilastro 3a). | ☑ | **gilt** | kein Test (Durchsicht); Hilfe-Beschriftungen in «…»: [help.rs](../crates/app/tests/help.rs) «issue_173_ak3_quoted_screen_labels_are_in_the_text_table» | 20.11.2026 ok: Abschnitt «Italienisch (Oberflächenbegriffe)» mit Regeln und rund 110 Begriffen; Tabelle und Hilfe danach geschrieben |
| #175/AK3 | Oberflächentest in Italienisch: Start mit Systemsprache `it`, Stichproben von Texten in Editor, Ergebnissen, Einstellungen, Menüs und Meldungen; Zahlen im Schweizer Format. | ☑ | **gilt** | [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «Italian user interface (#175)» «AK1, AK3», «Names of the optimization catalog and of the time series (#177)», [main.rs](../crates/app/src/main.rs) «issue_175_ak3_menus_and_dialogs_in_italian» | 20.11.2026 ok: Playwright (Systemsprache it-CH, auch Hilfe und F1) und Rust (native Menüs, Dateidialoge); Desktop-App nicht von Hand geprüft |
| #175/AK4 | Die Texte sind fachlich geprüft (Entwurf durch Claude, Prüfung durch den Maintainer); Inhalte der Planung (Namen) bleiben unübersetzt. | ☑ | **gilt** | – | 21.11.2026 ok: fachliche Prüfung in #182 (Projektinhaber, Entscheide zu allen Befunden der Vorprüfung #181 und von Gemini, Korrekturen in 2f31420 und 276efa7); Namen der Planung werden nicht übersetzt (Daten der Datei) |
| #175/AK5 | `BESCHREIBUNG.md` und `README.md` nennen die Sprache. | ☑ | **gilt** | kein Test (Durchsicht) | 20.11.2026 ok: BESCHREIBUNG «Sprache», README «Deutsch, Englisch, Französisch und Italienisch», dazu ENTWICKLUNG und Hilfe (Sprachwahl) |

### #176 Texte des Rechenkerns, Golden-Dateien und Kommandozeile auf Englisch

[Issue #176](https://github.com/ingen63/VivaPlan/issues/176) · geschlossen · Umsetzung: 14c7baa (Rechenkern), 6983f34 (Golden-Dateien), 903fbf0 (Kommandozeile, Doku)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #176/AK1 | Alle Texte des Rechenkerns sind englisch; Test wie #171/AK3, nun gegen die Tabelle `en`. | ☑ | **gilt** | [i18n.rs](../crates/app/src/i18n.rs) «issue_171_ak3_engine_codes_render_the_engine_text», [engine-messages.test.js](../e2e/unit/engine-messages.test.js) «#176/AK1» | 19.11.2026 ok: 339 Vorlagen und 4 Upgrade-Hinweise = `en.json`; Namen aus Optimierungskatalog und Zeitreihen (Daten, keine Meldungen) noch deutsch |
| #176/AK2 | Golden-Dateien neu erzeugt; der Diff enthält nur Textfelder, alle Zahlenwerte sind unverändert (Test oder Prüfskript vergleicht … | ☑ | **gilt** | [golden_numbers.py](../tools/golden_numbers.py) mit [Tests](../tools/test_golden_numbers.py), `make golden-numbers BASE=6983f34~1` | 19.11.2026 ok: 10 Dateien, 62'443 Zahlen, Schlüssel, Listenlängen und Monate unverändert; nur Textfelder geändert |
| #176/AK3 | Die Kommandozeile gibt Meldungen und Ereignisse englisch aus; ihre Tests sind angepasst. | ☑ | **gilt** | [cli.rs](../crates/cli/tests/cli.rs), Tests in `crates/cli/src/optimize.rs`, `devserver.rs` | 19.11.2026 ok: 26 Tests grün, Stichprobe `run --events` |
| #176/AK4 | Die Oberfläche zeigt in jeder Sprache weiterhin die übersetzten Texte (keine englischen Resttexte in de/fr/it). | ☑ | **gilt** | [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «#176/AK4» | 19.11.2026 ok für de (mit Gegenprobe); fr/it noch nicht vorhanden (#174, #175), gleicher Weg über Codes |
| #176/AK5 | `BESCHREIBUNG.md`, `README.md` und Spezifikation zur Sprache der Kommandozeile nachgeführt. | ☑ | **gilt** | kein Test (Durchsicht) | 19.11.2026 ok: BESCHREIBUNG, README, ENTWICKLUNG, Hilfe (Kommandozeile, Optimierung) |

### #177 Namen aus dem Katalog der Optimierung und der Zeitreihen übersetzen

[Issue #177](https://github.com/ingen63/VivaPlan/issues/177) · geschlossen · Umsetzung: 81c03f5

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #177/AK1 | Parameter der Optimierung, Strategienamen, Namen der Anlagestrategien und Indizes und die Beschreibungen der eingebauten … | ☑ | **gilt** | [i18n.rs](../crates/app/src/i18n.rs) «issue_170_ak4_tables_are_complete», «issue_177_ak1_stored_german_series_texts_are_the_table_de», [series_text.rs](../crates/engine/src/series_text.rs) «issue_177_ak1_*» | 20.11.2026 ok: 67 Schlüssel `engine.*` in de/en; deutsche Texte älterer Bibliotheksdateien werden erkannt |
| #177/AK2 | Der Text des Rechenkerns für diese Namen ist englisch, die Namen sind dieselben wie in der Tabelle `en` (Test wie #176/AK1). … | ☑ | **gilt** | [i18n.rs](../crates/app/src/i18n.rs) «issue_171_ak3_engine_codes_render_the_engine_text», [optimize_ranges.rs](../crates/engine/tests/optimize_ranges.rs) «invalid_ranges_are_reported» | 20.11.2026 ok: «Guardrails, Initial rate: the range from 8 % to 2 % is invalid …»; englische Namen fachlich noch nicht geprüft (wie #172) |
| #177/AK3 | In Deutsch zeigt die Oberfläche die bisherigen deutschen Namen, in Englisch die englischen; Oberflächentest mit Stichproben … | ☑ | **gilt** | [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «#177 AK3» | 20.11.2026 ok für de und en (Parametertabelle, Einheiten, Bandbreiten-Meldung, `#strategyClip`, Kurzinfo); fr/it mit #174/#175 |
| #177/AK4 | Die Berechnung ist unverändert: Ändern sich die Golden-Dateien, betrifft der Unterschied nur Textfelder (`make golden-numbers`), … | ☑ | **gilt** | `make golden-numbers` | 20.11.2026 ok: Golden-Dateien unverändert; nur Kommentarzeilen der mitgelieferten CSV-Dateien englisch (Begründung im Issue) |
| #177/AK5 | Gespeicherte Planungen und Ergebnisse bleiben lesbar; Schlüssel in der Datei (z. B. `optimization.ranges`, Strategie-Kennungen) … | ☑ | **gilt** | Durchsicht Diff, `engineText` mit Text | 20.11.2026 ok: Schlüssel unverändert; Namen als Text in älteren Ergebnissen werden unverändert angezeigt |

### #179 Fachglossar in vier Sprachen

[Issue #179](https://github.com/ingen63/VivaPlan/issues/179) · geschlossen · Umsetzung: 3906724, d0833d8

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #179/AK1 | `docs/GLOSSAR.md` hat einen Abschnitt «Übersetzungen» mit einer Tabelle DE, EN, FR, IT, Quelle, verbotene Varianten. | ☑ | **gilt** | [test_glossary.py](../tools/test_glossary.py) «test_issue_179_ak1_section_has_both_tables_with_six_columns» | 20.11.2026 ok: Tabellen «Kürzel» und «Fachbegriffe» mit sechs Spalten |
| #179/AK2 | Die Tabelle enthält alle Fachbegriffe aus `de.json` und der deutschen Hilfe, mindestens die genannten. | ☑ | **gilt** | [test_glossary.py](../tools/test_glossary.py) «test_issue_179_ak2_minimum_terms_present», «test_issue_179_ak2_german_terms_unique» | 20.11.2026 ok: 97 Fachbegriffe, Mindestliste vollständig; Auswahl von Hand (Durchsicht in #181) |
| #179/AK3 | Begriffe mit amtlicher Übersetzung nennen die Quelle; Begriffe ohne amtliche Quelle sind markiert. | ☑ | **gilt** | [test_glossary.py](../tools/test_glossary.py) «test_issue_179_ak3_every_source_is_known_or_marked», «test_issue_179_ak3_official_sources_named_in_legend» | 20.11.2026 ok: 30 Zeilen mit Quelle (fedlex, Merkblätter AHV/IV), übrige markiert; TERMDAT nicht erreichbar |
| #179/AK4 | Unübersetzte Kürzel stehen in einer eigenen Liste mit ihrer Form je Sprache. | ☑ | **gilt** | [test_glossary.py](../tools/test_glossary.py) «test_issue_179_ak4_abbreviations_have_form_per_language» | 20.11.2026 ok: 11 Kürzel, z. B. AHV/AHV/AVS/AVS |
| #179/AK5 | Die Tabelle lässt sich maschinell lesen: Das Skript aus #180 liest sie ohne Fehler. | ☑ | **gilt** | [test_glossary.py](../tools/test_glossary.py) «test_issue_179_ak5_cli_reads_glossary»; [test_check_i18n.py](../tools/test_check_i18n.py) «test_issue_179_ak5_issue_180_ak2_the_real_glossary_is_read» | 20.11.2026 ok: `tools/check_i18n.py` (#180) liest das Glossar mit `glossary.parse` fehlerfrei, `make i18n-check` grün |

### #180 Automatische Prüfung der Übersetzungen

[Issue #180](https://github.com/ingen63/VivaPlan/issues/180) · geschlossen · Umsetzung: d0833d8

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #180/AK1 | `make i18n-check` prüft `i18n/{en,fr,it}.json` und `hilfe/{en,fr,it}/` gegen das Deutsche und endet bei einem Befund mit Fehler; jeder Befund nennt Datei, Schlüssel oder Zeile und Grund. | ☑ | **gilt** | [test_check_i18n.py](../tools/test_check_i18n.py) «test_issue_180_ak1_clean_files_pass_and_a_finding_names_file_key_and_reason», «…_missing_and_extra_keys_are_findings», «…_the_repository_is_clean» | 20.11.2026 ok: Exit 1 mit Datei, Schlüssel bzw. Zeile/Abschnitt, Prüfung, Grund; Bestand grün (22 Ausnahmen) |
| #180/AK2 | Begriffe: Glossarbegriff des deutschen Textes muss im Zieltext in der festgelegten Form stehen, verbotene Variante ist ein Befund; Hilfe je Abschnitt. | ☑ | **gilt** | [test_check_i18n.py](../tools/test_check_i18n.py) «test_issue_180_ak2_missing_glossary_term_is_a_finding», «…_forbidden_variant_and_abbreviation_as_whole_word», «…_help_is_compared_section_by_section» | 20.11.2026 ok: Endungen bis 3 Buchstaben in den Zielsprachen erlaubt (GLOSSAR.md) |
| #180/AK3 | Platzhalter und Werte: Platzhalter, Zahlen, CHF, % und Jahreszahlen stimmen überein; Hilfe je Abschnitt. | ☑ | **gilt** | [test_check_i18n.py](../tools/test_check_i18n.py) «test_issue_180_ak3_placeholders_numbers_chf_and_percent», «…_help_numbers_of_a_section_must_appear_in_the_target_section» | 20.11.2026 ok: Schweizer Schreibweisen (1'000, 1 000, 6,8/6.8) gleichwertig |
| #180/AK4 | Reste: Zieltext gleich dem deutschen ist ein Befund, ausser in der Liste erlaubter Ausnahmen. | ☑ | **gilt** | [test_check_i18n.py](../tools/test_check_i18n.py) «test_issue_180_ak4_text_identical_to_german_unless_allowed», «…_help_line_copied_from_german» | 20.11.2026 ok: 36 erlaubte Wörter in 4 Gruppen |
| #180/AK5 | Hilfe: Zahl und Reihenfolge der Überschriften, Anker, Links und Tabellenzeilen je Datei gleich. | ☑ | **gilt** | [test_check_i18n.py](../tools/test_check_i18n.py) «test_issue_180_ak5_help_headings_links_and_table_rows_match» | 20.11.2026 ok; am Commit-Stand ein Befund in `hilfe/en/60-referenz.md` (Link-Reihenfolge), behoben durch die noch nicht committete Umsortierung aus #182 |
| #180/AK6 | Anrede und Typografie: keine Du-Form in FR/IT, keine Anführungszeichen „…“. | ☑ | **gilt** | [test_check_i18n.py](../tools/test_check_i18n.py) «test_issue_180_ak6_informal_address_and_german_quotes» | 20.11.2026 ok |
| #180/AK7 | Länge: mehr als 40 % länger als Deutsch ist ein Hinweis, kein Fehler. | ☑ | **gilt** | [test_check_i18n.py](../tools/test_check_i18n.py) «test_issue_180_ak7_long_text_is_a_hint_not_a_finding» | 20.11.2026 ok: 113 Hinweise (`--hints`), deutsche Texte ab 20 Zeichen |
| #180/AK8 | Eigene Tests mit einem Fall je Prüfung; läuft in `make test-quiet` und in `test-quality.yml`. | ☑ | **gilt** | [test_check_i18n.py](../tools/test_check_i18n.py) «test_issue_180_ak8_exceptions_silence_a_finding_and_stale_ones_are_reported» und alle obigen | 20.11.2026 ok: 15 Tests; Makefile und Job `english` (CI nicht gelaufen, kein Push) |
| #180/AK9 | Keine neue Abhängigkeit (nur Standardbibliothek). | ☑ | **gilt** | kein Test (Durchsicht der Importe) | 20.11.2026 ok: argparse, json, pathlib, re, sys, `tools/glossary.py` |

### #181 Vorprüfung der Übersetzungen durch Claude

[Issue #181](https://github.com/ingen63/VivaPlan/issues/181) · geschlossen · Umsetzung: kein Commit (Ergebnisse nur in `local/i18n-review/`, Kommentar im Issue)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #181/AK1 | Je Sprache liegt `local/i18n-review/<sprache>.csv` mit den Spalten Ort, DE, Ist, Vorschlag, Art, Schwere. | ☑ | **gilt** | kein Test (Durchsicht der Dateien) | 20.11.2026 ok: `en.csv` 55, `fr.csv` 58, `it.csv` 66 Befunde; zusätzliche Spalte Begründung |
| #181/AK2 | Die Liste deckt alle 1'709 Schlüssel und alle 7 Hilfedateien ab; Zusammenfassung je Sprache mit geprüften Gruppen und Befunden je Art und Schwere. | ☑ | **gilt** | kein Test (Durchsicht der Protokolle) | 20.11.2026 ok: je Sprache 1'709 Schlüssel in 96 Gruppen, 7 Dateien mit 101 Abschnitten; `<sprache>-zusammenfassung.md` |
| #181/AK3 | «Hoch» bedeutet anderer fachlicher Sinn: falscher Begriff, Wert, falsche Bedingung oder verneinte Aussage. | ☑ | **gilt** | kein Test (Durchsicht) | 20.11.2026 ok nach einer Nachbesserung: 7 Befunde mit falschem Fachbegriff von mittel auf hoch; hoch EN 2, FR 3, IT 3 |
| #181/AK4 | Die Vorprüfung ändert keine Übersetzung und committet nichts. | ☑ | **gilt** | kein Test (Prüfsummen vorher/nachher) | 20.11.2026 ok: Prüfsummen von `i18n/*.json` und `hilfe/*/*.md` gleich, HEAD unverändert |
| #181/AK5 | Kein Befund enthält private Daten. | ☑ | **gilt** | kein Test (Suche in den Befundlisten) | 20.11.2026 ok: nur App-Texte mit erfundenen Beispielen |

### #182 Fachliche Prüfung der Übersetzungen durch Muttersprachler

[Issue #182](https://github.com/ingen63/VivaPlan/issues/182) · geschlossen · Umsetzung: 4221280, 2f31420, d9da4a7, die Nacharbeit zu AK5 und a9359be (AK4 unter Linux und Windows)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #182/AK1 | Gemeinsame Prüftabelle `local/i18n-review/tabelle.csv` mit allen Texten und Hilfeabsätzen, Spalten nach Vorgabe, per Skript erzeugt, direkt in Excel lesbar. | ☑ | **gilt** (geändert per Kommentar vom 20.11.2026: Tabelle statt Bildergalerie) | [test_i18n_review_table.py](../tools/test_i18n_review_table.py) | 21.11.2026 ok: 2347 Zeilen (1709 Schlüssel, 638 Hilfeblöcke), 675 kritisch, UTF-8 mit BOM und `;`; 6 Tests grün |
| #182/AK2 | Für jeden Befund aus `local/i18n-review/<sprache>.csv` ist der Entscheid vermerkt. | ☑ | **gilt** | kein Test (Auswertung `entscheidliste.csv`) | 21.11.2026 ok: 179 Befunde in 270 Zeilen aufgegangen, jede mit Entscheid (261 übernommen, 5 angepasst, 4 verworfen) |
| #182/AK3 | Die kritischen Texte sind vollständig geprüft (Felder, Meldungen, Haftungsausschluss, `60-referenz.md`, Stichworte). | ☑ | **gilt** | kein Test (Durchsicht der Prüfprotokolle) | 21.11.2026 ok mit Einschränkung: zwei maschinelle Durchgänge (#181, Gemini) und Abnahme durch den Projektinhaber; Gemini Teil 01 ohne Lesebestätigung; keine Muttersprachler |
| #182/AK4 | Playwright-Test: jede Ansicht, jeder Dialog und jede Hilfeseite in DE, EN, FR, IT ohne abgeschnittenen oder überlappenden Text. | ☑ | **gilt** (geändert per Kommentar vom 20.11.2026) | [ui-textfit.spec.js](../e2e/ui-textfit.spec.js) «Text fits in every language (#182)» «AK4 …» | 21.11.2026 ok: 16 Tests grün; Fenster 1440 × 920, Desktop-App nicht von Hand geprüft. In CI (Linux) danach rot, wieder geöffnet; Nachprüfung 21.11.2026 nach a9359be ok: lokal 16 Tests grün, CI-Lauf 44912644203 `e2e` grün; Gegenprobe ohne Silbentrennung (wie Linux und WebView2) |
| #182/AK5 | Korrekturen in Texttabellen und Hilfe, Begriffe im Glossar, `make i18n-check` grün. | ☑ | **gilt** | [check_i18n.py](../tools/check_i18n.py) (`make i18n-check`), [help.rs](../crates/app/tests/help.rs) «issue_173_ak3_quoted_screen_labels_are_in_the_text_table» | 21.11.2026 ok nach Nacharbeit: Texttabellen (155) und Glossar ok, i18n-check 0 Befunde; Hilfe: Nr. 85 (EN) und 250 (IT) nachgetragen, Nr. 200 (FR) verworfen, weil zusammengesetzte Knopfbeschriftungen nach #173 in ‹…› stehen |
| #182/AK6 | Je Sprache ist im Issue vermerkt, wer geprüft hat (Rolle) und wann. | ☑ | **gilt** | kein Test (Durchsicht) | 21.11.2026 ok: Kommentar vom 20.11.2026 |

### #183 Widersprüche in der deutschen Hilfe (Lebenshaltung, Kombinationen, 3a-Einzahlung)

[Issue #183](https://github.com/ingen63/VivaPlan/issues/183) · geschlossen · Umsetzung: cc1e5ab

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #183/AK1 | Die Hilfe unterscheidet in `10-editor.md` (Lebenshaltung, Immobilien) die Eingabe der Lebenshaltungskosten (inklusive Wohnkosten) vom gerechneten Wert (ohne Wohnanteil), ohne Widerspruch. | ☑ | **gilt** | kein Test (Durchsicht, Abgleich mit `planned_living` in `sim.rs`) | 21.11.2026 ok: beide Stellen in DE, EN, FR, IT; Rechenkern zieht `housing_share` von der Eingabe ab und rechnet mit den simulierten Wohnkosten |
| #183/AK2 | Satz «Höchstens 400 Kombinationen sind erlaubt.» entfernt; einzige Grenze 1'000'000 (`MAX_COMBINATIONS`). | ☑ | **gilt** | [docs.test.js](../e2e/unit/docs.test.js) «#183/AK5: …» | 21.11.2026 ok: in allen vier Sprachen entfernt |
| #183/AK3 | 3a-Einzahlung bei den Leitplanken im Rechenkern nachgeprüft; Formel in `30-entnahme.md` bei fixer Rate und Leitplanken entsprechend; Rust-Test. | ☑ | **gilt** | [pillar3a.rs](../crates/engine/tests/pillar3a.rs) «issue_183_ak3_pillar3a_counts_towards_withdrawal_for_fixed_rate_and_guardrails» | 21.11.2026 ok: beide Strategien über `rule_living`, 3a zählt bei beiden zur Entnahme; Formel der Leitplanken ergänzt; kein Rechenfehler |
| #183/AK4 | Korrekturen in `hilfe/{de,en,fr,it}/`, `make i18n-check` grün. | ☑ | **gilt** | [check_i18n.py](../tools/check_i18n.py) (`make i18n-check`) | 21.11.2026 ok: 0 Befunde |
| #183/AK5 | Ein Test findet in der Hilfe keine Zahl der Kombinationen, die von `MAX_COMBINATIONS` abweicht. | ☑ | **gilt** | [docs.test.js](../e2e/unit/docs.test.js) «#183/AK5: the help names no number of combinations other than MAX_COMBINATIONS» | 21.11.2026 ok: Wert aus `scenarios.rs` gelesen; Gegenprobe mit der alten Fassung findet 400 in allen Sprachen |

### #184 Beispielplanung in der Sprache der Oberfläche

[Issue #184](https://github.com/ingen63/VivaPlan/issues/184) · geschlossen · Umsetzung: 123d4af

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #184/AK1 | Mit der Oberfläche auf EN, FR oder IT öffnen «Neu» und der erste Start die Beispielplanung mit allen Namen in dieser Sprache; kein deutsches Wort bleibt. | ☑ | **gilt** | [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «Example plan in the language of the user interface (#184)» «AK1: …», [util.test.js](../e2e/unit/util.test.js) «#184/AK1: …» | 21.11.2026 ok: FR, IT, EN; «Tranche 1/2» nach Glossar in allen Sprachen gleich; Desktop-App nicht von Hand geprüft |
| #184/AK2 | Die Beispielplanungen der vier Sprachen ergeben dieselben Ergebnisse (nur Namen verschieden); ein Test prüft das. | ☑ | **gilt** | [util.test.js](../e2e/unit/util.test.js) «#184/AK2: …» | 21.11.2026 ok |
| #184/AK3 | Gespeicherte Planungen werden nicht übersetzt; eine gespeicherte Beispielplanung behält in einer anderen Sprache ihre Namen. | ☑ | **gilt** | [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «AK3: a saved plan keeps its names in another language» | 21.11.2026 ok |
| #184/AK4 | Die Namen folgen dem Glossar; die Person heisst landesüblich und ist erfunden. | ☑ | **gilt** | kein Test (Durchsicht `example.*` in `i18n/*.json` gegen `docs/GLOSSAR.md`) | 21.11.2026 ok: Planung → plan, planification, pianificazione; PPE/PPP landesüblich; Jane Smith, Marie Dupont, Maria Rossi |
| #184/AK5 | Hilfe und `docs/BESCHREIBUNG.md` nachgeführt, wo sie die Beispielplanung beschreiben. | ☑ | **gilt** | kein Test (Durchsicht) | 21.11.2026 ok: `hilfe/*/10-editor.md` in allen vier Sprachen, `BESCHREIBUNG.md` |

### #185 Neues App-Icon

[Issue #185](https://github.com/ingen63/VivaPlan/issues/185) · geschlossen · Umsetzung: 85edb19

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #185/AK1 | Die Quelldatei des Icons liegt als SVG im Repository (`crates/app/icons/icon.svg`) und zeigt die Variante «Abendrot» wie oben beschrieben. | ☑ | **gilt** (gilt auch für `icon-small.svg`, Kommentar im Issue) | [test_icons.py](../tools/test_icons.py) «issue_185_ak1_…» | 21.11.2026 ok |
| #185/AK2 | Alle Icon-Dateien aus `crates/app/tauri.conf.json` (32, 128, 256 px, `.icns`, `.ico`) und `icon.png` sind aus dieser SVG-Datei erzeugt; … | ☑ | **gilt** (kleine Grössen aus `icon-small.svg`) | [test_icons.py](../tools/test_icons.py) «issue_185_ak2_…» (2 Tests) | 21.11.2026 ok (`make icons`, zweimal erzeugt gleiche Dateien) |
| #185/AK3 | Das Icon bleibt mit 16 und 32 px erkennbar (Sonne und Bergkette unterscheidbar); ein Bild mit den Grössen 16/32/128/512 px liegt im Issue. | ☑ | **gilt** | kein Test (Vorschau `docs/bilder/icon-vorschau.png`) | 21.11.2026 ok (Urteil von Auge des Projektinhabers zur Vorschau) |
| #185/AK4 | Das Icon wirkt auf hellem und auf dunklem Hintergrund (macOS-Dock, Windows-Taskleiste hell und dunkel). | ☑ | **gilt** | kein Test (Vorschau `docs/bilder/icon-vorschau.png`) | 21.11.2026 ok (Urteil von Auge des Projektinhabers zur Vorschau) |
| #185/AK5 | Kein Schweizerkreuz oder anderes geschütztes Zeichen, keine fremden Marken, keine Schriftart mit Lizenzpflicht. | ☑ | **gilt** | kein Test (Durchsicht der SVG-Dateien; `<text>` prüft «issue_185_ak1_…») | 21.11.2026 ok |
| #185/AK6 | Die Installationspakete (`.github/workflows/pakete.yml`) bauen mit dem neuen Icon. | ☑ | **gilt** | kein Test (Lauf [44913969121](https://github.com/ingen63/VivaPlan/actions/runs/44913969121) von Hand auf a9359be) | 21.11.2026 ok (macOS, Linux, Windows erfolgreich) |

### #186 Beispieldatei der Meldungen unter Windows mit CRLF

[Issue #186](https://github.com/ingen63/VivaPlan/issues/186) · geschlossen · Umsetzung: 8d3b49d

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #186/AK1 | `.gitattributes` checkt `e2e/unit/engine-messages.json` mit LF aus (`git check-attr eol` meldet `lf`). | ☑ | **gilt** | kein Test (`git check-attr eol`, `git ls-files --eol`) | 21.11.2026 ok |
| #186/AK2 | Job `tests-os (windows-latest)` im nächsten Lauf grün. | ☑ | **gilt** | `message::tests::issue_171_ak3_examples_file_is_current` unter Windows | 21.11.2026 ok: Läufe 44907095566 (7676c22) und 44912644203 (a9359be) |

### #187 Nicht erkannte Mutanten der Mehrsprachigkeit und der Meldungen

[Issue #187](https://github.com/ingen63/VivaPlan/issues/187) · geschlossen · Umsetzung: dcf8df2, 7676c22

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #187/AK1 | Tests erkennen die Mutanten in `message.rs` (Gleichheit, `param`, `render`, `number` fix1/fix2, `placeholders`, `bool`-Argument). | ☑ | **gilt** | [message.rs](../crates/engine/src/message.rs) «issue_187_ak1_…» (2 Tests) | 21.11.2026 ok; `render` (fehlender Parameter) und `placeholders` gleichwertig, Block «#187» in `.cargo/mutants.toml` |
| #187/AK2 | Tests erkennen die Mutanten in `model.rs`, `series_text.rs`, `import.rs`, `sim.rs`, `validate.rs`, oder ein Kommentar begründet einen gleichwertigen Mutanten. | ☑ | **gilt** | «issue_187_ak2_…» in [series_text.rs](../crates/engine/src/series_text.rs), [model.rs](../crates/engine/src/model.rs), [import.rs](../crates/engine/src/import.rs) | 21.11.2026 ok; `parts`, `capital_tax` gleichwertig; sechs `validate.rs`-Einträge mit verrutschter Zeile nachgeführt |
| #187/AK3 | Tests erkennen die Mutanten in `crates/app` (`pick`, `startup`, `language_menu`, `first_start_language`) und `devserver::content_type`. | ☑ | **gilt** | [main.rs](../crates/app/src/main.rs) «issue_187_ak3_first_start_language_via_command», [i18n.rs](../crates/app/src/i18n.rs) «startup_language_uses_the_saved_choice_or_the_system», [devserver.rs](../crates/cli/src/devserver.rs) «content_types_follow_the_extension» | 21.11.2026 ok; `pick` gleichwertig, `language_menu` nur im Hauptthread (begründet ausgeschlossen) |
| #187/AK4 | `cargo mutants` auf den betroffenen Dateien meldet keine nicht erkannten Mutanten mehr (ausser begründeten). | ☑ | **gilt** | `mutants-diff` in `test-quality.yml` | 21.11.2026 ok: lokal 44 von 44 erkannt; CI-Lauf 44907095566 1063 Mutanten, 0 nicht erkannt |

### #190 13. AHV-Rente im Dezember

[Issue #190](https://github.com/ingen63/VivaPlan/issues/190) · geschlossen · Umsetzung: 44a0619

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #190/AK1 | Die rechtliche Grundlage (Gesetz, Beginn, Höhe) ist im Issue mit Quelle festgehalten, bevor die Umsetzung beginnt. | ☑ | **gilt** | – | 21.11.2026 ok (Kommentar mit Quellen im Issue) |
| #190/AK2 | Im Dezember jedes Jahres ab 2026 wird zusätzlich zur AHV-Rente des Monats eine 13. Rente in gleicher Höhe ausbezahlt; … | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) «issue_190_ak2_…» | 21.11.2026 ok |
| #190/AK3 | Beginnt die AHV im Laufe eines Jahres, ist die Höhe der 13. Rente in diesem Jahr nach der gesetzlichen Regel entschieden … | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) «issue_190_ak3_…» (2) | 21.11.2026 ok (anteilig, 1'260) |
| #190/AK4 | Für Ehepaare ist entschieden und getestet, ob und wie der Plafond (`ahvCoupleCap`) für die 13. Rente gilt; … | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) «issue_190_ak4_…» (2) | 21.11.2026 ok (2'205 + 945, keine zweite Deckelung) |
| #190/AK5 | Referenztest mit Handrechnung in `crates/engine/tests/reference.rs`; jede Änderung der Golden-Dateien ist im Issue begründet. | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) «issue_190_…», `golden.rs` | 21.11.2026 ok (Golden 02–04, Begründung im Issue) |
| #190/AK6 | Jahreswerte zeigen die 13. Rente im Dezember im AHV-Einkommen. | ☑ | **gilt** | [ui-planung.spec.js](../e2e/ui-planung.spec.js) «AK6» | 21.11.2026 ok |
| #190/AK7 | Hilfe (Abschnitt AHV, vier Sprachen), `docs/BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt** | – | 21.11.2026 ok (Durchsicht) |
| #190/AK8 | Das Eingabefeld «Rente pro Monat bei Bezugsbeginn» zeigt einen Hinweis, welcher Betrag gemeint ist: … | ☑ | **gilt** | [ui-planung.spec.js](../e2e/ui-planung.spec.js) «AK8» | 21.11.2026 ok |
| #190/AK9 | Der Monatsanteil wird auf Rappen, die Summe im Dezember kaufmännisch auf ganze Franken gerundet (Kommentar). | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) «issue_190_ak9_…» | 21.11.2026 ok |
| #190/AK10 | Endet die Simulation vor Dezember, wird für dieses Jahr keine 13. Rente ausbezahlt (Kommentar). | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) «issue_190_ak10_…» | 21.11.2026 ok |
| #190/AK11 | Eine 13. Rente gibt es erst ab Dezember 2026, auch wenn die Simulation früher beginnt (Kommentar). | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) «issue_190_ak11_…» | 21.11.2026 ok |
| #190/AK12 | Beim Öffnen einer älteren Planung erscheint der Hinweis zur 13. Rente einmalig; Test mit einer alten Datei (Kommentar). | ☑ | **gilt** | [upgrade_api.rs](../crates/engine/tests/upgrade_api.rs) «issue_190_ak12_…», [ui-planung.spec.js](../e2e/ui-planung.spec.js) «AK12» | 21.11.2026 ok |

### #191 AHV-Vorbezug und -Aufschub: Kürzung und Zuschlag berechnen

[Issue #191](https://github.com/ingen63/VivaPlan/issues/191) · geschlossen · Umsetzung: e3eed8f, 0477c2e (Nachbesserung Plafond)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #191/AK1 | Die Sätze sind im Issue mit Quelle festgehalten (Merkblatt 3.04, Stand 1.1.2026): … | ☑ | **gilt** | – | 21.11.2026 ok (Kommentar mit Tabellen im Issue) |
| #191/AK2 | Die Sätze stehen an einer Stelle im Rechenkern mit Gültigkeitsjahr und Quelle; ein Test prüft die Tabellenwerte. | ☑ | **gilt** | [ahv.rs](../crates/engine/src/ahv.rs) «issue_191_ak2_…» | 21.11.2026 ok |
| #191/AK3 | Bezug mit 63, Rente 2'520: die App rechnet mit 2'520 − 343 = 2'177.00 pro Monat, auch nach Erreichen von 65. | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) «issue_191_ak3_…» | 21.11.2026 ok |
| #191/AK4 | Bezug 15 Monate vor 65: Kürzung 8,5 % (Tabelle); Rente 2'000 → 1'830.00. | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) «issue_191_ak4_…» | 21.11.2026 ok |
| #191/AK5 | Bezug mit 68, Rente 2'520: Zuschlag 17,1 % (431) → 2'951.00; zwischen 65 und 68 keine AHV-Rente. | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) «issue_191_ak5_…» | 21.11.2026 ok |
| #191/AK6 | Bezug mit 65: Rente unverändert; Golden-Dateien mit AHV-Bezug bei 65 unverändert. | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) «issue_191_ak6_…», `golden.rs` | 21.11.2026 ok (nur Golden 02 geändert: Person 2 Vorbezug, begründet) |
| #191/AK7 | Die Anpassung gilt je Person nach ihrem eigenen Alter; Plafond, Kürzungsbetrag und Zuschlag folgen dem Gesetz (AK11–AK24). | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) «issue_191_ak7_…» | 21.11.2026 ok |
| #191/AK8 | Neben dem Eingabefeld steht immer die effektive monatliche AHV-Rente, bei Ehepaaren zusätzlich die Rente nach Plafond und Kürzung/Zuschlag; … | ☑ | **gilt** | [ui-planung.spec.js](../e2e/ui-planung.spec.js) «AK8» | 21.11.2026 ok (Prozent mit Punkt im Zahlenformat der App) |
| #191/AK9 | Bestehende Planungen: beim Öffnen nicht doppelt gekürzt; Vorgehen entschieden und mit alter Datei getestet (`upgrade.rs`). | ☑ | **gilt** | [upgrade_api.rs](../crates/engine/tests/upgrade_api.rs) «issue_191_ak9_…», [ui-planung.spec.js](../e2e/ui-planung.spec.js) «AK9» | 21.11.2026 ok (gerechnet 2'177 statt 2'177.28, Franken-Rundung) |
| #191/AK10 | Hilfe (Abschnitt AHV, vier Sprachen), `docs/BESCHREIBUNG.md` und Spezifikation sind nachgeführt; … | ☑ | **gilt** | – | 21.11.2026 ok (Durchsicht) |
| #191/AK11 | Beide beziehen mit 65: Plafond ab dem 65. der zweiten Person; je 1'890. | ☑ | **gilt** | [ahv.rs](../crates/engine/src/ahv.rs) «issue_191_ak11_…» | 21.11.2026 ok |
| #191/AK12 | Summe unter dem Plafond: A 1'728, B 1'500. | ☑ | **gilt** | [ahv.rs](../crates/engine/src/ahv.rs) «issue_191_ak12_…» | 21.11.2026 ok |
| #191/AK13 | Nicht verheiratet: kein Plafond; A 2'177, B 2'520. | ☑ | **gilt** | [ahv.rs](../crates/engine/src/ahv.rs) «issue_191_ak13_…», [reference.rs](../crates/engine/tests/reference.rs) «issue_191_ak7_…» | 21.11.2026 ok |
| #191/AK14 | Vorbezug ohne Anspruch des Ehepartners: Kürzung 343; ab Plafond A 1'547, B 1'890. | ☑ | **gilt** | [ahv.rs](../crates/engine/src/ahv.rs) «issue_191_ak14_…» | 21.11.2026 ok |
| #191/AK15 | Ehepartner wird während des Vorbezugs 65: 2'177, dann 1'633; ab 65 Kürzung 300, A 1'590. | ☑ | **gilt** | [ahv.rs](../crates/engine/src/ahv.rs) «issue_191_ak15_…» | 21.11.2026 ok |
| #191/AK16 | Beide gleichzeitig mit 63: je 1'633, auch nach 65. | ☑ | **gilt** | [ahv.rs](../crates/engine/src/ahv.rs) «issue_191_ak16_…» | 21.11.2026 ok |
| #191/AK17 | Aufschub 3 Jahre, Ehepartner bezieht: B ab dem 65. von A 1'890; A ab 68 2'213; Summe 4'103. | ☑ | **gilt** | [ahv.rs](../crates/engine/src/ahv.rs) «issue_191_ak17_…» | 21.11.2026 ok |
| #191/AK18 | Aufschub 1 Jahr ohne Anspruch des Ehepartners: Zuschlag 131, A 2'651; ab Plafond 2'021 und 1'890. | ☑ | **gilt** | [ahv.rs](../crates/engine/src/ahv.rs) «issue_191_ak18_…» | 21.11.2026 ok |
| #191/AK19 | Bezug weniger als ein Jahr nach 65: kein Zuschlag; die Prüfung warnt. | ☑ | **gilt** | [ahv.rs](../crates/engine/src/ahv.rs) «issue_191_ak19_…», [reference.rs](../crates/engine/tests/reference.rs) «issue_191_ak19_…» | 21.11.2026 ok |
| #191/AK20 | Bezug vor 63 oder nach 70 im eigenen Alter: die Prüfung meldet es; höchstens 13,6 % bzw. 31,5 %. | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) «issue_191_ak20_…» | 21.11.2026 ok |
| #191/AK21 | Teuerung: Rente, Plafond, Kürzungsbetrag und Zuschlag steigen im Januar gleich. | ☑ | **gilt** | [ahv.rs](../crates/engine/src/ahv.rs) «issue_191_ak21_…» | 21.11.2026 ok |
| #191/AK22 | Simulationsstart während oder nach Vorbezug/Aufschub: Monate davor mit den Beträgen beim Start. | ☑ | **gilt** | [ahv.rs](../crates/engine/src/ahv.rs) «issue_191_ak22_…» | 21.11.2026 ok |
| #191/AK23 | Eine Person bezieht nie AHV: die andere wird nie plafoniert. | ☑ | **gilt** | [ahv.rs](../crates/engine/src/ahv.rs) «issue_191_ak23_…» | 21.11.2026 ok |
| #191/AK24 | 13. Rente auf den ausbezahlten Renten nach Plafond, Kürzung und Zuschlag (Fall AK14: 3'437). | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) «issue_191_ak7_…» | 21.11.2026 ok |

### #192 Anforderungsbezug: Kennungslisten erkennen und verwaiste Kennungen melden

[Issue #192](https://github.com/ingen63/VivaPlan/issues/192) · geschlossen · Umsetzung: 7e93347

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #192/AK1 | `#98/AK1–AK3` zählt für AK1, AK2 und AK3; `#76/AK1, AK3` für AK1 und AK3; `#85/AK1 und AK2` für beide; `issue_53_ak4_ak5` … | ☑ | **gilt** | [test_report.py](../tools/report/test_report.py) «test_issue_192_ak1_…» | 21.11.2026 ok (433 statt 382 Kriterien mit Test) |
| #192/AK2 | In `e2e/` zählt ein Testname «AK3, AK9» innerhalb von `test.describe("… (#170)")` weiterhin für beide. | ☑ | **gilt** | [test_report.py](../tools/report/test_report.py) «test_issue_192_ak2_…» | 21.11.2026 ok |
| #192/AK3 | Die Übersichtsseite listet verwaiste Kennungen (Issue oder Kriterium existiert nicht) mit Fundstelle; Issue `#0` … | ☑ | **gilt** | [test_report.py](../tools/report/test_report.py) «test_issue_192_ak3_…» (2) | 21.11.2026 ok (am echten Stand keine) |
| #192/AK4 | Tests in `tools/report/test_report.py` für jedes Beispiel oben. | ☑ | **gilt** | [test_report.py](../tools/report/test_report.py) «test_issue_192_…» | 21.11.2026 ok (Durchsicht; 26 Tests grün) |
| #192/AK5 | `docs/TESTS.md` beschreibt die erkannten Schreibweisen. | ☑ | **gilt** | – | 21.11.2026 ok (Durchsicht) |
| #192/AK6 | Ein per Kommentar nachgetragenes Kriterium zählt als vorhanden; Beispiel #69/AK8 und #170/AK9 sind nicht verwaist. … | ☑ | **gilt** | [test_report.py](../tools/report/test_report.py) «test_issue_192_ak6_…» | 21.11.2026 ok (am echten Stand nicht verwaist) |

### #193 Verwaiste Kennungen #69/AK8 und #170/AK9 zuordnen

[Issue #193](https://github.com/ingen63/VivaPlan/issues/193) · geschlossen · Umsetzung: keine Dateiänderung (Entscheid im Issue)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #193/AK1 | `#69/AK8`: Für die Fundstellen `crates/engine/tests/strategy.rs` …, `strategy.rs`, `library.rs` und `e2e/ui-start.spec.js` ist entschieden, … | ☑ | **gilt** | [strategy.rs](../crates/engine/tests/strategy.rs) «issue_69_ak8_…» (2), [ui-start.spec.js](../e2e/ui-start.spec.js) «#69/AK8: …» | 21.11.2026 ok: Entscheid «per Kommentar bereits nachgetragen» im Issue; Tests grün |
| #193/AK2 | `#170/AK9`: dasselbe für `crates/app/src/main.rs` (Test `issue_170_ak9_language_menu_entries`) und `e2e/ui-sprache.spec.js` (zwei Tests). | ☑ | **gilt** | [main.rs](../crates/app/src/main.rs) «issue_170_ak9_…» (2), [ui-sprache.spec.js](../e2e/ui-sprache.spec.js) «AK9» (2) | 21.11.2026 ok: Entscheid im Issue; Tests grün |
| #193/AK3 | Danach findet der Anforderungsbezug (`make report`, mit #192) keine verwaiste Kennung ausser `#0`. | ☑ | **gilt** | `tools/report/report.py` | 21.11.2026 ok: «Verwaiste Kennungen (0)» |
| #193/AK4 | Alle Tests bleiben grün; geändert werden nur Kennungen, Kommentare und Testnamen. | ☑ | **gilt** | `make test-quiet`, e2e gezielt | 21.11.2026 ok: keine Änderung, 789 Rust-Tests und 5 e2e-Tests grün |

### #194 Wertänderung der Liegenschaften im Hauptbuch unabhängig prüfen

[Issue #194](https://github.com/ingen63/VivaPlan/issues/194) · geschlossen · Umsetzung: 933c1e4

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #194/AK1 | Ein Test rechnet die Wertänderung eines Monats unabhängig nach und vergleicht `ledger.revaluation` auf 0.01 CHF. Beispiel: … | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) «issue_194_ak1_…» | 21.11.2026 ok |
| #194/AK2 | `value_and_fix_costs_inflation_are_independent` prüft den hergeleiteten Wert: Wert 500'000, Teuerung 10 % p. a., Januar 2031 = … | ☑ | **gilt** | [reference.rs](../crates/engine/tests/reference.rs) «value_and_fix_costs_inflation_are_independent» | 21.11.2026 ok |
| #194/AK3 | Ein Wert, der die Teuerung doppelt anwendet (`base × infl_idx²` in `worth()`), lässt beide Tests fehlschlagen; im Issue mit … | ☑ | **gilt** | kein Test (vorübergehende Änderung, Beleg im Issue) | 21.11.2026 ok: beide Tests rot, Änderung rückgängig |
| #194/AK4 | Rechenergebnisse und Golden-Dateien bleiben unverändert (es ändern sich nur Tests). | ☑ | **gilt** | [golden.rs](../crates/engine/tests/golden.rs) | 21.11.2026 ok (Commit ändert nur `reference.rs`) |

### #195 Golden-Zahlen im Workflow vergleichen

[Issue #195](https://github.com/ingen63/VivaPlan/issues/195) · geschlossen · Umsetzung: 8b36be7

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #195/AK1 | Der Push-Lauf von `test-quality.yml` vergleicht die Golden-Dateien mit dem Stand vor dem Push (`tools/golden_numbers.py --base …`) … | ☑ | **gilt** | Job `golden` in [test-quality.yml](../.github/workflows/test-quality.yml); [test_golden_numbers.py](../tools/test_golden_numbers.py) «test_issue_195_ak3_changed_monthly_value_without_reason_fails» (Pfad in der Ausgabe) | 21.11.2026 ok: Schritt lokal in einem Klon nachgestellt (Zusammenfassung mit `05-immobilie-hypothek.single.records[1].age`), actionlint ohne Befund; echter Lauf nach dem nächsten Push steht aus |
| #195/AK2 | Ändern sich Zahlen, schlägt der Lauf fehl, ausser die Commit-Nachricht eines der gepushten Commits begründet die Änderung … | ☑ | **gilt** | [test_golden_numbers.py](../tools/test_golden_numbers.py) «test_issue_195_ak2_…» (2), «test_issue_195_ak3_…» (6) | 21.11.2026 ok: Regel `Golden: #Nr Begründung` (Kommentar im Issue); im Klon ohne Zeile Code 1, mit Zeile Code 0 |
| #195/AK3 | Beispiel: Ein Commit, der in `05-immobilie-hypothek.json` nur einen Monatswert ändert, lässt den Lauf ohne Begründung fehlschlagen; … | ☑ | **gilt** | [test_golden_numbers.py](../tools/test_golden_numbers.py) «test_issue_195_ak3_…» | 21.11.2026 ok: Tests im Kleinen; im Klon mit der echten Datei nachgestellt |
| #195/AK4 | `tools/test_golden_numbers.py` läuft im Workflow. | ☑ | **gilt** | Job `golden`, Schritt «Tests des Prüfskripts» | 21.11.2026 ok (Durchsicht; 12 Tests lokal grün) |
| #195/AK5 | `docs/TESTS.md` (Abschnitt Golden) und `CLAUDE.md` (Regel zu Golden-Dateien) nennen die Prüfung. | ☑ | **gilt** | [TESTS.md](TESTS.md) Zeile 5c, [CLAUDE.md](../CLAUDE.md) Abschnitt Tests | 21.11.2026 ok (Durchsicht) |

### #196 Schalter «Immobilien nie verkaufen»

[Issue #196](https://github.com/ingen63/VivaPlan/issues/196) · geschlossen · Umsetzung: 02a5e37

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #196/AK1 | Der Reiter «Immobilien» hat oben den Schalter «Immobilien nie verkaufen» (Standard: aus). Er gilt für alle Immobilien der … | ☑ | **gilt** | [ui-planung.spec.js](../e2e/ui-planung.spec.js) «AK1» (#196); [never_sell.rs](../crates/engine/tests/never_sell.rs) «issue_196_ak1_…» | 21.11.2026 ok |
| #196/AK2 | Ist der Schalter an, sind alle übrigen Felder des Reiters (auch Hypotheken und Kosten) grau und nur lesbar. Die Werte … | ☑ | **gilt** | [ui-planung.spec.js](../e2e/ui-planung.spec.js) «AK2» (#196) | 21.11.2026 ok |
| #196/AK3 | Ist der Schalter an, verkauft der Rechenkern keine Immobilie: kein geplanter Verkauf, kein Notverkauf, kein Verkauf wegen … | ☑ | **gilt** | [never_sell.rs](../crates/engine/tests/never_sell.rs) «issue_196_ak3_…» (4) | 21.11.2026 ok |
| #196/AK4 | Ein geplanter Kauf, der nicht an einen Verkauf gebunden ist, findet auch bei eingeschaltetem Schalter statt. Käufe «nach … | ☑ | **gilt** | [never_sell.rs](../crates/engine/tests/never_sell.rs) «issue_196_ak4_…» | 21.11.2026 ok |
| #196/AK5 | Reicht das liquide Vermögen nicht mehr, zählt der Lauf als «Vermögen aufgebraucht» (Ruin), wie ohne Immobilie. Beispiel: … | ☑ | **gilt** | [never_sell.rs](../crates/engine/tests/never_sell.rs) «issue_196_ak5_…» (2) | 21.11.2026 ok (Beispielplanung, Bootstrap 200 Läufe) |
| #196/AK6 | Die Prüfung der Planung meldet einen Hinweis, wenn der Schalter an ist und die Planung Verkäufe oder Käufe «nach Verkauf» … | ☑ | **gilt** | [never_sell.rs](../crates/engine/tests/never_sell.rs) «issue_196_ak6_…»; [ui-planung.spec.js](../e2e/ui-planung.spec.js) «AK6» (#196) | 21.11.2026 ok |
| #196/AK7 | Die Kommandozeile (`run`, `optimize`) rechnet mit dem Schalter aus der Datei. | ☑ | **gilt** | [cli.rs](../crates/cli/tests/cli.rs) «issue_196_ak7_…» | 21.11.2026 ok |
| #196/AK8 | Regressionstest im Rechenkern. Bei ausgeschaltetem Schalter bleiben die Golden-Dateien unverändert. | ☑ | **gilt** | [never_sell.rs](../crates/engine/tests/never_sell.rs) «issue_196_ak8_…»; [golden.rs](../crates/engine/tests/golden.rs) | 21.11.2026 ok (Commit ändert keine Golden-Datei) |
| #196/AK9 | Hilfe (Immobilien, Notverkauf) und `docs/BESCHREIBUNG.md` sind nachgeführt. | ☑ | **gilt** | kein Test (Durchsicht) | 21.11.2026 ok: Hilfe in vier Sprachen, neuer Abschnitt `f-re-never-sell` |

### #197 Entnahmestrategien ohne Optimierung vergleichen

[Issue #197](https://github.com/ingen63/VivaPlan/issues/197) · geschlossen · Umsetzung: 793fdd9

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #197/AK1 | In der Optimierung startet ein Knopf «Strategien vergleichen (ohne Optimierung)» die gewählten Entnahmestrategien mit den … | ☑ | **gilt** | [ui-optimierung.spec.js](../e2e/ui-optimierung.spec.js) «AK1» (#197); [compare_strategies.rs](../crates/engine/tests/compare_strategies.rs) «issue_197_ak1_…» | 22.11.2026 ok |
| #197/AK2 | Die Ergebnistabelle zeigt je Strategie zusätzlich das nominale Endvermögen bei 1, 5, 50, 95 und 99 %, auch nach einer … | ☑ | **gilt** | [ui-optimierung.spec.js](../e2e/ui-optimierung.spec.js) «AK2» (#197); [compare_strategies.rs](../crates/engine/tests/compare_strategies.rs) «issue_197_ak2_…» | 22.11.2026 ok |
| #197/AK3 | Eine Auswahl legt fest, womit verglichen wird: «wie geplant», «Immobilien nie verkaufen» oder «beide». Bei «beide» steht … | ☑ | **gilt** | [ui-optimierung.spec.js](../e2e/ui-optimierung.spec.js) «AK3» (#197); [compare_strategies.rs](../crates/engine/tests/compare_strategies.rs) «issue_197_ak3_…» | 22.11.2026 ok |
| #197/AK4 | Die Kommandozeile kann dasselbe (Vergleich ohne Suche, Auswahl «wie geplant / nie verkaufen / beide»; Namen der Optionen … | ☑ | **gilt** | [cli.rs](../crates/cli/tests/cli.rs) «issue_197_ak4_…»; [optimize.rs](../crates/cli/src/optimize.rs) «issue_197_ak4_…» (2) | 22.11.2026 ok (Optionen `--compare`, `--property planned\|never-sell\|both`; von Hand mit der Beispielplanung) |
| #197/AK5 | Für den Vergleich ohne Optimierung gibt es einen Test mit der Beispielplanung. Die Kennzahlen sind dieselben wie bei `run` … | ☑ | **gilt** | [compare_strategies.rs](../crates/engine/tests/compare_strategies.rs) «issue_197_ak5_…» | 22.11.2026 ok (7 Strategien; von Hand: 50 % gleich wie `run`) |
| #197/AK6 | Hilfe (Optimierung) und `docs/BESCHREIBUNG.md` sind nachgeführt. | ☑ | **gilt** | kein Test (Durchsicht); Hilfe-Tests [help.rs](../crates/app/tests/help.rs) | 22.11.2026 ok: vier Sprachen, neuer Abschnitt `o-compare` |

### #198 13. AHV-Rente im Netto-Bedarf der 3-Topf-Strategie

[Issue #198](https://github.com/ingen63/VivaPlan/issues/198) · geschlossen · Umsetzung: 7a73566

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #198/AK1 | Ab 2026 zählt im Netto-Bedarf die AHV eines Jahres mit 13. Rente: 13 × Monatsrente. Wird hochgerechnet (Auslöser A), gilt … | ☑ | **gilt** | [buckets_need.rs](../crates/engine/tests/buckets_need.rs) «issue_198_ak1_…», «start_in_march_…» | 22.11.2026 ok |
| #198/AK2 | Beispiel aus `buckets_need.rs`: AHV 2'500 pro Monat, Lebenshaltung 100'000, Wohnen 24'000, PK-Rente 54'000, Steuersatz … | ☑ | **gilt** | [buckets_need.rs](../crates/engine/tests/buckets_need.rs) «issue_198_ak2_…» | 22.11.2026 ok |
| #198/AK3 | Die Einkommenssteuer im Netto-Bedarf rechnet mit dem Jahreseinkommen inklusive 13. Rente, gleich wie die Simulation (#190). | ☑ | **gilt** | [buckets_need.rs](../crates/engine/tests/buckets_need.rs) «issue_198_ak2_…», «earned_income_…» | 22.11.2026 ok |
| #198/AK4 | Ehepaare: Basis ist die plafonierte AHV des Haushalts, ohne zweite Deckelung (wie #190/AK4). | ☑ | **gilt** | [sim_rules.rs](../crates/engine/tests/sim_rules.rs) «issue_198_ak4_…» | 22.11.2026 ok |
| #198/AK5 | Die Tabelle «3-Topf-Strategie» in den Ergebnissen zeigt bei «AHV» den Wert mit 13. Rente. Ein Oberflächentest prüft das. | ☑ | **gilt** | [ui-ergebnisse.spec.js](../e2e/ui-ergebnisse.spec.js) «AK5» (#198) | 22.11.2026 ok |
| #198/AK6 | Hilfe (Netto-Bedarf, vier Sprachen) und Spezifikation in `docs/ENTWICKLUNG.md` sind nachgeführt; der Vermerk «ohne 13. … | ☑ | **gilt** | kein Test (Durchsicht) | 22.11.2026 ok: Hilfe vier Sprachen, Tooltip, Spezifikation |
| #198/AK7 | Ändern sich Golden-Zahlen, ist das im Issue begründet (Zeile `Golden: #Nr …`). | ☑ | **gilt** | [golden.rs](../crates/engine/tests/golden.rs) | 22.11.2026 ok (keine Golden-Datei geändert) |

### #199 Nicht tragbare Erneuerungen bei «Immobilien nie verkaufen» zählen

[Issue #199](https://github.com/ingen63/VivaPlan/issues/199) · geschlossen · Umsetzung: 2f25184

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #199/AK1 | Jeder Lauf zählt die Erneuerungen, die nur dank «Immobilien nie verkaufen» voll erneuert wurden. Das sind die Fälle mit dem … | ☑ | **gilt** | [never_sell.rs](../crates/engine/tests/never_sell.rs) «issue_199_ak1_…» | 22.11.2026 ok |
| #199/AK2 | Die Auswertung vieler Läufe nennt die Zahl der Läufe mit mindestens einer solchen Erneuerung. | ☑ | **gilt** | [never_sell.rs](../crates/engine/tests/never_sell.rs) «issue_199_ak2_…»; [batch.rs](../crates/engine/src/batch.rs) «summary_of_no_runs_and_counters» | 22.11.2026 ok |
| #199/AK3 | In den Ergebnissen erscheint die Zahl bei eingeschaltetem Schalter neben dem Notverkauf, als Anzahl Läufe und Anteil. … | ☑ | **gilt** | [ui-ergebnisse.spec.js](../e2e/ui-ergebnisse.spec.js) «AK3» (#199) | 22.11.2026 ok |
| #199/AK4 | Die Kommandozeile (`run`) gibt die Zahl im Bericht und im JSON aus. | ☑ | **gilt** | [cli.rs](../crates/cli/tests/cli.rs) «issue_199_ak4_…» (2) | 22.11.2026 ok (von Hand: Beispielplanung, Bootstrap 200 Läufe, 31 Läufe = 15.5 %) |
| #199/AK5 | Bei ausgeschaltetem Schalter ist die Zahl immer 0. Ist sie 0, fehlt das Feld im Ergebnis-JSON; die Golden-Dateien bleiben … | ☑ | **gilt** | [never_sell.rs](../crates/engine/tests/never_sell.rs) «issue_199_ak5_…», «issue_199_ak2_…»; [golden.rs](../crates/engine/tests/golden.rs) | 22.11.2026 ok (keine Golden-Datei geändert) |
| #199/AK6 | Texte in vier Sprachen. Hilfe (Immobilien nie verkaufen, Ergebnisse) und `docs/BESCHREIBUNG.md` sind nachgeführt. | ☑ | **gilt** | kein Test (Durchsicht); i18n-check; Hilfe-Tests [help.rs](../crates/app/tests/help.rs) | 22.11.2026 ok: vier Sprachen, Hilfe, Beschreibung, Spezifikation |
