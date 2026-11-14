# Abnahmeliste

Stand 14.11.2026 (FEA-77, #97). Die Liste nennt die Abnahmekriterien aller umgesetzten Issues und schätzt ein, ob sie zum
heutigen Funktionsstand noch passen. Sie ist die Grundlage für Einzelprüfungen: Claude prüft auf Anweisung ein Issue oder
ein Kriterium und trägt das Ergebnis in der Spalte «Geprüft» ein.

## Auswahl

Einbezogen sind alle geschlossenen Issues und alle offenen Issues mit dem Label `umgesetzt`. Nicht einbezogen sind nur
offene Issues ohne `umgesetzt` (noch nicht umgesetzt). Abfrage zum Nachführen:

```bash
gh issue list --state all --limit 500 --json number,title,state,labels,body,comments \
  | jq '[.[] | select(.state == "CLOSED" or (.labels | map(.name) | index("umgesetzt")))] | sort_by(.number)'
```

Kriterien sind die Häkchen im Abschnitt «Abnahmekriterien» des Issues, gezählt in ihrer Reihenfolge (`AK2` = zweites
Häkchen); ohne diesen Abschnitt alle Häkchen im Text. Kriterien, die später in einem Kommentar dazukamen, stehen mit dem
Vermerk «(Kommentar)» in der Liste. Änderungen an Kriterien per Kommentar sind in der Einschätzung genannt.

## Spalten

- **Im Issue:** Häkchen im Issue (☑ abgehakt, ☐ offen).
- **Einschätzung:** **gilt** (passt zum heutigen Stand, allenfalls mit Hinweis auf geänderte Namen oder Pfade),
  **überholt** (ein späteres Issue oder eine spätere Entscheidung hat das Verhalten geändert; der Grund steht dabei) oder
  **unklar** (braucht einen Entscheid).
- **Test:** Tests, die das Kriterium mit seiner Kennung nennen (`FEA-xx/AKn` im Namen oder Kommentar, in `e2e/` auch als
  `AKn` innerhalb eines Blocks mit der Issue-ID). Tests, die nur das Issue nennen, stehen beim Issue. Die Zeilennummern
  gelten für den Stand dieser Liste.
- **Geprüft:** Datum und Ergebnis der letzten Einzelprüfung (`ok`, `Fehler: …`, `nicht prüfbar: …`), mit Art der Prüfung
  (Test, Oberfläche, Durchsicht).

## Vorgehen bei der Einzelprüfung

1. Anweisung zum Beispiel «Prüfe FEA-41» oder «Prüfe FEA-41/AK4».
2. Claude prüft das Kriterium am heutigen Stand: vorhandenen Test laufen lassen, sonst in der Oberfläche oder im Code
   nachsehen. Bei «überholt» wird nur geprüft, ob das neue Verhalten gilt.
3. Claude trägt das Ergebnis in der Spalte «Geprüft» ein. Im Issue hakt Claude nichts ab und ändert keine Kriterien.
4. Ein Fehler wird als neues Issue vorgeschlagen (Label `bug`, `gefunden:abnahme`). Ein überholtes Kriterium bekommt
   auf Wunsch einen Kommentar im Issue.

## Übersicht

| | Anzahl |
|---|---|
| Issues in der Liste | 76 (davon 5 ohne Kriterien) |
| Kriterien | 452 |
| gilt | 433 |
| überholt | 16 |
| unklar | 3 |
| im Issue nicht abgehakt | 19 |
| ohne Test mit Kennung | 302 |

### Überholt oder unklar

| Kennung | Einschätzung |
|---|---|
| FEA-02/AK3 | **überholt**: durch FEA-54 und FEA-45: Beim ersten Start erscheinen Haftungsausschluss und Willkommensdialog; ohne gemerkte Datei lädt die Desktop-App `Neu.vplan`. |
| FEA-11/AK2 | **unklar**: Seit FEA-53 wählt man eine Anlagestrategie statt einer Zeitreihendatei; zu klären, was die Spalte «Zeitreihe» heute zeigen soll. |
| FEA-12/AK8 | **überholt**: durch FEA-24: Die Schwelle ist keine Einstellung geworden und gilt fest mit 5 % (Kommentar in FEA-24). |
| FEA-17/AK3 | **überholt**: durch FEA-58: Der Hinweis erscheint nur einmal, beim ersten Laden einer betroffenen Planung. |
| FEA-18/AK1 | **überholt**: durch FEA-53: Die Karte «Zeitreihe» gibt es nicht mehr; die Erklärung steht im Info-Popup der Anlagestrategie. |
| FEA-18/AK4 | **überholt**: durch den Kommentar im Issue (Info in der Tabellenzeile) und danach FEA-53 (keine Tabelle der Reihen mehr). |
| FEA-18/AK5 | **unklar**: Seit FEA-53 gibt es keine eigenen Dateien und keinen Ordnerpfad mehr; der Test prüft, dass der Text kopierbar ist. |
| FEA-18/AK6 | **überholt**: durch den Kommentar im Issue (Texte gekürzt) und FEA-53 (Texte zur Anlagestrategie). |
| FEA-18/K1 | **überholt**: durch FEA-53: keine Tabelle der Reihen mehr. |
| FEA-18/K2 | **überholt**: durch FEA-53: keine Tabelle der Reihen mehr; das Popup gibt es je Anlagestrategie. |
| FEA-18/K3 | **unklar**: Gilt die Grenze von 2 Zeilen auch für die Texte der Anlagestrategie? |
| FEA-18/K4 | **überholt**: durch FEA-53 und FEA-72: Die vier Reihen sind nicht mehr einzeln wählbar, Pictet ist entfernt. |
| ISS-9/AK3 | **überholt**: durch die Arbeitsweise in `CLAUDE.md`: keine Pull Requests mehr, gearbeitet wird direkt auf `main`; sinngemäss für den Push prüfen. |
| FEA-34/AK7 | **überholt**: durch GEN-01: Das öffentliche Repository beginnt mit einem einzigen Commit; die Historie liegt nur im alten, privaten Repository. |
| FEA-35/AK8 | **überholt**: durch GEN-01: keine übernommene Historie im öffentlichen Repository. |
| FEA-38/AK7 | **überholt**: durch FEA-24: Die Grenzen kommen aus den Einstellungen. |
| FEA-37/AK11 | **überholt**: durch FEA-24: Der Maximalbetrag kommt aus den Einstellungen. |
| FEA-42/AK4 | **überholt**: durch GEN-01: Das öffentliche Repository hat die alte Historie nicht; `.git-blame-ignore-revs` ist entfernt. |
| FEA-53/AK6 | **überholt**: durch FEA-72: «Schweiz (Pictet)» ist entfernt. |

### Im Issue nicht abgehakt

FEA-12/AK2, FEA-33/AK1, FEA-33/AK2, FEA-33/AK3, FEA-33/AK4, FEA-33/AK5, FEA-35/AK3, FEA-60/AK1, FEA-60/AK2, FEA-60/AK3, FEA-60/AK4, FEA-60/AK5, FEA-60/AK6, FEA-60/AK7, FEA-60/AK8, FEA-74/AK1, FEA-74/AK2, FEA-74/AK3, FEA-74/AK4

## Issues

### #1 ISS-1 Steuern 2029→2030 steigen stärker als die Einnahmen (False Positive)

[Issue #1](https://github.com/ingen63/VivaPlan/issues/1) · geschlossen, nicht geplant · Umsetzung: nicht vermerkt

Fehlalarm (`false-positive`), keine Kriterien.

### #2 ISS-2 Kauf nach Verkauf einer Immobilie erfolgt erst Jahre später

[Issue #2](https://github.com/ingen63/VivaPlan/issues/2) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/reference.rs:369](../crates/engine/tests/reference.rs#L369), [engine/tests/simulation.rs:961](../crates/engine/tests/simulation.rs#L961)

Keine Kriterien im Issue; der Entscheid zum Verhalten steht im Issue-Text.

### #3 FEA-01 Zeitleiste: Immobilienereignisse beschriften

[Issue #3](https://github.com/ingen63/VivaPlan/issues/3) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-01/AK1 | Mit `examples/Test 1.rplan` hat jedes Kauf-, Verkaufs- und Notverkaufsereignis eine Beschriftung. | ☑ | **gilt**: Die Planung `examples/Test 1.rplan` gibt es nicht mehr (GEN-01); prüfen mit der Beispielplanung `beispiel.vplan` oder einer erfundenen Planung. | kein Test | – |
| FEA-01/AK2 | Keine zwei Beschriftungen überdecken sich. | ☑ | **gilt** | kein Test | – |
| FEA-01/AK3 | Ereignisse und Meilensteine sind ohne Farbunterscheidung auseinanderzuhalten. | ☑ | **gilt** | kein Test | – |
| FEA-01/AK4 | Bei mehr als 15 Ereignissen erscheinen nur Symbole, Text beim Hover. | ☑ | **gilt** | kein Test | – |
| FEA-01/AK5 | Ohne Ergebnis keine Ereignisse. | ☑ | **gilt** | kein Test | – |

### #4 FEA-02 Beim Start die letzte Planung öffnen

[Issue #4](https://github.com/ingen63/VivaPlan/issues/4) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/example_neu.rs:1](../crates/engine/tests/example_neu.rs#L1), [ui.spec.js:1974](../e2e/ui.spec.js#L1974) (+3)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-02/AK1 | Planung öffnen, App beenden, neu starten: dieselbe Planung ist geladen, der Dateiname steht im Kopf. | ☑ | **gilt** | kein Test | – |
| FEA-02/AK2 | Speichern unter neuem Namen und Neustart: die neue Datei wird geöffnet. | ☑ | **gilt** | kein Test | – |
| FEA-02/AK3 | Erststart ohne Merkung: Beispielplanung, kein Hinweis. | ☑ | **überholt**: durch FEA-54 und FEA-45: Beim ersten Start erscheinen Haftungsausschluss und Willkommensdialog; ohne gemerkte Datei lädt die Desktop-App `Neu.vplan`. | kein Test | – |
| FEA-02/AK4 | Datei zwischen den Starts gelöscht: Beispielplanung und Hinweis «Zuletzt geöffnete Datei nicht gefunden». | ☑ | **gilt** | kein Test | – |
| FEA-02/AK5 | «Neue Planung» ohne Speichern, dann Neustart: die zuletzt gespeicherte Planung wird geöffnet. | ☑ | **gilt** | kein Test | – |
| FEA-02/AK6 | Beenden mit ungespeicherten Änderungen: Dialog mit Speichern, Verwerfen, Abbrechen; Abbrechen lässt die App offen. | ☑ | **gilt** | kein Test | – |
| FEA-02/AK7 | Die Merkung enthält nur den Dateipfad und bleibt lokal. | ☑ | **gilt** | kein Test | – |
| FEA-02/AK8 | Die Beispielplanung enthält keine persönlichen Daten und ist ohne Fehler simulierbar. | ☑ | **gilt**: Beispielplanung heute `crates/engine/assets/beispiel.vplan`. | kein Test | – |

### #5 FEA-03 Ergebnisse: Reiter «Parameter»

[Issue #5](https://github.com/ingen63/VivaPlan/issues/5) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-03/AK1 | Nach einem Lauf zeigt der Reiter alle Werte des Laufs; danach im Editor eine Zahl ändern: Der Reiter des bereits berechneten Ergebnisses bleibt unverändert. | ☑ | **gilt** | kein Test | – |
| FEA-03/AK2 | Ein gespeichertes Ergebnis zeigt nach dem Öffnen dieselbe Zusammenfassung wie beim Berechnen. | ☑ | **gilt** | kein Test | – |
| FEA-03/AK3 | Nichts im Reiter ist editierbar. | ☑ | **gilt** | kein Test | – |
| FEA-03/AK4 | Test: je einen Parameter pro Editorabschnitt ändern, neu rechnen: Die beiden Zusammenfassungen unterscheiden sich an dieser Stelle. | ☑ | **gilt** | kein Test | – |
| FEA-03/AK5 | Ein altes Ergebnis ohne Parameter zeigt «Nicht verfügbar» und keinen Fehler. | ☑ | **gilt** | kein Test | – |

### #6 FEA-05 Guardrails und 3-Topf gemeinsam anwenden

[Issue #6](https://github.com/ingen63/VivaPlan/issues/6) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/src/optimize.rs:1566](../crates/engine/src/optimize.rs#L1566), [engine/tests/buckets_guardrails.rs:1](../crates/engine/tests/buckets_guardrails.rs#L1) (+2)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-05/AK1 | Auf der Karte «3-Topf» kann «Ausgaben ab Pensionierung» zwischen «wie geplant» und «mit Guardrails steuern» gewählt werden; bei «mit Guardrails» erscheinen die Karten «Guardrails» und «Grenzen der Lebenshaltung». | ☑ | **gilt** | [ui.spec.js:1866](../e2e/ui.spec.js#L1866) | – |
| FEA-05/AK2 | Bei «mit Guardrails» bestimmen Startrate, Teuerung, Leitplanken und Grenzen die Lebenshaltung genau wie bei der Strategie «Guardrails»; bei gleichen Eingaben und gleichem Vermögensverlauf ist die Entnahme in beiden identisch. | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:99](../crates/engine/tests/buckets_guardrails.rs#L99) | – |
| FEA-05/AK3 | Die Guardrails (rein und kombiniert) starten mit der Pensionierung der ersten Person; in der Kombination gemeinsam mit den Töpfen. | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:121](../crates/engine/tests/buckets_guardrails.rs#L121) | – |
| FEA-05/AK4 | Ein für einen Kauf reservierter Verkaufserlös zählt für die Guardrails-Rate (rein und kombiniert) nicht zum Vermögen. | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:156](../crates/engine/tests/buckets_guardrails.rs#L156) | – |
| FEA-05/AK5 | Nach einem Zufluss rechnen die Guardrails (rein und kombiniert) die Entnahme neu; in der Kombination geht das Geld in Topf 3, und die Topfziele bleiben bis zum nächsten Januar. | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:187](../crates/engine/tests/buckets_guardrails.rs#L187) | – |
| FEA-05/AK6 | Netto-Bedarf und Topfziele werden aus der Guardrails-Lebenshaltung berechnet; die Tabelle «3-Topf-Strategie» nennt diese Quelle. | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:228](../crates/engine/tests/buckets_guardrails.rs#L228) | – |
| FEA-05/AK7 | Nach einer Kürzung durch die Leitplanke sinken die Ziele von Topf 1 und 2 in der nächsten Januar-Auffüllung entsprechend. | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:276](../crates/engine/tests/buckets_guardrails.rs#L276) | – |
| FEA-05/AK8 | Die Topf-Regeln (Januar-Auffüllung, A, B, C, Renditekriterium, Notverkauf, Phase 8) gelten unverändert. | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:292](../crates/engine/tests/buckets_guardrails.rs#L292) | – |
| FEA-05/AK9 | Planungen ohne das neue Feld rechnen unverändert, ausser mit der Strategie «Guardrails» bei zwei Personen mit verschiedenem Pensionierungsdatum oder mit reserviertem Verkaufserlös (Golden-Dateien unverändert, keine verwendet Guardrails). | ☑ | **gilt** | [ui.spec.js:1866](../e2e/ui.spec.js#L1866) | – |
| FEA-05/AK10 | Der Strategiename «3-Topf + Guardrails» erscheint in Ergebnissen, gespeicherten Ergebnissen, Vergleich und Tooltip (FEA-10). | ☑ | **gilt** | [engine/tests/buckets_guardrails.rs:347](../crates/engine/tests/buckets_guardrails.rs#L347), [ui.spec.js:1866](../e2e/ui.spec.js#L1866) | – |
| FEA-05/AK11 | Die Optimierung bietet «3-Topf + Guardrails» als eigene Strategie an (standardmässig nicht angewählt; untere Leitplanke und Anpassung standardmässig fix). | ☑ | **gilt** | kein Test | – |
| FEA-05/AK12 | Hilfe, `BESCHREIBUNG.md` und Spezifikation beschreiben die Kombination. | ☑ | **gilt** | kein Test | – |

### #7 FEA-06 Jahreswerte: Entnahmerate als Spalte

[Issue #7](https://github.com/ingen63/VivaPlan/issues/7) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-06/AK1 | Entnahme 40'000 pro Jahr bei Total 1'000'000 und Liquid 500'000 am Jahresanfang ergibt 4.0 % bzw. 8.0 %. | ☑ | **gilt** | kein Test | – |
| FEA-06/AK2 | Im ersten Jahr einer im November startenden Simulation: Entnahme der zwei Monate mal 6, geteilt durch das Startvermögen. | ☑ | **gilt** | kein Test | – |
| FEA-06/AK3 | Jahre mit Einkommen über den Ausgaben zeigen leere Zellen. | ☑ | **gilt** | kein Test | – |
| FEA-06/AK4 | Wechsel nominal/real ändert die Werte nicht. | ☑ | **gilt** | kein Test | – |
| FEA-06/AK5 | Jede Zeile stimmt mit der manuellen Rechnung aus den Spalten «Entnahme», «Total» und «Liquid» (Test mit `examples/Test 1.rplan`). | ☑ | **gilt**: Die Planung `examples/Test 1.rplan` gibt es nicht mehr (GEN-01); prüfen mit der Beispielplanung `beispiel.vplan` oder einer erfundenen Planung. | kein Test | – |

### #8 FEA-07 Jahreswerte: Steuerspalten einklappbar

[Issue #8](https://github.com/ingen63/VivaPlan/issues/8) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui.spec.js:837](../e2e/ui.spec.js#L837)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-07/AK1 | Beim ersten Öffnen (ohne gespeicherte Wahl) zeigt die Jahrestabelle keine Spalte mit «Steuer». | ☑ | **gilt** | kein Test | – |
| FEA-07/AK2 | «Steuern anzeigen» blendet die fünf Spalten mit denselben Werten wie bisher ein, ein zweiter Klick blendet sie aus. | ☑ | **gilt** | kein Test | – |
| FEA-07/AK3 | Nach Neustart der App gilt die zuletzt gewählte Einstellung. | ☑ | **gilt** | kein Test | – |
| FEA-07/AK4 | Kennzahl «Steuern total» bleibt sichtbar. | ☑ | **gilt** | kein Test | – |
| FEA-07/AK5 | Die Berechnung und ein allfälliger Export sind unverändert. | ☑ | **gilt** | kein Test | – |
| FEA-07/AK6 | Das Umschalten verschiebt keine anderen Spalten. | ☑ | **gilt** | kein Test | – |
| FEA-07/AK7 | Steht die gespeicherte Einstellung nicht zur Verfügung (zum Beispiel Datenverzeichnis nicht lesbar), gilt eingeklappt. | ☑ | **gilt** | kein Test | – |

### #9 FEA-08 3-Topf-Herleitung aufklappbar in den Jahreswerten

[Issue #9](https://github.com/ingen63/VivaPlan/issues/9) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui.spec.js:1196](../e2e/ui.spec.js#L1196)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-08/AK1 | Ein 3-Topf-Lauf hat keine Karte «3-Topf-Strategie, Herleitung …». | ☑ | **gilt** | kein Test | – |
| FEA-08/AK2 | In den Jahreswerten lässt sich zu einem Jahr mit Auffüllung die Herleitung aufklappen, mit allen Angaben nach Abschnitt 10. | ☑ | **gilt** | kein Test | – |
| FEA-08/AK3 | Jahre ohne Auffüllung haben keine Aufklapp-Zeile. | ☑ | **gilt** | kein Test | – |
| FEA-08/AK4 | Läufe ohne 3-Topf-Strategie zeigen keine Aufklapp-Zeilen. | ☑ | **gilt** | kein Test | – |
| FEA-08/AK5 | Der UI-Test zur Herleitung (`ui-tests/ui.spec.js:728`), die 3-Topf-Spezifikation (Abschnitt 10), die Hilfe und die Beschreibung sind nachgeführt. | ☑ | **gilt**: Einmalige Nachführung; der Test liegt heute in `e2e/ui.spec.js` (FEA-35). | kein Test | – |
| FEA-08/AK6 | Beim Wechsel des gewählten Einzellaufs zeigt die Tabelle die Herleitung des neuen Laufs. | ☑ | **gilt** | kein Test | – |

### #10 FEA-09 Ereignisliste ohne 3-Topf-Meldungen

[Issue #10](https://github.com/ingen63/VivaPlan/issues/10) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui.spec.js:1196](../e2e/ui.spec.js#L1196)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-09/AK1 | Ein 3-Topf-Lauf zeigt in der Karte «Ereignisse» keine Zeile zu Auffüllung, Zufluss oder Reservierung. | ☑ | **gilt** | kein Test | – |
| FEA-09/AK2 | Pensionierung, Kapitalbezug, Käufe, Verkäufe, Notverkäufe, Defizitphase und Geld aufgebraucht stehen weiterhin in der Liste. | ☑ | **gilt** | kein Test | – |
| FEA-09/AK3 | Ein Lauf ohne 3-Topf-Strategie zeigt dieselbe Liste wie bisher. | ☑ | **gilt** | kein Test | – |
| FEA-09/AK4 | Jeder reservierte Verkaufserlös ist in den Jahreswerten unter der Herleitung (FEA-08) zu finden. | ☑ | **gilt** | kein Test | – |
| FEA-09/AK5 | Zeitleiste und Grafikmarken sind unverändert. | ☑ | **gilt** | kein Test | – |
| FEA-09/AK6 | Der UI-Test, der 3-Topf-Ereignisse in der Liste erwartet, ist angepasst. | ☑ | **gilt** | kein Test | – |

### #11 FEA-10 Gespeicherte Ergebnisse: Strategie-Tooltip mit Parametern

[Issue #11](https://github.com/ingen63/VivaPlan/issues/11) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui.spec.js:1879](../e2e/ui.spec.js#L1879)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-10/AK1 | Mouseover über «3-Topf» zeigt Jahre Topf 1 und 2 und die Renditeschwelle mit den Werten des Ergebnisses (nicht der aktuellen Planung). | ☑ | **gilt** | kein Test | – |
| FEA-10/AK2 | Jede der sechs Strategien zeigt genau ihre Parameter. | ☑ | **gilt**: Heute sieben Strategien (FEA-05: «3-Topf + Guardrails»). | kein Test | – |
| FEA-10/AK3 | Die Werte stimmen mit dem Reiter «Parameter» (FEA-03) überein. | ☑ | **gilt** | kein Test | – |
| FEA-10/AK4 | Der Tooltip ist per Tastaturfokus erreichbar. | ☑ | **gilt** | kein Test | – |
| FEA-10/AK5 | Ein altes Ergebnis ohne Kurzform zeigt «Parameter nicht verfügbar», ohne Fehler. | ☑ | **gilt** | kein Test | – |
| FEA-10/AK6 | Die Liste lädt nicht langsamer (Ergebnisdaten werden dafür nicht geladen). | ☑ | **gilt** | kein Test | – |

### #12 FEA-11 Gespeicherte Ergebnisse: Spalte «Zeitreihe» statt «Art»

[Issue #12](https://github.com/ingen63/VivaPlan/issues/12) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-11/AK1 | Liste und Vergleich haben keine Spalte «Art», aber eine Spalte «Zeitreihe». | ☑ | **gilt** | kein Test | – |
| FEA-11/AK2 | Ein historisches oder Bootstrap-Ergebnis zeigt den Reihennamen ohne Pfad und «.csv». | ☑ | **unklar**: Seit FEA-53 wählt man eine Anlagestrategie statt einer Zeitreihendatei; zu klären, was die Spalte «Zeitreihe» heute zeigen soll. | kein Test | – |
| FEA-11/AK3 | Feste und parametrische Ergebnisse zeigen «feste Renditen» bzw. «parametrisch» in Grau. | ☑ | **gilt** | kein Test | – |
| FEA-11/AK4 | Ein Ergebnis zeigt auch nach Änderung der Zeitreihe in der aktuellen Planung weiterhin die Zeitreihe von damals. | ☑ | **gilt**: Sinngemäss: Änderung der Anlagestrategie statt der Zeitreihe (FEA-53). | kein Test | – |
| FEA-11/AK5 | Einzellauf, Szenarien und Anzahl Läufe sind weiterhin erkennbar (zweite Zeile in «Ergebnis»). | ☑ | **gilt** | kein Test | – |
| FEA-11/AK6 | Alte Einträge zeigen «–» ohne Fehler. | ☑ | **gilt** | kein Test | – |

### #13 FEA-12 Gespeicherte Ergebnisse: 5-%-Vermögen und Warnsymbol bei Notverkauf

[Issue #13](https://github.com/ingen63/VivaPlan/issues/13) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui.spec.js:1658](../e2e/ui.spec.js#L1658)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-12/AK1 | Ein Monte-Carlo-Ergebnis zeigt «Erfolgsquote · 5 %: Betrag», der Betrag entspricht dem 5. Perzentil der Ergebnistabelle der Übersicht; der Median steht im Tooltip. | ☑ | **gilt** | kein Test | – |
| FEA-12/AK2 | Mit `examples/Test 1.rplan` und einem Notverkauf in mindestens 5 % der Läufe erscheint das Symbol, der Tooltip nennt Anteil und Anzahl Läufe. | ☐ | **gilt**: Die Planung `examples/Test 1.rplan` gibt es nicht mehr (GEN-01); prüfen mit der Beispielplanung `beispiel.vplan` oder einer erfundenen Planung. Im Issue nicht abgehakt. | kein Test | – |
| FEA-12/AK3 | Bei weniger als 5 % Läufen mit Notverkauf erscheint kein Symbol. | ☑ | **gilt** | kein Test | – |
| FEA-12/AK4 | Eine Planung ohne Notverkauf oder ohne Immobilien zeigt kein Symbol. | ☑ | **gilt** | kein Test | – |
| FEA-12/AK5 | Ein geplanter Verkauf löst das Symbol nicht aus. | ☑ | **gilt** | kein Test | – |
| FEA-12/AK6 | Ein Einzellauf mit Notverkauf zeigt das Symbol. | ☑ | **gilt** | kein Test | – |
| FEA-12/AK7 | Alte Einträge ohne Angabe werden fehlerfrei und ohne Symbol angezeigt. | ☑ | **gilt** | kein Test | – |
| FEA-12/AK8 | Wird in den Einstellungen (FEA-24) eine andere Schwelle gewählt, gilt sie. | ☑ | **überholt**: durch FEA-24: Die Schwelle ist keine Einstellung geworden und gilt fest mit 5 % (Kommentar in FEA-24). | kein Test | – |

### #14 FEA-13 Pflichteingaben prüfen und markieren

[Issue #14](https://github.com/ingen63/VivaPlan/issues/14) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/mandatory_inputs.rs:1](../crates/engine/tests/mandatory_inputs.rs#L1), [ui.spec.js:437](../e2e/ui.spec.js#L437) (+1)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-13/AK1 | Leerer Name: Fehler am Feld, Start blockiert. | ☑ | **gilt** | kein Test | – |
| FEA-13/AK2 | Leeres oder ungültiges Geburtsdatum: Fehler am Feld, Start blockiert. | ☑ | **gilt** | kein Test | – |
| FEA-13/AK3 | Jede Zeile der Pflichtliste hat einen Test, der den Fehler auslöst und die Meldung prüft. | ☑ | **gilt** | kein Test | – |
| FEA-13/AK4 | Sind alle Pflichtfelder gültig, startet die Simulation wie bisher. | ☑ | **gilt** | kein Test | – |
| FEA-13/AK5 | Beim Blockieren führt der Verweis in den ersten Abschnitt mit Fehler. | ☑ | **gilt** | kein Test | – |
| FEA-13/AK6 | Verkaufsdatum vor dem Simulationsstart: Warnung im Abschnitt Immobilien. | ☑ | **gilt** | kein Test | – |
| FEA-13/AK7 | Die Liste in der Spezifikation stimmt mit den Prüfungen in der Engine überein. | ☑ | **gilt** | kein Test | – |

### #15 FEA-14 Meilensteine: «Verwendet» entfällt, Löschen mit Umstellung

[Issue #15](https://github.com/ingen63/VivaPlan/issues/15) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui.spec.js:374](../e2e/ui.spec.js#L374), [ui.spec.js:413](../e2e/ui.spec.js#L413)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-14/AK1 | Keine Spalte «Verwendet». | ☑ | **gilt** | kein Test | – |
| FEA-14/AK2 | Meilenstein «AHV» löschen: Dialog nennt Verwendung und Datum; danach zeigt die AHV-Rente das feste Datum, die Rechnung liefert dieselben Zahlen wie vorher. | ☑ | **gilt** | kein Test | – |
| FEA-14/AK3 | Bei «Alter von» wird das berechnete Datum eingetragen. | ☑ | **gilt** | kein Test | – |
| FEA-14/AK4 | Rückgängig stellt Meilenstein und Verwendungen wieder her. | ☑ | **gilt** | kein Test | – |
| FEA-14/AK5 | «Pensionierung», «Simulationsstart» und «Simulationsende» lassen sich nicht löschen. | ☑ | **gilt**: «Pensionierung» heisst seit FEA-38 «Bezug Pensionskasse»; gesperrt sind heute Simulationsstart, Simulationsende, Bezug Pensionskasse und AHV-Bezug (FEA-38/AK1). | kein Test | – |
| FEA-14/AK6 | Nicht verwendete Meilensteine lassen sich ohne Dialog löschen. | ☑ | **gilt** | kein Test | – |
| FEA-14/AK7 | Doppelter Zeitpunkt in einer Reihe: genau eine Zeile bleibt, mit Hinweis. | ☑ | **gilt** | kein Test | – |
| FEA-14/AK8 | Variation und Optimierung laufen nach dem Löschen eines anderen Meilensteins unverändert. | ☑ | **gilt** | kein Test | – |

### #16 FEA-15 «Altersguthaben heute» → «per Simulationsstart»

[Issue #16](https://github.com/ingen63/VivaPlan/issues/16) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui.spec.js:114](../e2e/ui.spec.js#L114)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-15/AK1 | Bei Start 11.2026 heisst das Feld «Altersguthaben per Simulationsstart (11.2026)»; nach Änderung des Starts auf 03.2027 ändert sich der Text sofort. | ☑ | **gilt** | kein Test | – |
| FEA-15/AK2 | Die Bezeichnung gilt für beide Personen. | ☑ | **gilt** | kein Test | – |
| FEA-15/AK3 | «Altersguthaben heute» kommt in Editor, Hilfe und Beschreibung nicht mehr vor. | ☑ | **gilt** | kein Test | – |
| FEA-15/AK4 | Oberflächentests sind nachgeführt. | ☑ | **gilt** | kein Test | – |

### #17 FEA-16 «Bestände heute» → «per Simulationsstart»

[Issue #17](https://github.com/ingen63/VivaPlan/issues/17) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-16/AK1 | Karte und Einleitung tragen die neue Bezeichnung mit dem aktuellen Start. | ☑ | **gilt** | kein Test | – |
| FEA-16/AK2 | Die Felder 3a-Guthaben und Marktwert tragen die neue Bezeichnung. | ☑ | **gilt** | kein Test | – |
| FEA-16/AK3 | «Bestände heute», «Aktuelle Bestände», «Guthaben heute» und «Marktwert heute» kommen in Editor, Hilfe und Beschreibung nicht mehr vor. | ☑ | **gilt** | kein Test | – |
| FEA-16/AK4 | Oberflächentests sind nachgeführt. | ☑ | **gilt** | kein Test | – |

### #18 FEA-17 Liquiditätsreserve nur für die Strategie «Standard»

[Issue #18](https://github.com/ingen63/VivaPlan/issues/18) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/reserve_standard_only.rs:1](../crates/engine/tests/reserve_standard_only.rs#L1), [ui.spec.js:1937](../e2e/ui.spec.js#L1937)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-17/AK1 | «Vermögen & Anlage» enthält die Reserve nicht mehr, die Karte «Standard» enthält sie. | ☑ | **gilt**: «Liquiditätsreserve» heisst seit FEA-40 «Cash-Reserve». | kein Test | – |
| FEA-17/AK2 | Eine Planung mit Strategie Standard und Reserve 6 liefert dieselben Ergebnisse wie vor der Änderung (Regressionstest mit den Golden-Dateien). | ☑ | **gilt** | kein Test | – |
| FEA-17/AK3 | Eine Planung mit Guardrails und Reserve 6 rechnet mit 3 Monaten, der Hinweis erscheint beim Laden einmal. | ☑ | **überholt**: durch FEA-58: Der Hinweis erscheint nur einmal, beim ersten Laden einer betroffenen Planung. | kein Test | – |
| FEA-17/AK4 | Die Optimierung bietet die Reserve weiterhin nur für Standard an. | ☑ | **gilt** | kein Test | – |
| FEA-17/AK5 | Hilfe, Beschreibung und Tests sind nachgeführt. | ☑ | **gilt** | kein Test | – |

### #19 FEA-18 Zeitreihen-Erklärungen in ein Info-Popup

[Issue #19](https://github.com/ingen63/VivaPlan/issues/19) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui.spec.js:138](../e2e/ui.spec.js#L138)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-18/AK1 | Die Karte «Zeitreihe» enthält unter der Tabelle keine festen Erklärungstexte mehr. | ☑ | **überholt**: durch FEA-53: Die Karte «Zeitreihe» gibt es nicht mehr; die Erklärung steht im Info-Popup der Anlagestrategie. | kein Test | – |
| FEA-18/AK2 | Maus auf ⓘ öffnet das Popup mit dem bisherigen Text; Tastaturfokus und Klick öffnen es ebenfalls, Esc schliesst es. | ☑ | **gilt**: Heute für das Info-Popup der Anlagestrategie (Test FEA-18/FEA-53). | kein Test | – |
| FEA-18/AK3 | Das Popup bleibt offen, wenn die Maus hineinbewegt wird. | ☑ | **gilt**: Heute für das Info-Popup der Anlagestrategie. | kein Test | – |
| FEA-18/AK4 | Die Info zur gewählten Reihe bleibt sichtbar. | ☑ | **überholt**: durch den Kommentar im Issue (Info in der Tabellenzeile) und danach FEA-53 (keine Tabelle der Reihen mehr). | kein Test | – |
| FEA-18/AK5 | Der Ordnerpfad lässt sich aus dem Popup kopieren. | ☑ | **unklar**: Seit FEA-53 gibt es keine eigenen Dateien und keinen Ordnerpfad mehr; der Test prüft, dass der Text kopierbar ist. | kein Test | – |
| FEA-18/AK6 | Der Inhalt der Texte ist unverändert. | ☑ | **überholt**: durch den Kommentar im Issue (Texte gekürzt) und FEA-53 (Texte zur Anlagestrategie). | kein Test | – |
| FEA-18/K1 (Kommentar) | Unter der Tabelle steht keine Zeile «Gewählt:» mehr; ein Fehler zur Reihe wird weiterhin angezeigt. | ☑ | **überholt**: durch FEA-53: keine Tabelle der Reihen mehr. | kein Test | – |
| FEA-18/K2 (Kommentar) | Jede Reihe hat ein ⓘ; Maus, Tastaturfokus und Klick öffnen das Popup, Esc schliesst es. | ☑ | **überholt**: durch FEA-53: keine Tabelle der Reihen mehr; das Popup gibt es je Anlagestrategie. | kein Test | – |
| FEA-18/K3 (Kommentar) | Jeder Popup-Text hat höchstens 2 Zeilen. | ☑ | **unklar**: Gilt die Grenze von 2 Zeilen auch für die Texte der Anlagestrategie? | kein Test | – |
| FEA-18/K4 (Kommentar) | Die Texte der vier mitgelieferten Reihen entsprechen dem Vorschlag oder der abgestimmten Fassung. | ☑ | **überholt**: durch FEA-53 und FEA-72: Die vier Reihen sind nicht mehr einzeln wählbar, Pictet ist entfernt. | kein Test | – |
| FEA-18/K5 (Kommentar) | Die vollständigen Quellenangaben (mit Stand der Daten) bleiben in den Dateien erhalten. | ☑ | **gilt** | kein Test | – |
| FEA-18/K6 (Kommentar) | Test und Hilfe nachgeführt. | ☑ | **gilt** | kein Test | – |

### #20 FEA-19 Optimierung: Ziel wählbar

[Issue #20](https://github.com/ingen63/VivaPlan/issues/20) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/optimize_goal.rs:1](../crates/engine/tests/optimize_goal.rs#L1), [ui.spec.js:2176](../e2e/ui.spec.js#L2176)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-19/AK1 | Die Optimierung hat die Auswahl «Ziel»; Standard ist Ziel 1; die Ergebnisse für Ziel 1 sind identisch mit denen vor der Änderung (Regressionstest). | ☑ | **gilt** | kein Test | – |
| FEA-19/AK2 | Bei Ziel 2 zeigt die Ergebnistabelle je Strategie den Anteil Läufe mit ungeplantem Immobilienverkauf; die Reihenfolge richtet sich nach diesem Anteil, bei Gleichstand nach der Ruinwahrscheinlichkeit. | ☑ | **gilt** | kein Test | – |
| FEA-19/AK3 | Strategien mit Konsum unter K % der geplanten Ausgaben werden nicht als beste gewählt. | ☑ | **gilt** | kein Test | – |
| FEA-19/AK4 | Mit `examples/Test 1.rplan` liefert Ziel 2 eine Strategie mit höchstens so vielen ungeplanten Verkäufen wie Ziel 1. | ☑ | **gilt**: Die Planung `examples/Test 1.rplan` gibt es nicht mehr (GEN-01); prüfen mit der Beispielplanung `beispiel.vplan` oder einer erfundenen Planung. | kein Test | – |
| FEA-19/AK5 | Planungen ohne Immobilien: Ziel 2 ist nicht wählbar (mit Hinweis). | ☑ | **gilt** | kein Test | – |
| FEA-19/AK6 | Das Ziel steht in der Plandatei und in der Hilfe. | ☑ | **gilt** | kein Test | – |

### #21 FEA-20 Variationen ohne feste Obergrenze, Warnung ab 60 s

[Issue #21](https://github.com/ingen63/VivaPlan/issues/21) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui.spec.js:1363](../e2e/ui.spec.js#L1363)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-20/AK1 | Mit 1'001 Kombinationen (bisher abgelehnt) wird die Berechnung angeboten. | ☑ | **gilt** | kein Test | – |
| FEA-20/AK2 | Bei geschätzten 90 s erscheint die Warnung mit Dauer und Läufen; «Abbrechen» startet nichts, «Trotzdem starten» startet. | ☑ | **gilt** | kein Test | – |
| FEA-20/AK3 | Bei geschätzten 30 s keine Warnung. | ☑ | **gilt** | kein Test | – |
| FEA-20/AK4 | Die Zusammenfassung der Variationen nennt keine Grenze mehr ausser der technischen. | ☑ | **gilt** | kein Test | – |
| FEA-20/AK5 | 1'000'001 Kombinationen werden mit der technischen Meldung abgelehnt, ohne die App zu belasten. | ☑ | **gilt** | kein Test | – |
| FEA-20/AK6 | Abbrechen bleibt während der Berechnung möglich. | ☑ | **gilt** | kein Test | – |

### #22 FEA-21 Legenden: gestrichelte Linien

[Issue #22](https://github.com/ingen63/VivaPlan/issues/22) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui.spec.js:1446](../e2e/ui.spec.js#L1446)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-21/AK1 | In «Bandbreite des Gesamtvermögens» zeigt die Legende «1. und 99. Perzentil» als gestricheltes Linien-Symbol in der Farbe des Diagramms. | ☑ | **gilt** | kein Test | – |
| FEA-21/AK2 | Die Legende der Verteilung des Endvermögens nennt die fünf Perzentil-Linien mit gestricheltem Symbol. | ☑ | **gilt** | kein Test | – |
| FEA-21/AK3 | Übersicht und Grafiken zeigen dieselben Legenden. | ☑ | **gilt** | kein Test | – |
| FEA-21/AK4 | Jede Linie in den beiden Diagrammen hat einen Legendeneintrag und umgekehrt. | ☑ | **gilt** | kein Test | – |
| FEA-21/AK5 | Die Legenden sind im hellen und dunklen Erscheinungsbild lesbar. | ☑ | **gilt** | kein Test | – |

### #23 FEA-22 Übersicht: Immobilienverkäufe statt Notverkäufe und Einmalzahlungen

[Issue #23](https://github.com/ingen63/VivaPlan/issues/23) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui.spec.js:934](../e2e/ui.spec.js#L934), [ui.spec.js:1331](../e2e/ui.spec.js#L1331)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-22/AK1 | Die Übersicht hat keine Karte und keine Spalte «Einmalzahlungen». | ☑ | **gilt** | kein Test | – |
| FEA-22/AK2 | Die Spalte heisst «Immobilienverkäufe» und zeigt pro Verkauf eine Zeile «Haus: 05.2034». | ☑ | **gilt** | kein Test | – |
| FEA-22/AK3 | Ein Notverkauf trägt das Warnsymbol, ein geplanter Verkauf nicht. | ☑ | **gilt** | kein Test | – |
| FEA-22/AK4 | Ein Lauf mit zwei Verkäufen zeigt zwei Zeilen. | ☑ | **gilt** | kein Test | – |
| FEA-22/AK5 | Die Kennzahlkarte «Notverkäufe» zeigt die Zahlen wie bisher. | ☑ | **gilt** | kein Test | – |
| FEA-22/AK6 | Planungen ohne Immobilien zeigen die Spalte nicht. | ☑ | **gilt** | kein Test | – |
| FEA-22/AK7 | Zahlen und Daten stimmen mit den Immobilienereignissen des jeweiligen Perzentil-Laufs überein. | ☑ | **gilt** | kein Test | – |

### #24 FEA-23 Berechnete Variationen in «Einzelne Läufe» und «Jahreswerte» wählbar

[Issue #24](https://github.com/ingen63/VivaPlan/issues/24) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/rerun_paths.rs:1](../crates/engine/tests/rerun_paths.rs#L1), [ui.spec.js:2140](../e2e/ui.spec.js#L2140)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-23/AK1 | Ein Monte-Carlo-Ergebnis mit zwei Variationen zeigt in «Einzelne Läufe» und «Jahreswerte» zwei Auswahlfelder mit den berechneten Stufen. | ☑ | **gilt** | kein Test | – |
| FEA-23/AK2 | Die Auswahl «Aktienanteil 30 %, Kapitalbezug 60 %» zeigt den 50.-Perzentil-Lauf dieser Stufe; sein Endvermögen stimmt mit dem Wert in der Szenarientabelle überein. | ☑ | **gilt** | kein Test | – |
| FEA-23/AK3 | Der Wechsel dauert wenige Sekunden und löst keinen vollständigen Durchlauf aus. | ☑ | **gilt** | kein Test | – |
| FEA-23/AK4 | Die Auswahl wirkt in beiden Reitern; die anderen Reiter zeigen Planwerte mit Vermerk. | ☑ | **gilt** | kein Test | – |
| FEA-23/AK5 | Bei gespeicherten Ergebnissen funktioniert die Auswahl; bei abweichendem Endvermögen erscheint der Hinweis. | ☑ | **gilt** | kein Test | – |
| FEA-23/AK6 | Der Einzellauf mit Variationen funktioniert wie bisher. | ☑ | **gilt** | kein Test | – |

### #25 FEA-24 Einstellungen (Grenzen AHV-/PK-Bezug, Maximalbetrag 3a)

[Issue #25](https://github.com/ingen63/VivaPlan/issues/25) · geschlossen · Umsetzung: `477381f` (altes Repository) · Tests zum Issue ohne Kriterium: [engine/src/upgrade.rs:488](../crates/engine/src/upgrade.rs#L488), [engine/src/validate.rs:1475](../crates/engine/src/validate.rs#L1475) (+3)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-24/AK1 | Die App hat einen erreichbaren Bereich «Einstellungen» mit den fünf Einstellungen der Tabelle, je mit Beschriftung, Einheit und Standardwert. | ☑ | **gilt**: Laut Kommentar fünf Einstellungen (die Schwelle von FEA-12 entfällt). | [engine/src/settings.rs:69](../crates/engine/src/settings.rs#L69), [ui.spec.js:2297](../e2e/ui.spec.js#L2297) | – |
| FEA-24/AK2 | Eine Änderung wird gespeichert und gilt nach einem Neustart der App weiterhin. Ohne gespeicherten Wert gilt der Standardwert. | ☑ | **gilt** | [ui.spec.js:2297](../e2e/ui.spec.js#L2297) | – |
| FEA-24/AK3 | Eine Schaltfläche «Standardwerte» setzt alle Einstellungen (nach Rückfrage) auf die Standardwerte zurück. | ☑ | **gilt** | [ui.spec.js:2297](../e2e/ui.spec.js#L2297) | – |
| FEA-24/AK4 | Prüfung der Eingabe: Die untere Grenze ist höchstens so gross wie die obere Grenze. Alter sind ganze Jahre. Der Maximalbetrag ist 0 oder mehr. Ein ungültiger Wert wird mit klarer Meldung abgelehnt und nicht gespeichert (Beispiel: untere Grenze AHV 71 bei oberer Grenze 70). | ☑ | **gilt** | [engine/src/settings.rs:83](../crates/engine/src/settings.rs#L83), [ui.spec.js:2297](../e2e/ui.spec.js#L2297) | – |
| FEA-24/AK5 | Die Grenzen für AHV-Bezug und Bezug Pensionskasse (FEA-38) und der Maximalbetrag der 3a (FEA-37) werden an einer Stelle aus den Einstellungen gelesen und dem Rechenkern als Parameter übergeben. Der Rechenkern kennt die Standardwerte als Rückfall (für Kommandozeile und Tests). | ☑ | **gilt** | [engine/tests/milestones_limits.rs:169](../crates/engine/tests/milestones_limits.rs#L169) | – |
| FEA-24/AK6 | Beispiel: Obere Grenze Bezug Pensionskasse 67 statt 65: Ein Bezug mit 67 Jahren ist gültig, mit 68 wird er abgelehnt. Der Standard 65 des Meilensteins ändert sich dadurch nicht. | ☑ | **gilt** | [engine/tests/milestones_limits.rs:169](../crates/engine/tests/milestones_limits.rs#L169), [ui.spec.js:2332](../e2e/ui.spec.js#L2332) | – |
| FEA-24/AK7 | Die Einstellungen ändern keine Planungsdatei und kein gespeichertes Ergebnis. Liegt eine Planung nach einer Änderung ausserhalb der neuen Grenzen, meldet die Prüfung das mit klarer Meldung (siehe FEA-38). | ☑ | **gilt** | kein Test | – |
| FEA-24/AK8 | Hilfe, `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test | – |

### #26 GEN-01 Repository öffentlich machen (Daten- und Plandateien bereinigen)

[Issue #26](https://github.com/ingen63/VivaPlan/issues/26) · geschlossen · Umsetzung: nicht vermerkt

Keine Kriterien im Issue; die Aufgaben stehen in den Kommentaren.

### #27 ISS-6 Beenden mit ungespeicherten Änderungen fragt nicht nach

[Issue #27](https://github.com/ingen63/VivaPlan/issues/27) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui.spec.js:2111](../e2e/ui.spec.js#L2111)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| ISS-6/AK1 | Schliessen-Knopf mit Änderungen: Dialog erscheint, «Abbrechen» lässt die App offen. | ☑ | **gilt** | kein Test | – |
| ISS-6/AK2 | ⌘Q mit Änderungen: dasselbe Verhalten. | ☑ | **gilt** | kein Test | – |
| ISS-6/AK3 | «Speichern» speichert (bei neuer Planung mit Ortsabfrage) und beendet; bricht die Ortsabfrage ab, bleibt die App offen. | ☑ | **gilt** | kein Test | – |
| ISS-6/AK4 | «Verwerfen» beendet ohne Speichern. | ☑ | **gilt** | kein Test | – |
| ISS-6/AK5 | Ohne Änderungen schliesst die App ohne Dialog. | ☑ | **gilt** | kein Test | – |
| ISS-6/AK6 | Test für beide Wege (Fenster schliessen, ⌘Q). | ☑ | **gilt** | kein Test | – |

### #28 ISS-5 Immobilien: Verkauf auch bei «Kauf geplant» erfassen

[Issue #28](https://github.com/ingen63/VivaPlan/issues/28) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui.spec.js:2271](../e2e/ui.spec.js#L2271)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| ISS-5/AK1 | Bei «Kauf geplant» erscheint das Feld «Verkauf ab» mit Vorgabe «Nie». | ☑ | **gilt** | kein Test | – |
| ISS-5/AK2 | Datum und Meilenstein sind wählbar; die Simulation verkauft das Objekt zum gewählten Zeitpunkt (Ereignis «Verkauf»). | ☑ | **gilt** | kein Test | – |
| ISS-5/AK3 | Verkauf vor oder im Monat des Kaufs: Fehler am Feld, Start blockiert (Engine und Oberfläche). | ☑ | **gilt** | kein Test | – |
| ISS-5/AK4 | Gleiche Prüfung bei «Besitz» gegenüber dem Kaufdatum. | ☑ | **gilt** | kein Test | – |
| ISS-5/AK5 | Bei Kauf «nach Verkauf von …» keine Fehlermeldung zum Datum. | ☑ | **gilt** | kein Test | – |
| ISS-5/AK6 | Testfall in der Engine für die Prüfung, Oberflächentest für das Feld. | ☑ | **gilt** | kein Test | – |
| ISS-5/AK7 | Hilfe (`app/ui/hilfe/`) und Spezifikation sind nachgeführt. | ☑ | **gilt**: Pfad heute `crates/app/ui/hilfe/` (FEA-35). | kein Test | – |

### #29 ISS-3 «Neu» setzt die Planung nicht zurück

[Issue #29](https://github.com/ingen63/VivaPlan/issues/29) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui.spec.js:92](../e2e/ui.spec.js#L92), [ui.spec.js:2207](../e2e/ui.spec.js#L2207)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| ISS-3/AK1 | «Neu» setzt alle Felder auf Standardwerte oder leer. | ☑ | **gilt** | kein Test | – |
| ISS-3/AK2 | Mit ungespeicherten Änderungen erscheint die Rückfrage; «Abbrechen» ändert nichts. | ☑ | **gilt** | kein Test | – |
| ISS-3/AK3 | Der Dateiname im Kopf zeigt «Neue Planung», das Speichern fragt nach einem Pfad. | ☑ | **gilt** | kein Test | – |
| ISS-3/AK4 | Oberflächentest vorhanden. | ☑ | **gilt** | kein Test | – |

### #30 ISS-4 Meilenstein «Verkauf Haus» lässt sich nicht löschen

[Issue #30](https://github.com/ingen63/VivaPlan/issues/30) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui.spec.js:374](../e2e/ui.spec.js#L374), [ui.spec.js:387](../e2e/ui.spec.js#L387)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| ISS-4/AK1 | `Neu.rplan`: «Verkauf Haus» löschen fragt nach und nennt 2 Eingaben im Abschnitt Immobilien. | ☑ | **gilt**: Die Planung `Neu.rplan` mit «Verkauf Haus» gibt es nicht mehr (heute `Neu.vplan` ohne diesen Meilenstein); prüfen mit einer erfundenen Planung. | kein Test | – |
| ISS-4/AK2 | Nach «Löschen und umstellen» tragen Verkauf und Kauf das feste Datum 06.2041. | ☑ | **gilt**: Wie AK1: braucht eine erfundene Planung mit dem Meilenstein. | kein Test | – |
| ISS-4/AK3 | Ein Meilenstein ohne Verwendung lässt sich ohne Rückfrage löschen. | ☑ | **gilt** | kein Test | – |

### #31 FEA-25 Pflicht: Grundstückgewinnsteuer bei Immobilien

[Issue #31](https://github.com/ingen63/VivaPlan/issues/31) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-25/AK1 | Immobilie «Besitz» oder «Kauf geplant» ohne Grundstückgewinnsteuer: Fehler im Abschnitt Steuern mit Meldung, Start blockiert (Engine und Oberfläche). | ☑ | **gilt** | kein Test | – |
| FEA-25/AK2 | Nur Mietobjekte oder keine Immobilie: kein Fehler. | ☑ | **gilt** | kein Test | – |
| FEA-25/AK3 | Mit Vorlage oder eigenen Regeln (auch Satz 0 %): kein Fehler. | ☑ | **gilt** | kein Test | – |
| FEA-25/AK4 | Die Pflichtliste (FEA-13) in der Spezifikation und die Hilfe nennen die neue Pflicht. | ☑ | **gilt** | kein Test | – |
| FEA-25/AK5 | Test in der Engine und in der Oberfläche. | ☑ | **gilt** | kein Test | – |

### #32 FEA-26 Immobilien: einheitliche Feldreihenfolge

[Issue #32](https://github.com/ingen63/VivaPlan/issues/32) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui.spec.js:2236](../e2e/ui.spec.js#L2236)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-26/AK1 | Reihenfolge und Plätze der Felder sind bei «Besitz» und «Kauf geplant» gleich. | ☑ | **gilt** | [ui.spec.js:2235](../e2e/ui.spec.js#L2235) | – |
| FEA-26/AK2 | Der Marktwert steht bei «Besitz» in der Kopfzeile neben dem Namen und fehlt bei «Kauf geplant». | ☑ | **gilt** | [ui.spec.js:2235](../e2e/ui.spec.js#L2235) | – |
| FEA-26/AK3 | Beim Statuswechsel bleiben Name, Kaufpreis, Verkauf und Nebenkosten an ihrem Platz. | ☑ | **gilt** | [ui.spec.js:2235](../e2e/ui.spec.js#L2235) | – |
| FEA-26/AK4 | Alle Felder bleiben erreichbar, erfasste Werte sind unverändert. | ☑ | **gilt** | [ui.spec.js:2235](../e2e/ui.spec.js#L2235) | – |
| FEA-26/AK5 | Schmales Fenster: Marktwert unter dem Namen. | ☑ | **gilt** | [ui.spec.js:2235](../e2e/ui.spec.js#L2235) | – |
| FEA-26/AK6 | Oberflächentest prüft die Reihenfolge. | ☑ | **gilt** | [ui.spec.js:2235](../e2e/ui.spec.js#L2235) | – |
| FEA-26/AK7 | Hilfe (`app/ui/hilfe/`) ist nachgeführt. | ☑ | **gilt**: Pfad heute `crates/app/ui/hilfe/` (FEA-35). | kein Test | – |

### #33 ISS-7 Mixed findings

[Issue #33](https://github.com/ingen63/VivaPlan/issues/33) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [optimum.spec.js:50](../e2e/optimum.spec.js#L50), [optimum.spec.js:55](../e2e/optimum.spec.js#L55) (+1)

Sammelbefund ohne Kriterien; die Punkte sind in ISS-10 (#37) aufgenommen (dort AK3).

### #34 ISS-7 Optimum «≥ 100 %» bei Anteilen irreführend

[Issue #34](https://github.com/ingen63/VivaPlan/issues/34) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [optimum.spec.js:50](../e2e/optimum.spec.js#L50), [optimum.spec.js:55](../e2e/optimum.spec.js#L55) (+1)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| ISS-7/AK1 | Kapitalbezug Pensionskasse, bester Wert 100 %: Anzeige «100 %», nicht «≥ 100 %». | ☑ | **gilt** | [optimum.spec.js:54](../e2e/optimum.spec.js#L54) | – |
| ISS-7/AK2 | Bester Wert 0 %: Anzeige «0 %», nicht «≤ 0 %». | ☑ | **gilt** | [optimum.spec.js:54](../e2e/optimum.spec.js#L54) | – |
| ISS-7/AK3 | Anteil mit Bereich kleiner als 0 bis 100 % (zum Beispiel 0 bis 50 %), bester Wert 50 %: weiterhin «≥ 50 %». | ☑ | **gilt** | [optimum.spec.js:54](../e2e/optimum.spec.js#L54) | – |
| ISS-7/AK4 | Ausgaben-Faktor am oberen Rand: weiterhin «≥». | ☑ | **gilt** | [optimum.spec.js:54](../e2e/optimum.spec.js#L54) | – |
| ISS-7/AK5 | Die Erklärung unter der Tabelle ist angepasst. | ☑ | **gilt** | kein Test | – |
| ISS-7/AK6 | Test für `Optimum` und die Anzeige. | ☑ | **gilt** | [optimum.spec.js:54](../e2e/optimum.spec.js#L54) | – |

### #35 ISS-8 coverage.yml: Tauri-Abhängigkeiten fehlen auf dem Runner

[Issue #35](https://github.com/ingen63/VivaPlan/issues/35) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| ISS-8/AK1 | Ein Lauf von `coverage.yml` auf GitHub ist grün (Engine und App). | ☑ | **gilt** | kein Test | – |
| ISS-8/AK2 | Die Abhängigkeiten stehen an einer Stelle und werden von `coverage.yml` und `pakete.yml` gemeinsam verwendet, oder sind in beiden gleich. | ☑ | **gilt** | kein Test | – |
| ISS-8/AK3 | Sinkt die Abdeckung von Engine oder App unter 80 %, schlägt der Lauf fehl (einmal geprüft). | ☑ | **gilt** | kein Test | – |
| ISS-8/AK4 | Die Laufzeit des Workflows ist im Issue vermerkt. | ☑ | **gilt** | kein Test | – |

### #36 ISS-9 Einheitliche Mindestabdeckung 80 % je Paket

[Issue #36](https://github.com/ingen63/VivaPlan/issues/36) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| ISS-9/AK1 | `test-quality.yml` prüft 80 % je Paket statt 92 % über alles. | ☑ | **gilt** | kein Test | – |
| ISS-9/AK2 | Hook, `coverage.yml`, `test-quality.yml`, `CLAUDE.md` und `TESTS.md` nennen dieselbe Schwelle. | ☑ | **gilt** | kein Test | – |
| ISS-9/AK3 | Ein Pull Request, der Hook und `coverage.yml` besteht, besteht auch `test-quality.yml` bei der Abdeckung. | ☑ | **überholt**: durch die Arbeitsweise in `CLAUDE.md`: keine Pull Requests mehr, gearbeitet wird direkt auf `main`; sinngemäss für den Push prüfen. | kein Test | – |
| ISS-9/AK4 | Sinkt ein Paket unter 80 %, schlägt `test-quality.yml` fehl (einmal geprüft). | ☑ | **gilt** | kein Test | – |

### #37 ISS-10 Abnahme-Befunde: Jahreswerte, Meilensteine, Texte

[Issue #37](https://github.com/ingen63/VivaPlan/issues/37) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui.spec.js:284](../e2e/ui.spec.js#L284), [ui.spec.js:318](../e2e/ui.spec.js#L318) (+2)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| ISS-10/AK1 | Spalte «Entnahme» steht nach «Ausgaben total», «Entnahmerate» nach «Einkommen total». | ☑ | **gilt** | kein Test | – |
| ISS-10/AK2 | Der Knopf «Steuern anzeigen/ausblenden» ist sichtbar und blendet die fünf Steuerspalten ein und aus. | ☑ | **gilt** | kein Test | – |
| ISS-10/AK3 | Punkte 1 bis 12 sind umgesetzt oder durch Rückfrage geklärt. | ☑ | **gilt** | kein Test | – |
| ISS-10/AK4 | Hilfe (`app/ui/hilfe/`) und Spezifikation sind nachgeführt, wo sich Texte oder Spalten ändern. | ☑ | **gilt**: Pfad heute `crates/app/ui/hilfe/` (FEA-35). | kein Test | – |
| ISS-10/AK5 | Ein Oberflächentest je Änderung der Tabelle, der Meilensteine und der Warnung. | ☑ | **gilt** | kein Test | – |

### #38 FEA-27 Test-Agent: Mutationstest statt nur Abdeckung als Ziel

[Issue #38](https://github.com/ingen63/VivaPlan/issues/38) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-27/AK1 | Die Anweisung `.claude/agents/rust-test-agent.md` enthält den Mutationslauf, das neue Ziel und das Verbot von Tests ohne Prüfung. | ☑ | **gilt** | kein Test | – |
| FEA-27/AK2 | Ein Beispiellauf mit einer absichtlich schwach getesteten Änderung im Rechenkern meldet einen missed Mutanten, und der Agent ergänzt einen Test, der ihn tötet. | ☑ | **gilt** | kein Test | – |
| FEA-27/AK3 | Der Bericht des Agenten listet verbleibende missed Mutanten. | ☑ | **gilt** | kein Test | – |
| FEA-27/AK4 | Die Laufzeit des pre-push-Hooks ist gemessen und im Issue vermerkt. | ☑ | **gilt** | kein Test | – |
| FEA-27/AK5 | `TESTS.md` beschreibt das neue Vorgehen. | ☑ | **gilt** | kein Test | – |

### #39 FEA-28 Abdeckung des geänderten Codes im Pull Request anzeigen

[Issue #39](https://github.com/ingen63/VivaPlan/issues/39) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-28/AK1 | Ein Pull Request mit neuem, ungetestetem Code zeigt in der Lauf-Zusammenfassung einen Diff-Abdeckungswert unter 100 % und die nicht abgedeckten Zeilen. | ☑ | **gilt**: Pull Requests werden nicht mehr verwendet (`CLAUDE.md`); sinngemäss für den Push und lokal mit `make coverage-diff` prüfen. | kein Test | – |
| FEA-28/AK2 | Ein Pull Request ohne Rust-Änderungen zeigt «keine Rust-Änderungen». | ☑ | **gilt**: Wie AK1. | kein Test | – |
| FEA-28/AK3 | Die Diff-Abdeckung lässt den Lauf nicht fehlschlagen. | ☑ | **gilt** | kein Test | – |
| FEA-28/AK4 | Der lokale Befehl liefert dasselbe Ergebnis. | ☑ | **gilt** | kein Test | – |
| FEA-28/AK5 | `TESTS.md` und `CLAUDE.md` nennen die Diff-Abdeckung und dass die 80 % eine Untergrenze sind. | ☑ | **gilt** | kein Test | – |

### #40 FEA-29 Nicht erkannte Mutanten im Rechenkern einzeln beurteilen

[Issue #40](https://github.com/ingen63/VivaPlan/issues/40) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/src/api.rs:605](../crates/engine/src/api.rs#L605), [engine/src/batch.rs:398](../crates/engine/src/batch.rs#L398) (+16)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-29/AK1 | Ein voller Lauf `make mutants` auf `pensionsplaner-engine` ist ausgewertet; jeder missed Mutant ist getestet oder begründet ausgeschlossen. | ☑ | **gilt**: Das Paket heisst seit FEA-34 `vivaplan-engine`. | kein Test | – |
| FEA-29/AK2 | Jeder Ausschluss trägt eine Begründung im Code oder in `.cargo/mutants.toml`. | ☑ | **gilt** | kein Test | – |
| FEA-29/AK3 | Die Liste der offenen missed Mutanten ist dokumentiert (Ort in `TESTS.md` genannt). | ☑ | **gilt** | kein Test | – |
| FEA-29/AK4 | `TESTS.md` beschreibt die Regel «einzeln beurteilen, kein Score-Ziel» und die Durchsicht pro Version. | ☑ | **gilt** | kein Test | – |

### #41 FEA-30 Übersichtsseite Testqualität (Stufe 1: Hygiene und Angemessenheit)

[Issue #41](https://github.com/ingen63/VivaPlan/issues/41) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-30/AK1 | `make report` erzeugt die Seite nach einem Lauf von `make test` und `make coverage`; sie öffnet sich ohne Netzwerk. | ☑ | **gilt** | kein Test | – |
| FEA-30/AK2 | Mit einem absichtlich fehlschlagenden Test zeigt Hygiene «nicht erfüllt» und nennt den Test. | ☑ | **gilt** | kein Test | – |
| FEA-30/AK3 | Mit einem vorhandenen `mutants.out/` stehen die missed Mutanten einzeln mit Datei und Zeile auf der Seite. | ☑ | **gilt** | kein Test | – |
| FEA-30/AK4 | Ohne Mutationslauf zeigt der Bereich «keine Daten» statt eines Fehlers. | ☑ | **gilt** | kein Test | – |
| FEA-30/AK5 | Es gibt keine Gesamtnote; jede Schicht hat ihren eigenen Status mit genannter Regel. | ☑ | **gilt** | kein Test | – |
| FEA-30/AK6 | Ein Lauf von `test-quality.yml` enthält die Seite als Artefakt und die Kurzfassung in der Zusammenfassung. | ☑ | **gilt** | kein Test | – |
| FEA-30/AK7 | `TESTS.md` beschreibt die Seite, den Befehl und die Statusregeln. | ☑ | **gilt** | kein Test | – |

### #42 FEA-31 Übersichtsseite Testqualität (Stufe 2: Anforderungsbezug und Ergebnis)

[Issue #42](https://github.com/ingen63/VivaPlan/issues/42) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-31/AK1 | Ein Issue mit drei Kriterien, davon zwei mit gekennzeichnetem Test, erscheint mit 2 von 3 belegt. | ☑ | **gilt** | kein Test | – |
| FEA-31/AK2 | Ein Bug-Issue mit Label `gefunden:nutzung` erscheint unter «Ergebnis» mit Ursache und Regressionstest. | ☑ | **gilt** | kein Test | – |
| FEA-31/AK3 | Der Vergleich zur vorherigen Version zeigt Veränderungen je Kennzahl. | ☑ | **gilt** | kein Test | – |
| FEA-31/AK4 | Ohne `gh`-Anmeldung erscheinen die beiden Schichten als «keine Daten», die übrige Seite bleibt vollständig. | ☑ | **gilt** | kein Test | – |
| FEA-31/AK5 | Labels und Kennzeichnung sind in `CLAUDE.md`, `TESTS.md` und den Issue-Vorlagen beschrieben. | ☑ | **gilt** | kein Test | – |

### #43 FEA-32 Mutationstests beschleunigen (Timeout, schwere Tests, Shards)

[Issue #43](https://github.com/ingen63/VivaPlan/issues/43) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/all.rs:1](../crates/engine/tests/all.rs#L1), [engine/tests/all.rs:54](../crates/engine/tests/all.rs#L54)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-32/AK1 | `.cargo/mutants.toml` setzt den Timeout; ein Mutant mit Endlosschleife bricht nach der Höchstdauer ab. | ☑ | **gilt** | kein Test | – |
| FEA-32/AK2 | Mutationsläufe verwenden das eigene nextest-Profil ohne die schweren Tests; `make test` führt sie weiterhin aus. | ☑ | **gilt** | kein Test | – |
| FEA-32/AK3 | Voller Lauf lokal (`make mutants`): Wanduhrzeit vorher und nachher im Issue, Ziel unter 8 h. | ☑ | **gilt** | kein Test | – |
| FEA-32/AK4 | `--in-diff` für eine mittlere Änderung (50–100 Zeilen im Rechenkern) dauert lokal unter 10 min. | ☑ | **gilt** | kein Test | – |
| FEA-32/AK5 | `mutation-full.yml` läuft auf GitHub mit allen Shards innerhalb der Zeitgrenze durch. | ☑ | **gilt** | kein Test | – |
| FEA-32/AK6 | Vergleich vorher/nachher: Anteil erkannter Mutanten und Liste neu nicht erkannter Mutanten im Issue. | ☑ | **gilt** | kein Test | – |
| FEA-32/AK7 | `TESTS.md` beschreibt Timeout, Profil und ausgenommene Tests. | ☑ | **gilt** | kein Test | – |

### #44 FEA-34 App umbenennen: RetirementPlanner → VivaPlan (im bestehenden Repository, GitHub-Repository am Schluss umbenennen)

[Issue #44](https://github.com/ingen63/VivaPlan/issues/44) · geschlossen · Umsetzung: `0b67d93` (altes Repository) · Tests zum Issue ohne Kriterium: [engine/tests/legacy_format.rs:1](../crates/engine/tests/legacy_format.rs#L1), [engine/tests/legacy_format.rs:102](../crates/engine/tests/legacy_format.rs#L102) (+2)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-34/AK1 | Die Suche ohne Beachtung der Gross- und Kleinschreibung nach `retirement`, `rplan` und `pensionsplaner` findet nur noch markierte `legacy`-Stellen und die im Bericht genannten fachlichen Begriffe. | ☑ | **gilt** | kein Test | – |
| FEA-34/AK2 | Die App startet als «VivaPlan»: Dock, Programmmenü, About-Dialog, Fenstertitel `VivaPlan - ‹Dateiname›`. | ☑ | **gilt** | [app/src/main.rs:1791](../crates/app/src/main.rs#L1791) | – |
| FEA-34/AK3 | Eine `.rplan`-Datei öffnet mit identischen Werten; «Speichern unter» schlägt `.vplan` vor; die `.vplan` öffnet wieder und ergibt dieselben Ergebnisse; die `.rplan` ist unverändert (Prüfsumme). | ☑ | **gilt** | [app/src/main.rs:1731](../crates/app/src/main.rs#L1731), [engine/tests/legacy_format.rs:82](../crates/engine/tests/legacy_format.rs#L82) | – |
| FEA-34/AK4 | Beim ersten Start werden die App-Daten aus dem alten Datenordner übernommen; der alte Ordner existiert danach noch. | ☑ | **gilt** | [app/src/migrate.rs:92](../crates/app/src/migrate.rs#L92) | – |
| FEA-34/AK5 | Ein Doppelklick auf eine `.vplan`-Datei im Finder öffnet VivaPlan. | ☑ | **gilt** | [app/src/main.rs:1803](../crates/app/src/main.rs#L1803) | – |
| FEA-34/AK6 | Alle Tests sind grün (`make test`, Oberflächentests, Rauchtest), Golden-Ergebnisse unverändert, ein Release-Build läuft durch, alle GitHub-Workflows laufen mit den neuen Paketnamen. | ☑ | **gilt** | kein Test | – |
| FEA-34/AK7 | Die Git-Historie umbenannter Dateien bleibt erhalten (`git log --follow`). | ☑ | **überholt**: durch GEN-01: Das öffentliche Repository beginnt mit einem einzigen Commit; die Historie liegt nur im alten, privaten Repository. | kein Test | – |
| FEA-34/AK8 | Das GitHub-Repository heisst `VivaPlan`, `origin` zeigt auf die neue Adresse, Push und Workflows funktionieren, im Repository verweist nichts mehr auf `RetirementPlanning` (ausser Weiterleitungs-Hinweis, falls gewünscht). | ☑ | **gilt**: Seit GEN-01 ist `ingen63/VivaPlan` ein neues Repository; das alte heisst wieder `RetirementPlanning` und bleibt privat. | kein Test | – |

### #45 Verwaltung, nicht öffentlich

[Issue #45](https://github.com/ingen63/VivaPlan/issues/45) · geschlossen, nicht geplant · Umsetzung: nicht vermerkt

Platzhalter «Verwaltung, nicht öffentlich», kein Inhalt.

### #46 ISS-11 Nächtlicher Fuzz-Job baut für musl und bricht ab

[Issue #46](https://github.com/ingen63/VivaPlan/issues/46) · geschlossen · Umsetzung: `b5c8990` (altes Repository)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| ISS-11/AK1 | Der Job `fuzz` übergibt das Ziel-Triple des Hosts ausdrücklich (`--target`), unabhängig davon, welche Fassung von cargo-fuzz installiert ist. | ☑ | **gilt** | kein Test | – |
| ISS-11/AK2 | Im nächsten nächtlichen Lauf (oder einem von Hand gestarteten Lauf) laufen `parse_csv`, `parse_snb` und `validate_plan` je 5 Minuten ohne Baufehler. | ☑ | **gilt** | kein Test | – |
| ISS-11/AK3 | Die Ursache ist im Workflow kommentiert. | ☑ | **gilt** | kein Test | – |

### #47 FEA-33 Fixe Rate, VPW und Endowment ab erster Pensionierung

[Issue #47](https://github.com/ingen63/VivaPlan/issues/47) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/buckets_guardrails.rs:1](../crates/engine/tests/buckets_guardrails.rs#L1), [engine/tests/buckets_guardrails.rs:413](../crates/engine/tests/buckets_guardrails.rs#L413)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-33/AK1 | Bei zwei Personen mit verschiedenem Pensionierungsdatum legen «Fixe Rate», «VPW» und «Endowment» die jährliche Entnahme im Monat der ersten Pensionierung fest (Rate × liquides Vermögen; bei VPW mit dem Alter der Hauptperson in diesem Monat). Ab diesem Monat gilt die Lebenshaltung der Strategie. | ☐ | **gilt** | [engine/tests/buckets_guardrails.rs:416](../crates/engine/tests/buckets_guardrails.rs#L416) | – |
| FEA-33/AK2 | Ein noch laufendes Erwerbseinkommen der zweiten Person zählt zu «Einkommen und Renten»: Lebenshaltung = Entnahme + Einkommen und Renten − Wohnkosten, wie bei Guardrails. | ☐ | **gilt** | [engine/tests/buckets_guardrails.rs:416](../crates/engine/tests/buckets_guardrails.rs#L416) | – |
| FEA-33/AK3 | Die Unter- und die Obergrenze der Lebenshaltung gelten ab demselben Monat. | ☐ | **gilt** | [engine/tests/buckets_guardrails.rs:416](../crates/engine/tests/buckets_guardrails.rs#L416) | – |
| FEA-33/AK4 | Bei einer Person und bei zwei Personen mit gleichem Pensionierungsdatum ändert sich nichts (Regressionstest, Golden-Dateien unverändert). | ☐ | **gilt** | [engine/tests/buckets_guardrails.rs:452](../crates/engine/tests/buckets_guardrails.rs#L452) | – |
| FEA-33/AK5 | Die Hilfe (Absatz «Wann gilt die Strategie?» und die Stellen der drei Strategien) und `BESCHREIBUNG.md` nennen den neuen Start. | ☐ | **gilt** | kein Test | – |

### #48 FEA-35 Projektstruktur vereinfachen (Workspace mit crates/, docs/, tools/)

[Issue #48](https://github.com/ingen63/VivaPlan/issues/48) · geschlossen · Umsetzung: `faba2aa` (altes Repository)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-35/AK1 | `make test` läuft grün, die Golden-Dateien sind unverändert | ☑ | **gilt** | kein Test | – |
| FEA-35/AK2 | Die Abdeckung von Engine und App erreicht weiterhin die Schwelle in `coverage-min`; `make coverage`, `make mutants` und `make report` laufen mit den neuen Pfaden | ☑ | **gilt** | kein Test | – |
| FEA-35/AK3 | Alle GitHub-Workflows (Abdeckung, Pakete, Mutationen, Testqualität) laufen mit den neuen Pfaden grün | ☐ | **gilt**: Im Issue nicht abgehakt; im neuen Repository mit `gh run list` prüfbar. | kein Test | – |
| FEA-35/AK4 | Die Desktop-App lässt sich bauen und starten (`scripts/build-install.sh` bzw. dessen neuer Pfad, Git-Hook und Rauchtest `ui-smoke.sh` inbegriffen); die Oberflächentests laufen | ☑ | **gilt**: Das Skript heisst heute `tools/build-install.sh`. | kein Test | – |
| FEA-35/AK5 | Die Engine enthält ihre Daten und Beispiele selbst; `cargo package -p <engine>` schlägt nicht an fehlenden Dateien fehl | ☑ | **gilt** | kein Test | – |
| FEA-35/AK6 | Im Wurzelverzeichnis liegen keine `mutants.out*`-Ordner mehr, neue Mutationsläufe schreiben nach `target/` oder `local/` | ☑ | **gilt** | kein Test | – |
| FEA-35/AK7 | Hooks, `.claude/`, Makefile, CLAUDE.md, README, `docs/` und Spezifikation verweisen auf die neuen Pfade, keine toten Verweise | ☑ | **gilt** | kein Test | – |
| FEA-35/AK8 | Die Git-Historie der verschobenen Dateien bleibt erhalten (`git mv`) | ☑ | **überholt**: durch GEN-01: keine übernommene Historie im öffentlichen Repository. | kein Test | – |

### #50 FEA-36 Indirekte Amortisation nur bis zum AHV-Alter, danach direkt; 3a-Auszahlung beim AHV-Alter

[Issue #50](https://github.com/ingen63/VivaPlan/issues/50) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/src/validate.rs:1358](../crates/engine/src/validate.rs#L1358), [engine/src/validate.rs:1387](../crates/engine/src/validate.rs#L1387) (+4)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-36/AK1 | Bis zum Monat vor dem AHV-Bezug fliesst die Amortisation einer Tranche mit indirekter Amortisation (`amortMode: indirect`) in das 3a-Konto, wie bisher. | ☑ | **gilt** | [engine/tests/pillar3a.rs:215](../crates/engine/tests/pillar3a.rs#L215) | – |
| FEA-36/AK2 | Ab dem Monat des AHV-Bezugs wird dieselbe Amortisation als direkte Amortisation behandelt: Sie verringert die Hypothek und wird nicht mehr in die Säule 3a eingezahlt. Der Betrag je Quartal bleibt gleich. | ☑ | **gilt** | [engine/tests/pillar3a.rs:215](../crates/engine/tests/pillar3a.rs#L215) | – |
| FEA-36/AK3 | Beispiel mit erfundenen Zahlen: Hypothek 400'000, Tranche mit 750 pro Quartal indirekt, AHV-Bezug im Monat M. Vor M steigt der 3a-Saldo um 750 pro Quartal und die Hypothek bleibt 400'000. Ab M sinkt die Hypothek um 750 pro Quartal, und der 3a-Saldo bleibt unverändert (die Auszahlung des Rests in M ist Sache von FEA-37). | ☑ | **gilt** | [engine/tests/pillar3a.rs:215](../crates/engine/tests/pillar3a.rs#L215) | – |
| FEA-36/AK4 | Eine Tranche oder Periode, die erst ab dem AHV-Alter beginnt (auch eine Erneuerung), kann nicht indirekt amortisieren; sie wird direkt behandelt. Die Eingabemaske schaltet die Auswahl auf «direkt» und nennt den Grund. | ☑ | **gilt** | [engine/tests/pillar3a.rs:243](../crates/engine/tests/pillar3a.rs#L243), [ui.spec.js:2411](../e2e/ui.spec.js#L2411) | – |
| FEA-36/AK5 | Gibt es keine Person mit AHV-Bezug in der Planung, bleibt das Verhalten unverändert. Das ist in der Hilfe benannt. | ☑ | **gilt** | kein Test | – |
| FEA-36/AK6 | Regressionstest für den Rechenkern; Golden-Dateien ändern nur mit Begründung im Issue. Hilfetexte (`app/ui/hilfe/`), `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt**: Pfad heute `crates/app/ui/hilfe/` (FEA-35). | kein Test | – |

### #51 FEA-38 Meilensteine AHV-Bezug und Bezug Pensionskasse: immer vorhanden, nicht löschbar, Grenzen und Standard 65

[Issue #51](https://github.com/ingen63/VivaPlan/issues/51) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/src/optimize.rs:1180](../crates/engine/src/optimize.rs#L1180), [engine/src/resolve.rs:185](../crates/engine/src/resolve.rs#L185) (+7)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-38/AK1 | Jede Person hat die Meilensteine «AHV-Bezug» und «Bezug Pensionskasse». Beide lassen sich in der Oberfläche nicht löschen (keine Löschen-Schaltfläche, Hinweis warum). | ☑ | **gilt** | [engine/src/upgrade.rs:496](../crates/engine/src/upgrade.rs#L496), [engine/tests/milestones_limits.rs:108](../crates/engine/tests/milestones_limits.rs#L108) (+2) | – |
| FEA-38/AK2 | Beide lassen sich ändern (Alter oder Datum, wie die übrigen Meilensteine). Das Alter muss innerhalb der Grenzen liegen: AHV-Bezug 63 bis 70 Jahre, Bezug Pensionskasse 58 bis 65 Jahre. Die Grenzen selbst sind gültig (63 und 70, 58 und 65), ein Wert ausserhalb wird mit klarer Meldung abgelehnt (Oberfläche und Validierung im Rechenkern). | ☑ | **gilt**: Die Grenzen sind seit FEA-24 einstellbar; 63 bis 70 und 58 bis 65 sind die Standardwerte. | [engine/tests/milestones_limits.rs:59](../crates/engine/tests/milestones_limits.rs#L59), [ui.spec.js:631](../e2e/ui.spec.js#L631) (+1) | – |
| FEA-38/AK3 | Neue Planung: Beide Meilensteine stehen auf 65 Jahren. | ☑ | **gilt** | kein Test | – |
| FEA-38/AK4 | Beispiel: Geburt 1980-06. AHV-Bezug mit 65 liegt im Monat 2045-06, mit 63 im Monat 2043-06, mit 70 im Monat 2050-06. Bezug Pensionskasse mit 58 liegt im Monat 2038-06. | ☑ | **gilt** | [engine/tests/milestones_limits.rs:95](../crates/engine/tests/milestones_limits.rs#L95) | – |
| FEA-38/AK5 | Der Meilenstein «Pensionierung» heisst überall «Bezug Pensionskasse» (Oberfläche, Hilfe, Berichte). Die Id `retire` bleibt, damit Planungsdateien gültig bleiben. | ☑ | **gilt** | [engine/src/upgrade.rs:439](../crates/engine/src/upgrade.rs#L439), [ui.spec.js:2346](../e2e/ui.spec.js#L2346) | – |
| FEA-38/AK6 | Ältere Planungsdateien ohne einen der beiden Meilensteine erhalten ihn beim Laden mit dem Standard 65. Liegt ein vorhandenes Alter ausserhalb der Grenzen, wird es auf die nächste Grenze gesetzt, mit Hinweis beim Laden (kein stiller Datenverlust). | ☑ | **gilt** | [engine/src/upgrade.rs:439](../crates/engine/src/upgrade.rs#L439), [engine/src/upgrade.rs:459](../crates/engine/src/upgrade.rs#L459) (+2) | – |
| FEA-38/AK7 | Bis FEA-24 umgesetzt ist, gelten die Grenzen fest. Die Stelle, die später die Einstellung liest, ist an einer Stelle gebündelt. | ☑ | **überholt**: durch FEA-24: Die Grenzen kommen aus den Einstellungen. | kein Test | – |
| FEA-38/AK8 | Prüfung (Warnung, keine Sperre): Beträgt das Erwerbseinkommen im Monat des Bezugs der Pensionskasse oder danach mehr als 0, meldet die Prüfung eine Warnung mit dem Betrag, z. B. «Das Erwerbseinkommen beträgt nach dem Bezug der Pensionskasse (06.2040) noch 12'500 pro Monat.» Bei Einkommen 0 ab dem Bezug erscheint keine Warnung. Die Warnung steht in der Prüfung des Rechenkerns (`Severity::Warning`) und in der Oberfläche. | ☑ | **gilt** | [engine/tests/milestones_limits.rs:123](../crates/engine/tests/milestones_limits.rs#L123), [ui.spec.js:2365](../e2e/ui.spec.js#L2365) | – |
| FEA-38/AK9 | Beispiel mit erfundenen Zahlen: Einkommen 12'500 ab 2035-02 ohne spätere Zeile mit 0, Bezug Pensionskasse 2040-06: Es erscheint die Warnung mit 12'500. Mit einer Zeile «ab Bezug Pensionskasse: 0» erscheint keine Warnung. Mit einer Zeile «ab 2040-06: 3'000» lautet die Warnung 3'000. | ☑ | **gilt** | [engine/tests/milestones_limits.rs:123](../crates/engine/tests/milestones_limits.rs#L123) | – |
| FEA-38/AK10 | Hilfe, `BESCHREIBUNG.md` und Spezifikation sind nachgeführt; Regressionstest im Rechenkern. | ☑ | **gilt** | kein Test | – |

### #52 FEA-37 Säule 3a als eigenes Konto mit Auszahlungsplan, Rest spätestens im AHV-Alter

[Issue #52](https://github.com/ingen63/VivaPlan/issues/52) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/src/validate.rs:1358](../crates/engine/src/validate.rs#L1358), [engine/src/validate.rs:1420](../crates/engine/src/validate.rs#L1420) (+12)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-37/AK1 | Die Oberfläche hat einen eigenen Abschnitt «Säule 3a» (nicht unter Immobilien, nicht unter Pensionskasse) mit Guthaben bei Simulationsstart und Verzinsung. | ☑ | **gilt** | [ui.spec.js:2372](../e2e/ui.spec.js#L2372) | – |
| FEA-37/AK2 | Der Auszahlungsplan ist eine Liste aus Monat/Jahr und Betrag. Zeilen lassen sich hinzufügen und löschen. | ☑ | **gilt** | [ui.spec.js:2372](../e2e/ui.spec.js#L2372) | – |
| FEA-37/AK3 | Beispiel mit erfundenen Zahlen: Guthaben 120'000, Plan 2041: 50'000 und 2042: 50'000, AHV-Bezug im Jahr 2045. Wird der AHV-Bezug auf 2046 verschoben, bleibt der Plan gültig (frühester Zeitpunkt 2041) und der Rest wird erst 2046 ausbezahlt. Ergebnis: Auszahlungen von 50'000 in 2041 und 2042 und Auszahlung des Rests (20'000 plus Zins) im Monat des AHV-Bezugs. Danach ist das Guthaben 0. | ☑ | **gilt** | [engine/tests/pillar3a.rs:62](../crates/engine/tests/pillar3a.rs#L62) | – |
| FEA-37/AK4 | Ein Plan-Eintrag früher als 5 Jahre vor dem Meilenstein «AHV-Bezug» wird mit klarer Meldung abgelehnt (Validierung im Rechenkern und in der Oberfläche). Genau 5 Jahre vorher ist gültig. | ☑ | **gilt** | [engine/src/validate.rs:1397](../crates/engine/src/validate.rs#L1397) | – |
| FEA-37/AK5 | Ist ein Betrag grösser als das Guthaben zum Zeitpunkt, wird nur das Guthaben ausbezahlt (Hinweis in den Ereignissen). | ☑ | **gilt** | [engine/src/validate.rs:1397](../crates/engine/src/validate.rs#L1397), [engine/tests/pillar3a.rs:102](../crates/engine/tests/pillar3a.rs#L102) | – |
| FEA-37/AK6 | Jede Auszahlung ist ein Kapitalbezug mit Kapitalbezugssteuer; Bezüge im selben Jahr werden mit PK-Kapitalbezügen für die Progression zusammengerechnet (wie bisher). | ☑ | **gilt** | [engine/tests/pillar3a.rs:128](../crates/engine/tests/pillar3a.rs#L128) | – |
| FEA-37/AK7 | Das 3a-Konto hat das Feld «Einzahlung pro Jahr» (Standard 0). Gültig sind Werte von 0 bis zum Maximalbetrag (7'258), beide Grenzen eingeschlossen. Ein negativer Wert oder ein Wert über dem Maximalbetrag wird mit klarer Meldung abgelehnt (Oberfläche und Validierung im Rechenkern). | ☑ | **gilt**: Der Maximalbetrag ist seit FEA-24 einstellbar (Standard 7'258). | kein Test | – |
| FEA-37/AK8 | Die Einzahlung erhöht das 3a-Guthaben und zählt zu den Ausgaben: Sie erscheint in «Ausgaben total» der Jahreswerte, erhöht die Entnahme bzw. verringert den Überschuss und wirkt in den Ausgaben der Strategien (3-Topf, Guardrails, Netto-Bedarf). Die Einzahlung verteilt sich auf zwölf gleiche Monatsbeträge. Beispiel mit erfundenen Zahlen: Einzahlung 6'000 pro Jahr, Guthaben 10'000, Zins 0: nach einem Jahr 16'000, die Ausgaben des Jahres sind um 6'000 höher (500 pro Monat im Monatsdurchschnitt). | ☑ | **gilt** | [engine/tests/pillar3a.rs:146](../crates/engine/tests/pillar3a.rs#L146), [ui.spec.js:2396](../e2e/ui.spec.js#L2396) | – |
| FEA-37/AK9 | Einzahlungen enden spätestens im Monat vor dem Meilenstein «AHV-Bezug». Danach zahlt keine Einzahlung mehr ein (Hinweis in der Eingabe). | ☑ | **gilt** | [engine/tests/pillar3a.rs:173](../crates/engine/tests/pillar3a.rs#L173) | – |
| FEA-37/AK10 | Die Einzahlungen und die indirekte Amortisation (FEA-36) zusammen dürfen im Jahr den Maximalbetrag nicht überschreiten. Die Prüfung meldet eine Überschreitung mit klarer Warnung (keine Sperre). | ☑ | **gilt** | kein Test | – |
| FEA-37/AK11 | Der Maximalbetrag ist an einer Stelle gebündelt, damit FEA-24 ihn später aus den Einstellungen liest. | ☑ | **überholt**: durch FEA-24: Der Maximalbetrag kommt aus den Einstellungen. | kein Test | – |
| FEA-37/AK12 | Die Pensionskasse enthält kein 3a-Guthaben mehr. In den Auswertungen bleibt «Säule 3a» getrennt von «Pensionskasse» (heute schon im Diagramm Vorsorge). | ☑ | **gilt** | kein Test | – |
| FEA-37/AK13 | Alte Planungsdateien (`realestate.properties[].mortgage.pillar3a` mit Guthaben, Zins, `payout`) werden beim Laden in das neue 3a-Konto übernommen: Guthaben und Zins bleiben erhalten, die frühere Auszahlung wird ein einzelner Plan-Eintrag über das ganze Guthaben, auf den frühesten erlaubten Zeitpunkt begrenzt, mit Hinweis beim Laden. Mehrere Immobilien mit je einer 3a werden zu einem Konto zusammengefasst (Hinweis). | ☑ | **gilt** | [engine/src/upgrade.rs:533](../crates/engine/src/upgrade.rs#L533), [engine/src/upgrade.rs:592](../crates/engine/src/upgrade.rs#L592) (+2) | – |
| FEA-37/AK14 | Regressionstest im Rechenkern; Golden-Dateien ändern nur mit Begründung im Issue. Hilfe, `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test | – |

### #53 FEA-39 Lebenshaltungskosten inklusive Wohnkosten, Simulation mit den echten Wohnkosten

[Issue #53](https://github.com/ingen63/VivaPlan/issues/53) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/src/scenarios.rs:257](../crates/engine/src/scenarios.rs#L257), [engine/tests/cash_housing.rs:1](../crates/engine/tests/cash_housing.rs#L1) (+5)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-39/AK1 | Beschriftung und Hilfe der Lebenshaltungskosten sagen «inklusive Miete bzw. Wohnkosten (Zins, Amortisation, Nebenkosten)». | ☑ | **gilt** | [ui.spec.js:2451](../e2e/ui.spec.js#L2451) | – |
| FEA-39/AK2 | Der Wohnanteil ist die Summe der simulierten Wohnkosten im Startmonat (Miete bzw. Hypothekarzins, Amortisation, Nebenkosten), mit der Teuerung fortgeschrieben. Er wird von jeder Zeile einer zeitabhängigen Lebenshaltung gleich abgezogen. | ☑ | **gilt** | [engine/src/housing.rs:99](../crates/engine/src/housing.rs#L99), [ui.spec.js:2451](../e2e/ui.spec.js#L2451) | – |
| FEA-39/AK3 | Beispiel mit erfundenen Zahlen: Eingabe 8'000, Wohnkosten im Startmonat 2'000 (Zins 1'000, Amortisation 500, Nebenkosten 500), Teuerung 0. Die Simulation rechnet mit 6'000 Lebenshaltung plus den echten Wohnkosten; solange sich nichts ändert, sind die «Ausgaben total» 8'000. Steigt der Zins bei einer Erneuerung um 300, sind sie 8'300; bei einem Wechsel in eine Miete von 2'500 sind sie 8'500. | ☑ | **gilt** | [engine/src/housing.rs:99](../crates/engine/src/housing.rs#L99), [engine/tests/cash_housing.rs:70](../crates/engine/tests/cash_housing.rs#L70) | – |
| FEA-39/AK4 | Ist der Wohnanteil grösser als eine Zeile der Lebenshaltung, meldet die Prüfung einen Fehler mit beiden Beträgen. | ☑ | **gilt** | [engine/tests/cash_housing.rs:85](../crates/engine/tests/cash_housing.rs#L85), [ui.spec.js:2459](../e2e/ui.spec.js#L2459) | – |
| FEA-39/AK5 | Ohne Wohnkosten im Startmonat (keine Immobilie, keine Miete) ändert sich nichts. | ☑ | **gilt** | [engine/src/housing.rs:122](../crates/engine/src/housing.rs#L122), [engine/tests/cash_housing.rs:85](../crates/engine/tests/cash_housing.rs#L85) | – |
| FEA-39/AK6 | Die Einzahlung in die Säule 3a (FEA-37) ist nicht in der Eingabe enthalten und kommt wie bisher separat zu den Ausgaben. | ☑ | **gilt** | [engine/tests/cash_housing.rs:108](../crates/engine/tests/cash_housing.rs#L108) | – |
| FEA-39/AK7 | Ältere Planungen (Lebenshaltung bisher ohne Wohnkosten) werden beim Öffnen umgerechnet (Wohnanteil zu jeder Zeile addiert), mit Hinweis, damit sich das Ergebnis nicht ändert. | ☑ | **gilt** | [engine/tests/cash_housing.rs:118](../crates/engine/tests/cash_housing.rs#L118), [ui.spec.js:2480](../e2e/ui.spec.js#L2480) | – |
| FEA-39/AK8 | Jahreswerte («Ausgaben total», «Abweichung vom Plan»), Netto-Bedarf der 3-Topf-Strategie, Ausgabenregeln (Guardrails & Co.), Grenzen der Lebenshaltung und Ausgaben-Faktor der Variation verwenden die neue Aufteilung. Regressionstest im Rechenkern; Golden-Dateien ändern nur mit Begründung im Issue. Hilfe, `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt** | [engine/tests/cash_housing.rs:158](../crates/engine/tests/cash_housing.rs#L158) | – |

### #54 FEA-40 Cash getrennt vom Portfolio, Strategien erst ab Pensionierung, «Liquidität» heisst «Cash»

[Issue #54](https://github.com/ingen63/VivaPlan/issues/54) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/cash_housing.rs:1](../crates/engine/tests/cash_housing.rs#L1), [engine/tests/cash_housing.rs:192](../crates/engine/tests/cash_housing.rs#L192) (+4)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-40/AK1 | Die Aktienquote gilt nur für Aktien und Obligationen; Cash ist nicht Teil davon. Beschriftung und Hilfe sagen das. | ☑ | **gilt** | [ui.spec.js:2470](../e2e/ui.spec.js#L2470) | – |
| FEA-40/AK2 | Beispiel mit erfundenen Zahlen: Aktien 80'000, Obligationen 20'000, Cash 50'000, Aktienquote 80 %, keine Renditen, Einnahmen gleich Ausgaben: Bis zur Pensionierung bleiben Cash 50'000, Aktien 80'000, Obligationen 20'000 (kein Umschichten von Cash ins Portfolio). | ☑ | **gilt** | [engine/tests/cash_housing.rs:168](../crates/engine/tests/cash_housing.rs#L168) | – |
| FEA-40/AK3 | Beispiel Überschuss: wie oben, aber 2'000 Einkommen mehr als Ausgaben pro Monat. Cash bleibt 50'000, die 2'000 gehen ins Portfolio (1'600 Aktien, 400 Obligationen). | ☑ | **gilt** | [engine/tests/cash_housing.rs:180](../crates/engine/tests/cash_housing.rs#L180) | – |
| FEA-40/AK4 | Vor der Pensionierung gilt keine Entnahmestrategie, auch nicht «Standard»; die Liquiditäts- bzw. Cash-Reserve in Monaten wirkt erst ab der Pensionierung. | ☑ | **gilt** | [engine/tests/cash_housing.rs:214](../crates/engine/tests/cash_housing.rs#L214), [ui.spec.js:2470](../e2e/ui.spec.js#L2470) | – |
| FEA-40/AK5 | Ab der Pensionierung: «Standard» mit der konfigurierten Cash-Reserve (Monate), alle anderen Strategien wie bisher (3 Monate bzw. 3-Topf mit ihren Töpfen). | ☑ | **gilt** | [engine/tests/cash_housing.rs:214](../crates/engine/tests/cash_housing.rs#L214) | – |
| FEA-40/AK6 | «Liquidität…» heisst überall «Cash…» (Oberfläche, Hilfe, Jahreswerte, Grafiken, Berichte, Prüfmeldungen, Dokumentation). Die Feldnamen in der Planungsdatei bleiben (`wealth.cash`, `reserveMonths`), damit bestehende Dateien gültig bleiben. | ☑ | **gilt** | kein Test | – |
| FEA-40/AK7 | Beispiel Nachfüllen und Teuerung: Cash 50'000, Teuerung 2 % pro Jahr, Ausgaben 1'000 über den Einnahmen pro Monat: Cash wird jeden Monat aus dem Portfolio auf den mit der Teuerung fortgeschriebenen Betrag gebracht (nach einem Jahr rund 51'000). | ☑ | **gilt** | kein Test | – |
| FEA-40/AK8 | Ältere Planungen rechnen nach dem neuen Verhalten; beim Öffnen weist ein Hinweis darauf hin, dass Cash vor der Pensionierung neu fest gehalten wird. | ☑ | **gilt** | [engine/tests/cash_housing.rs:143](../crates/engine/tests/cash_housing.rs#L143), [ui.spec.js:2480](../e2e/ui.spec.js#L2480) | – |
| FEA-40/AK9 | Regressionstest im Rechenkern; Golden-Dateien ändern nur mit Begründung im Issue. Hilfe, `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test | – |

### #55 FEA-41 Säule 3a: Auszahlungsplan mit Datum oder Alter, Guthaben anzeigen, Restbetrag vorabfüllen

[Issue #55](https://github.com/ingen63/VivaPlan/issues/55) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/pillar3a_plan.rs:1](../crates/engine/tests/pillar3a_plan.rs#L1), [engine/tests/pillar3a_plan.rs:119](../crates/engine/tests/pillar3a_plan.rs#L119) (+3)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-41/AK1 | Jede Zeile des Auszahlungsplans hat die Auswahl «Datum» oder «Alter». Beim Wechsel bleibt der Zeitpunkt gleich (wie bei Meilensteinen, ISS-10). Daneben steht der berechnete Monat und das Alter. | ☑ | **gilt** | [ui.spec.js:2516](../e2e/ui.spec.js#L2516) | – |
| FEA-41/AK2 | Beispiel mit erfundenen Zahlen: Geburt 1980-06, Zeile «Alter 61.5» ergibt 12.2041; als «Datum» umgestellt steht 12.2041 im Feld. | ☑ | **gilt** | [engine/tests/pillar3a_plan.rs:44](../crates/engine/tests/pillar3a_plan.rs#L44), [ui.spec.js:2516](../e2e/ui.spec.js#L2516) | – |
| FEA-41/AK3 | Die Regel «frühestens 5 Jahre vor dem AHV-Bezug» (FEA-37) gilt auch für eine Zeile nach Alter; die Meldung steht am Feld. | ☑ | **gilt** | [engine/tests/pillar3a_plan.rs:63](../crates/engine/tests/pillar3a_plan.rs#L63), [ui.spec.js:2534](../e2e/ui.spec.js#L2534) | – |
| FEA-41/AK4 | Jede Zeile zeigt «Guthaben vor Auszahlung» und «Rest danach» aus der Vorschau; unter dem Plan steht das Guthaben, das im AHV-Bezug ausbezahlt wird. Ohne Vorschau (Fehler in der Planung) steht «–». | ☑ | **gilt** | [engine/tests/pillar3a_plan.rs:94](../crates/engine/tests/pillar3a_plan.rs#L94), [ui.spec.js:2534](../e2e/ui.spec.js#L2534) (+1) | – |
| FEA-41/AK5 | Beispiel: Guthaben 120'000, Zins 0, keine Einzahlung, Zeile 2041 mit 50'000: vor 120'000, Rest 70'000; Rest im AHV-Bezug 70'000. | ☑ | **gilt** | [engine/tests/pillar3a_plan.rs:94](../crates/engine/tests/pillar3a_plan.rs#L94), [ui.spec.js:2544](../e2e/ui.spec.js#L2544) | – |
| FEA-41/AK6 | «+ Auszahlung» legt die neue Zeile ein Jahr nach der letzten an (ohne Zeile: frühester erlaubter Monat, 5 Jahre vor dem AHV-Bezug, nicht vor dem Start, höchstens im AHV-Bezug). Der Betrag ist das Guthaben laut Vorschau zu diesem Zeitpunkt nach den früheren Auszahlungen, abgerundet auf 1'000. Er bleibt danach frei änderbar. | ☑ | **gilt** | [ui.spec.js:2544](../e2e/ui.spec.js#L2544) | – |
| FEA-41/AK7 | Beispiel: wie oben; «+ Auszahlung» ergibt eine Zeile 2042 mit 70'000. | ☑ | **gilt** | [ui.spec.js:2544](../e2e/ui.spec.js#L2544) | – |
| FEA-41/AK8 | Ältere Planungen (Zeile nur mit Datum) bleiben gültig und erscheinen als «Datum». | ☑ | **gilt** | [engine/tests/pillar3a_plan.rs:130](../crates/engine/tests/pillar3a_plan.rs#L130), [ui.spec.js:2516](../e2e/ui.spec.js#L2516) | – |
| FEA-41/AK9 | Regressionstest im Rechenkern für die Zeile nach Alter; Oberflächentest für Umschalten, Anzeige und Vorabfüllen. Hilfe, `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test | – |

### #56 FEA-42 Ganzes Projekt mit rustfmt formatieren, Formatierung prüfen, Agentenanleitung anpassen

[Issue #56](https://github.com/ingen63/VivaPlan/issues/56) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-42/AK1 | Im Wurzelverzeichnis liegt eine `rustfmt.toml` mit den gewählten Einstellungen (z. B. Zeilenlänge); sie gilt für alle Pakete. | ☑ | **gilt** | kein Test | – |
| FEA-42/AK2 | `cargo fmt --all --check` meldet im ganzen Workspace keine Abweichung. | ☑ | **gilt** | kein Test | – |
| FEA-42/AK3 | Die Umformatierung ist ein eigener Commit ohne inhaltliche Änderung; alle Rust-Tests (`make test`) laufen danach unverändert grün, die Golden-Dateien bleiben gleich. | ☑ | **gilt** | kein Test | – |
| FEA-42/AK4 | Der Commit der Umformatierung steht in `.git-blame-ignore-revs`, damit `git blame` die fachlichen Änderungen zeigt. | ☑ | **überholt**: durch GEN-01: Das öffentliche Repository hat die alte Historie nicht; `.git-blame-ignore-revs` ist entfernt. | kein Test | – |
| FEA-42/AK5 | Ein GitHub-Workflow (oder ein bestehender Job) prüft bei jedem Push `cargo fmt --all --check` und schlägt bei Abweichungen fehl. | ☑ | **gilt** | kein Test | – |
| FEA-42/AK6 | `CLAUDE.md` beschreibt die neue Regel: Code wird mit `cargo fmt` formatiert, vor dem Commit läuft `cargo fmt --all --check`. | ☑ | **gilt** | kein Test | – |
| FEA-42/AK7 | Die Sperre von `cargo fmt` in `.claude/settings.json` ist aufgehoben (bzw. auf das Nötige beschränkt). | ☑ | **gilt** | kein Test | – |
| FEA-42/AK8 | Die Agentenanleitung `.claude/agents/rust-test-agent.md` und die Aufträge in `.claude/hooks/after-commit.sh` und `scripts/git-hooks/pre-push` sagen nicht mehr «Kein cargo fmt (das Projekt ist nicht rustfmt-formatiert)», sondern verlangen formatierten Code. | ☑ | **gilt**: Der Hook liegt heute unter `tools/git-hooks/pre-push`. | kein Test | – |
| FEA-42/AK9 | Keine anderen Dokumente (`TESTS.md`, `QA.md`, `README.md`) beschreiben die alte Regel noch. | ☑ | **gilt** | kein Test | – |

### #57 ISS-13 Fehlermeldung am Feld erst beim Verlassen, nicht während der Eingabe

[Issue #57](https://github.com/ingen63/VivaPlan/issues/57) · geschlossen · Umsetzung: `88cf6fe`, `c6d2073` (altes Repository) · Tests zum Issue ohne Kriterium: [ui.spec.js:224](../e2e/ui.spec.js#L224), [ui.spec.js:457](../e2e/ui.spec.js#L457) (+7)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| ISS-13/AK1 | Solange ein Feld den Fokus hat, erscheint an diesem Feld keine neue Fehlermeldung (weder vom Rechenkern noch zu ungültiger Eingabe wie falschem Datumsformat). Beispiel: Alter 60 → 6 → 65 ohne Meldung. | ☑ | **gilt** | [ui.spec.js:2572](../e2e/ui.spec.js#L2572), [ui.spec.js:2613](../e2e/ui.spec.js#L2613) | – |
| ISS-13/AK2 | Beim Verlassen des Feldes (Tab, Klick daneben, Enter) erscheint die Meldung, wenn der Wert ungültig ist. Beispiel: Alter 6 eingeben und das Feld verlassen: Meldung «ausserhalb der Grenzen». | ☑ | **gilt** | [ui.spec.js:2572](../e2e/ui.spec.js#L2572), [ui.spec.js:2613](../e2e/ui.spec.js#L2613) | – |
| ISS-13/AK3 | Ist an einem Feld bereits eine Meldung sichtbar und wird der Wert gültig, verschwindet sie sofort, schon während der Eingabe. | ☑ | **gilt**: Neu gefasst per Kommentar: Eine sichtbare Meldung bleibt während der Eingabe stehen und verschwindet erst beim Verlassen. | [ui.spec.js:2572](../e2e/ui.spec.js#L2572) | – |
| ISS-13/AK4 | Die Vorschau, die Meldungen an anderen Feldern und die Prüfliste rechnen weiter bei jeder Eingabe; nur die Anzeige am aktiven Feld wartet. | ☑ | **gilt**: Neu gefasst per Kommentar: Solange ein Feld den Fokus hat, wartet alles (Vorschau, Meldungen, Prüfliste); Auswahllisten und Kästchen wirken sofort. | [ui.spec.js:2572](../e2e/ui.spec.js#L2572) | – |
| ISS-13/AK5 | Oberflächentest für die drei Beispiele; Hilfe nachgeführt. | ☑ | **gilt** | kein Test | – |

### #58 FEA-43 Erwerbseinkommen optional mit der Teuerung

[Issue #58](https://github.com/ingen63/VivaPlan/issues/58) · geschlossen · Umsetzung: `ca1a939` (altes Repository) · Tests zum Issue ohne Kriterium: [engine/tests/income_indexed.rs:1](../crates/engine/tests/income_indexed.rs#L1), [engine/tests/income_indexed.rs:113](../crates/engine/tests/income_indexed.rs#L113) (+4)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-43/AK1 | Beim Erwerbseinkommen jeder Person gibt es eine Auswahl «steigt mit der Teuerung» (ja/nein). Ohne Angabe gilt «nein»; bestehende Planungen rechnen unverändert. | ☑ | **gilt** | [engine/tests/income_indexed.rs:52](../crates/engine/tests/income_indexed.rs#L52), [engine/tests/income_indexed.rs:189](../crates/engine/tests/income_indexed.rs#L189) (+1) | – |
| FEA-43/AK2 | Mit «ja» wird jeder Wert, auch jede Zeile eines zeitabhängigen Einkommens, als Betrag in Preisen des Simulationsstarts verstanden und mit dem Preisindex der Simulation fortgeschrieben (wie die Lebenshaltung). | ☑ | **gilt** | [engine/tests/income_indexed.rs:72](../crates/engine/tests/income_indexed.rs#L72) | – |
| FEA-43/AK3 | Beispiel mit erfundenen Zahlen: Einkommen 8'000 pro Monat, Teuerung 2 % pro Jahr, «ja»: nach 12 Monaten rund 8'160, nach 5 Jahren rund 8'830. Mit «nein» bleiben es 8'000. | ☑ | **gilt** | [engine/tests/income_indexed.rs:52](../crates/engine/tests/income_indexed.rs#L52) | – |
| FEA-43/AK4 | Die Auswahl wirkt in allen Renditequellen (fix, historisch, Monte Carlo) mit dem jeweiligen Preisindex des Pfades. | ☑ | **gilt** | [engine/tests/income_indexed.rs:87](../crates/engine/tests/income_indexed.rs#L87) | – |
| FEA-43/AK5 | Jahreswerte, Grafiken, Einkommenssteuer und die Warnung «Erwerbseinkommen nach dem Bezug der Pensionskasse» (FEA-38) verwenden den angepassten Betrag. | ☑ | **gilt** | [engine/tests/income_indexed.rs:139](../crates/engine/tests/income_indexed.rs#L139) | – |
| FEA-43/AK6 | Regressionstest im Rechenkern; Golden-Dateien ändern nicht (Vorgabe «nein»). Oberflächentest für die Auswahl. Hilfe, `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test | – |

### #59 FEA-44 Einstellungen auf macOS im Programmmenü

[Issue #59](https://github.com/ingen63/VivaPlan/issues/59) · geschlossen · Umsetzung: `77f560b` (altes Repository)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-44/AK1 | macOS: Das Menü «RetirementPlanner» zeigt «Über RetirementPlanner», Trennlinie, «Einstellungen…» (⌘,), Trennlinie, dann die Systemeinträge und «Beenden». | ☑ | **gilt**: Das Programmmenü heisst seit FEA-34 «VivaPlan». | [app/src/main.rs:1370](../crates/app/src/main.rs#L1370) | – |
| FEA-44/AK2 | macOS: Das Menü «Datei» enthält keinen Eintrag «Einstellungen…». | ☑ | **gilt** | [app/src/main.rs:1370](../crates/app/src/main.rs#L1370) | – |
| FEA-44/AK3 | Windows und Linux: Das Menü «Datei» endet mit «Einstellungen…» (Ctrl+,), Trennlinie, «Beenden». | ☑ | **gilt** | [app/src/main.rs:1370](../crates/app/src/main.rs#L1370) | – |
| FEA-44/AK4 | Auf jedem System gibt es den Eintrag genau einmal; er öffnet den Dialog «Einstellungen» wie bisher. | ☑ | **gilt** | [app/src/main.rs:1370](../crates/app/src/main.rs#L1370) | – |
| FEA-44/AK5 | Hilfe, BESCHREIBUNG.md und Spezifikation nennen den Ort je System. | ☑ | **gilt** | kein Test | – |

### #60 ISS-14 Zeitreihen-CSV: Absturz bei Jahreszahl ausserhalb des Bereichs (Fuzz parse_csv)

[Issue #60](https://github.com/ingen63/VivaPlan/issues/60) · geschlossen · Umsetzung: `b900a7e` (altes Repository)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| ISS-14/AK1 | Eine jährliche Zeitreihe mit Jahr ausserhalb 1000 bis 9999 (z. B. `99999999999`, `-2147483648`, `2147483647`) ergibt eine Fehlermeldung mit Zeilennummer, keinen Absturz. | ☑ | **gilt** | [engine/tests/data.rs:99](../crates/engine/tests/data.rs#L99) | – |
| ISS-14/AK2 | Die Prüfung auf fortlaufende Daten stürzt auch bei extremen Werten nicht ab. | ☑ | **gilt** | [engine/tests/data.rs:99](../crates/engine/tests/data.rs#L99) | – |
| ISS-14/AK3 | Bestehende Zeitreihen (Monats- und Jahresdaten) lesen sich unverändert; alle Rust-Tests und Golden-Dateien bleiben gleich. | ☑ | **gilt** | kein Test | – |
| ISS-14/AK4 | Regressionstest mit dem Absturzfall des Fuzzers; der Job `fuzz` läuft im nächsten nächtlichen oder von Hand gestarteten Lauf ohne Absturz in `parse_csv`. | ☑ | **gilt** | kein Test | – |

### #61 FEA-45 Haftungsausschluss: Bestätigung beim ersten Start, Fusszeile in den Ergebnissen, vollständiger Text im Info-Bereich

[Issue #61](https://github.com/ingen63/VivaPlan/issues/61) · geschlossen · Umsetzung: `d713431` (altes Repository) · Tests zum Issue ohne Kriterium: [cli/tests/cli.rs:55](../crates/cli/tests/cli.rs#L55), [ui.spec.js:1977](../e2e/ui.spec.js#L1977) (+2)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-45/AK1 | Beim ersten Start erscheint die Kurzfassung als Dialog. Weiter geht es erst, wenn das Kästchen «Ich habe den Hinweis gelesen und verstanden» angekreuzt und bestätigt ist; ohne Bestätigung ist die App nicht bedienbar. | ☑ | **gilt** | [ui.spec.js:2000](../e2e/ui.spec.js#L2000) | – |
| FEA-45/AK2 | Die Bestätigung wird lokal gespeichert (wie die Einstellungen, nicht in der Planungsdatei); bei späteren Starts erscheint der Dialog nicht mehr. Ändert sich der Text (Versionsnummer des Hinweises), wird erneut gefragt. | ☑ | **gilt** | [ui.spec.js:2000](../e2e/ui.spec.js#L2000), [ui.spec.js:2029](../e2e/ui.spec.js#L2029) (+1) | – |
| FEA-45/AK3 | Alle Ergebnisseiten (Übersicht, Grafiken, Ereignisse, Jahreswerte, Szenarien, Optimierung) zeigen unten die Fusszeile «Modellrechnung ohne Gewähr. Keine Finanz- oder Steuerberatung.»; auch in Ausdruck bzw. PDF-Export, falls vorhanden. | ☑ | **gilt** | [ui.spec.js:2895](../e2e/ui.spec.js#L2895), [ui.spec.js:2908](../e2e/ui.spec.js#L2908) | – |
| FEA-45/AK4 | Der vollständige Haftungsausschluss ist jederzeit erreichbar: im Dialog «Über …» (Menü und Hilfe) und als Seite in der Hilfe, mit Stand-Datum. | ☑ | **gilt** | [ui.spec.js:2921](../e2e/ui.spec.js#L2921) | – |
| FEA-45/AK5 | Oberflächentests für AK1 bis AK4 (erster Start, gespeicherte Bestätigung, neue Version fragt erneut, Fusszeile, Info-Bereich). Hilfe, `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt**: Kommentar vom 14.11.2026: «AK-5. Nicht erfüllt.» Das Issue ist trotzdem geschlossen und abgehakt; nachprüfen. | kein Test | – |

### #67 FEA-51 Eigener Welt-Aktienindex aus den JST-Daten nach BIP gewichtet, Vergleich mit DMS

[Issue #67](https://github.com/ingen63/VivaPlan/issues/67) · geschlossen · Umsetzung: `2ccfc9e` (altes Repository) · Tests zum Issue ohne Kriterium: [engine/tests/jst.rs:1](../crates/engine/tests/jst.rs#L1), [engine/tests/jst.rs:63](../crates/engine/tests/jst.rs#L63) (+1)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-51/AK1 | Der Rechenweg ist beschrieben und aus den Rohdaten nachrechenbar: BIP in USD über den Wechselkurs, Gewicht = BIP-Anteil des Vorjahres, Umrechnung der Länderrenditen in CHF, Gewichte jährlich angepasst. | ☑ | **gilt** | [engine/tests/jst.rs:91](../crates/engine/tests/jst.rs#L91) | – |
| FEA-51/AK2 | Fehlende Länder oder Jahre (zum Beispiel Deutschland 1945 ohne Wechselkurs) sind benannt; die Gewichte der übrigen Länder werden neu auf 100 % gerechnet. | ☑ | **gilt** | [engine/tests/jst.rs:91](../crates/engine/tests/jst.rs#L91) | – |
| FEA-51/AK3 | Ein Test rechnet drei Jahre von Hand nach (Beispielwerte im Test). | ☑ | **gilt** | [engine/tests/jst.rs:61](../crates/engine/tests/jst.rs#L61) | – |
| FEA-51/AK4 | Vergleich mit den DMS-Kennzahlen (Rendite, Schwankung, schlechtestes Jahr) mit einem Absatz Bewertung. Die DMS-Zahlen dienen nur dem Vergleich und werden nicht ausgeliefert. | ☑ | **gilt** | [engine/tests/jst.rs:138](../crates/engine/tests/jst.rs#L138) | – |
| FEA-51/AK5 | Der Entscheid über Japan und Australien ist mit Begründung dokumentiert. | ☑ | **gilt** | kein Test | – |
| FEA-51/AK6 | Ist die Abweichung zu DMS unerwartet gross, ist die Ursache beschrieben, bevor FEA-52 beginnt. | ☑ | **gilt** | kein Test | – |

### #68 FEA-52 Zeitreihen «Welt (JST)» und Schweizer Staatsanleihen aus SNB und JST

[Issue #68](https://github.com/ingen63/VivaPlan/issues/68) · geschlossen · Umsetzung: `2ccfc9e` (altes Repository) · Tests zum Issue ohne Kriterium: [engine/tests/jst.rs:1](../crates/engine/tests/jst.rs#L1), [engine/tests/jst.rs:63](../crates/engine/tests/jst.rs#L63)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-52/AK1 | Die Reihe «Welt (JST)» beginnt 1900 und endet mit dem letzten JST-Jahr; Teuerung aus JST bzw. Landesindex (BFS). | ☑ | **gilt** | [engine/tests/jst.rs:200](../crates/engine/tests/jst.rs#L200) | – |
| FEA-52/AK2 | Obligationen: jährlich aus JST vor 1988, monatlich aus der SNB ab 1988; der Übergang ist im Test ohne Sprung geprüft. | ☑ | **gilt** | [engine/tests/jst.rs:170](../crates/engine/tests/jst.rs#L170) | – |
| FEA-52/AK3 | `data/QUELLEN.md`, Dateikopf und Hilfe nennen die Quelle, die Zitate (Jordà, Schularick, Taylor 2017; Jordà et al. 2019), das Abrufdatum und die Lizenz CC BY-NC-SA 4.0. | ☑ | **gilt**: Die Datei liegt heute unter `crates/engine/data/QUELLEN.md`. | [engine/tests/jst.rs:200](../crates/engine/tests/jst.rs#L200) | – |
| FEA-52/AK4 | Die Reihen sind als Datei mit derselben Lizenz gekennzeichnet; in `QUELLEN.md` steht, dass die App kostenlos bleiben muss (nicht kommerziell). | ☑ | **gilt**: Wie AK3. | [engine/tests/jst.rs:200](../crates/engine/tests/jst.rs#L200) | – |
| FEA-52/AK5 | Ein Golden-Test hält die Werte fest. | ☑ | **gilt** | [engine/tests/jst.rs:37](../crates/engine/tests/jst.rs#L37) | – |

### #69 FEA-53 Anlagestrategie statt Renditewahl, Gewichtung von CH, EU und USA, Reiter «Renditen & Daten» entfällt

[Issue #69](https://github.com/ingen63/VivaPlan/issues/69) · geschlossen · Umsetzung: `c2b4e38`, `93adc88` (altes Repository) · Tests zum Issue ohne Kriterium: [cli/tests/cli.rs:111](../crates/cli/tests/cli.rs#L111), [engine/tests/strategy.rs:1](../crates/engine/tests/strategy.rs#L1) (+2)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-53/AK1 | Der Reiter «Renditen & Daten» ist weg; die Strategie wird bei den Anlagen gewählt (Aktien und Obligationen getrennt). | ☑ | **gilt** | [ui.spec.js:167](../e2e/ui.spec.js#L167) | – |
| FEA-53/AK2 | «Welt individuell»: Die Gewichte ergeben immer 100 %; ungültige Eingaben werden angezeigt. Beispiel: EU 40 % auf 20 % halbiert jeden EU-Länderanteil. | ☑ | **gilt** | [engine/tests/strategy.rs:60](../crates/engine/tests/strategy.rs#L60), [engine/tests/strategy.rs:82](../crates/engine/tests/strategy.rs#L82) (+1) | – |
| FEA-53/AK3 | 100 % USA in «Welt individuell» ergibt dieselbe Reihe wie die USA-Reihe der Welt (JST) (Test). | ☑ | **gilt** | [engine/tests/strategy.rs:46](../crates/engine/tests/strategy.rs#L46) | – |
| FEA-53/AK4 | Feste Renditen funktionieren wie bisher; der Import eigener CSV-Dateien ist entfernt, die Hilfe nennt das. | ☑ | **gilt**: Der Import eigener Zeitreihen ist entfernt; das betrifft auch FEA-73 (#91). | [engine/tests/strategy.rs:242](../crates/engine/tests/strategy.rs#L242), [engine/tests/strategy.rs:407](../crates/engine/tests/strategy.rs#L407) (+1) | – |
| FEA-53/AK5 | Alte Planungen werden beim Öffnen auf die neue Strategie abgebildet; die Abbildung ist beschrieben und getestet. | ☑ | **gilt** | [engine/tests/strategy.rs:179](../crates/engine/tests/strategy.rs#L179), [engine/tests/strategy.rs:228](../crates/engine/tests/strategy.rs#L228) (+2) | – |
| FEA-53/AK6 | Vor der Umsetzung ist die Frage «CH Pictet bleibt oder nicht» geklärt. | ☑ | **überholt**: durch FEA-72: «Schweiz (Pictet)» ist entfernt. | kein Test | – |
| FEA-53/AK7 | Hilfe, `BESCHREIBUNG.md`, `README.md` und Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test | – |
| FEA-53/AK8 (Kommentar) | Die Kurzinfo zur Strategie nennt den gemeinsamen Zeitraum, auf den sich die mittleren Renditen beziehen. Wird ein Index gekürzt, weil der andere kürzer ist, erscheint ein Hinweis mit dem Index, seinem eigenen Zeitraum und dem gekürzten Zeitraum, und dass die Simulation (Monte Carlo und historische Verläufe) nur diesen Zeitraum nutzt. Ohne Kürzung kein Hinweis. Test in Rechenkern und Oberfläche. | ☑ | **gilt**: Nachtrag im Kommentar vom 14.11.2026. Das Beispiel dort nutzt Pictet (seit FEA-72 entfernt); mit einer anderen Kombination prüfen. | [engine/tests/strategy.rs:280](../crates/engine/tests/strategy.rs#L280), [engine/tests/strategy.rs:320](../crates/engine/tests/strategy.rs#L320) (+1) | – |

### #70 FEA-54 Willkommensdialog beim ersten Start

[Issue #70](https://github.com/ingen63/VivaPlan/issues/70) · geschlossen · Umsetzung: `7ba159d` (altes Repository) · Tests zum Issue ohne Kriterium: [ui.spec.js:2665](../e2e/ui.spec.js#L2665), [ui.spec.js:2671](../e2e/ui.spec.js#L2671)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-54/AK1 | Beim allerersten Start (es gibt noch keine `einstellungen.json`) erscheint der Dialog «Willkommen bei VivaPlan». Er enthält einen Satz zum Zweck der App und drei Knöpfe: «Beispiel erkunden», «Eigene Planung beginnen» und «Leere Planung». | ☑ | **gilt** | [app/src/main.rs:1934](../crates/app/src/main.rs#L1934), [ui.spec.js:2672](../e2e/ui.spec.js#L2672) | – |
| FEA-54/AK2 | «Beispiel erkunden» lässt die Beispielplanung geladen, wie heute, und öffnet die Hilfe beim Thema «Was VivaPlan kann» (`@start`). | ☑ | **gilt** | [ui.spec.js:2686](../e2e/ui.spec.js#L2686) | – |
| FEA-54/AK3 | «Eigene Planung beginnen» lädt die Beispielplanung als Vorlage, öffnet den Abschnitt «Haushalt & Projekt» und blendet die Checkliste «Erste Schritte» ein (FEA-55). Solange FEA-55 noch nicht umgesetzt ist, öffnet sich stattdessen die Hilfe beim Schnellstart (`@quickstart`). | ☑ | **gilt**: Nacharbeit laut Kommentar: Die Checkliste wird in den Blick gescrollt und erhält den Fokus. | [ui.spec.js:2695](../e2e/ui.spec.js#L2695), [ui.spec.js:2803](../e2e/ui.spec.js#L2803) | – |
| FEA-54/AK4 | «Leere Planung» wirkt genau wie Datei → Neu. | ☑ | **gilt** | [ui.spec.js:2709](../e2e/ui.spec.js#L2709) | – |
| FEA-54/AK5 | Esc oder der Schliessen-Knopf wirken wie «Beispiel erkunden», nur ohne die Hilfe zu öffnen. | ☑ | **gilt** | [ui.spec.js:2719](../e2e/ui.spec.js#L2719) | – |
| FEA-54/AK6 | Bei späteren Starts erscheint der Dialog nicht mehr, auch nicht, wenn die zuletzt geöffnete Datei fehlt. | ☑ | **gilt** | [app/src/main.rs:1934](../crates/app/src/main.rs#L1934) | – |
| FEA-54/AK7 | Hilfe → «Willkommen …» öffnet den Dialog jederzeit. Bei ungespeicherten Änderungen fragt die App vor dem Laden nach, wie bei Datei → Neu. | ☑ | **gilt** | [app/src/main.rs:1956](../crates/app/src/main.rs#L1956), [ui.spec.js:2738](../e2e/ui.spec.js#L2738) | – |
| FEA-54/AK8 | Der Dialog lässt sich ganz mit der Tastatur bedienen (Tab, Enter, Esc). | ☑ | **gilt** | [ui.spec.js:2719](../e2e/ui.spec.js#L2719) | – |
| FEA-54/AK9 | Die Hilfe (`00-einstieg.md`, `@bedienung`), `docs/BESCHREIBUNG.md` und die Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test | – |

### #71 FEA-55 Checkliste «Erste Schritte» im Editor

[Issue #71](https://github.com/ingen63/VivaPlan/issues/71) · geschlossen · Umsetzung: `7ba159d` (altes Repository) · Tests zum Issue ohne Kriterium: [ui.spec.js:2665](../e2e/ui.spec.js#L2665), [ui.spec.js:2753](../e2e/ui.spec.js#L2753)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-55/AK1 | Im Editor gibt es einen Kasten «Erste Schritte» mit den acht Schritten des Schnellstarts: Haushalt & Projekt, Meilensteine, Vorsorge, Einnahmen & Ausgaben, Vermögen & Anlage, Renditen & Daten, Steuern, Simulation starten. | ☑ | **gilt**: Der sechste Schritt heisst seit FEA-53 «Simulation» statt «Renditen & Daten» (Kommentar im Issue). | [ui.spec.js:2754](../e2e/ui.spec.js#L2754) | – |
| FEA-55/AK2 | Ein Klick auf einen Schritt öffnet dessen Abschnitt. Das Zeichen ⓘ öffnet die Hilfe zum Schritt. | ☑ | **gilt** | [ui.spec.js:2763](../e2e/ui.spec.js#L2763) | – |
| FEA-55/AK3 | Ein Abschnitt-Schritt gilt als erledigt, sobald darin mindestens ein Wert geändert wurde oder der Benutzer ihn von Hand abhakt. «Simulation starten» ist erledigt, sobald eine Simulation ohne Abbruch durchgelaufen ist. | ☑ | **gilt** | [ui.spec.js:2772](../e2e/ui.spec.js#L2772), [ui.spec.js:2790](../e2e/ui.spec.js#L2790) | – |
| FEA-55/AK4 | Ein Haken bleibt, auch wenn die Änderung mit ⌘Z rückgängig gemacht wird. | ☑ | **gilt** | [ui.spec.js:2772](../e2e/ui.spec.js#L2772) | – |
| FEA-55/AK5 | Der Fortschritt steht im Kopf des Kastens, zum Beispiel «3 von 8». | ☑ | **gilt** | [ui.spec.js:2754](../e2e/ui.spec.js#L2754) | – |
| FEA-55/AK6 | Mit × lässt sich der Kasten ausblenden, über Hilfe → «Erste Schritte» wieder einblenden. | ☑ | **gilt** | [app/src/main.rs:1956](../crates/app/src/main.rs#L1956), [ui.spec.js:2797](../e2e/ui.spec.js#L2797) | – |
| FEA-55/AK7 | Sind alle acht Schritte erledigt, erscheint ein kurzer Hinweis. Danach klappt der Kasten zu. | ☑ | **gilt** | [ui.spec.js:2797](../e2e/ui.spec.js#L2797) | – |
| FEA-55/AK8 | Wählt der Benutzer im Willkommensdialog (FEA-54) «Eigene Planung beginnen», wird die Checkliste eingeblendet und zurückgesetzt, auch wenn er sie vorher ausgeblendet hatte. Alle Schritte stehen dann auf offen. | ☑ | **gilt** | [ui.spec.js:2813](../e2e/ui.spec.js#L2813) | – |
| FEA-55/AK9 | Der Fortschritt wird in `einstellungen.json` gespeichert und gilt für den Benutzer, nicht für eine einzelne Planung. Das `.vplan`-Format bleibt unverändert. | ☑ | **gilt** | [ui.spec.js:2772](../e2e/ui.spec.js#L2772) | – |
| FEA-55/AK10 | Die Hilfe, `docs/BESCHREIBUNG.md` und die Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test | – |

### #72 FEA-56 Ergebnis in Worten in der Übersicht

[Issue #72](https://github.com/ingen63/VivaPlan/issues/72) · geschlossen · Umsetzung: `7ba159d` (altes Repository) · Tests zum Issue ohne Kriterium: [ui.spec.js:2665](../e2e/ui.spec.js#L2665), [ui.spec.js:2825](../e2e/ui.spec.js#L2825)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-56/AK1 | **Mehrere Läufe:** Die Einordnung steht in Klammern hinter der Zahl, zum Beispiel «In 87 % (mittel) der Läufe reicht das Vermögen bis 12.2055 (Alter 95).» Die Einordnung ist ein Link auf die Hilfe «Wie viel Erfolgsquote ist genug?» (`@r-overview`). Die Grenzen sind dieselben wie bei der Ampel: «hoch» ab 95 %, «mittel» von 80 % bis unter 95 %, «tief» unter 80 %. | ☑ | **gilt** | [ui.spec.js:2826](../e2e/ui.spec.js#L2826) | – |
| FEA-56/AK2 | An den Grenzen gilt 95,0 % als «hoch» und 80,0 % als «mittel». Massgebend ist derselbe gerundete Wert, den die Kennzahl «Erfolgsquote» anzeigt. | ☑ | **gilt** | [ui.spec.js:2842](../e2e/ui.spec.js#L2842) | – |
| FEA-56/AK3 | Geht in mindestens einem Lauf das Geld aus, folgt der Satz «Im ungünstigsten Lauf ist das Geld mit Alter … aufgebraucht.» Er nennt dasselbe Alter wie die Kennzahl «Früheste Erschöpfung». | ☑ | **gilt** | [ui.spec.js:2826](../e2e/ui.spec.js#L2826) | – |
| FEA-56/AK4 | **Ein einzelner Lauf:** «Das Vermögen reicht bis zum Ende der Planung und beträgt dann CHF ….» oder «Das Vermögen ist im 03.2049 (Alter 84) aufgebraucht.» | ☑ | **gilt** | [ui.spec.js:2849](../e2e/ui.spec.js#L2849) | – |
| FEA-56/AK5 | Beträge im Text folgen dem Umschalter «Nominal» bzw. «Heutige Kaufkraft». | ☑ | **gilt** | [ui.spec.js:2849](../e2e/ui.spec.js#L2849) | – |
| FEA-56/AK6 | Ein Hinweis auf den nächsten Schritt mit Link in die Hilfe: Liegt die Erfolgsquote unter 95 %, verweist er auf die Optimierung. Sonst verweist er auf die Szenarien. | ☑ | **gilt** | [ui.spec.js:2826](../e2e/ui.spec.js#L2826), [ui.spec.js:2868](../e2e/ui.spec.js#L2868) | – |
| FEA-56/AK7 | Ergebnisse aus der Liste der Simulationen zeigen den Text ebenfalls. | ☑ | **gilt** | [ui.spec.js:2849](../e2e/ui.spec.js#L2849) | – |
| FEA-56/AK8 | Der Text entsteht in der Oberfläche aus den vorhandenen Kennzahlen. Der Rechenkern und die Golden-Dateien bleiben unverändert. | ☑ | **gilt** | kein Test | – |
| FEA-56/AK9 | Die Hilfe (`@r-overview`), `docs/BESCHREIBUNG.md` und die Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test | – |

### #73 FEA-57 Vermögen & Anlage: Quote von Aktien und Obligationen als Info neben den Beständen

[Issue #73](https://github.com/ingen63/VivaPlan/issues/73) · geschlossen · Umsetzung: `c5167d7` (altes Repository) · Tests zum Issue ohne Kriterium: [ui.spec.js:1897](../e2e/ui.spec.js#L1897)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-57/AK1 | Aktien 80 000, Obligationen 20 000, Cash 50 000: neben «Aktien» steht «80 %», neben «Obligationen» «20 %». | ☑ | **gilt** | kein Test | – |
| FEA-57/AK2 | Die Quote wird ganzzahlig gerundet (Aktien 1, Obligationen 2: «33 %» und «67 %»). | ☑ | **gilt** | kein Test | – |
| FEA-57/AK3 | Nach einer Änderung von Aktien oder Obligationen stimmt die Anzeige, sobald der Wert übernommen ist (beim Verlassen des Feldes, wie die übrigen Textfelder nach ISS-13). | ☑ | **gilt** | kein Test | – |
| FEA-57/AK4 | Sind Aktien und Obligationen beide 0 oder leer, erscheint keine Quote (kein «NaN», kein «0 %»). | ☑ | **gilt** | kein Test | – |
| FEA-57/AK5 | Cash allein ändert die Quoten nicht (Beispiel AK1 mit Cash 0 oder 500 000: weiterhin 80 % / 20 %). | ☑ | **gilt** | kein Test | – |
| FEA-57/AK6 | Die Quote wird nicht in der Planungsdatei gespeichert; Rechenergebnisse und Golden-Dateien bleiben unverändert. | ☑ | **gilt** | kein Test | – |
| FEA-57/AK7 | Die Hilfe zu den Beständen erwähnt die angezeigte Quote. | ☑ | **gilt** | kein Test | – |

### #74 FEA-58 Hinweis zur Cash-Reserve nur einmal, beim ersten Laden einer betroffenen Planung

[Issue #74](https://github.com/ingen63/VivaPlan/issues/74) · geschlossen · Umsetzung: `b73b0d8` (altes Repository)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-58/AK1 | Das erste Laden einer betroffenen Planung nach dem Update (beim Start, über «Öffnen» oder per Doppelklick) zeigt den Hinweis einmal. | ☑ | **gilt** | [ui.spec.js:2049](../e2e/ui.spec.js#L2049) | – |
| FEA-58/AK2 | Jeder weitere Start mit derselben oder einer anderen betroffenen Planung: Kein Hinweis. | ☑ | **gilt** | [ui.spec.js:2057](../e2e/ui.spec.js#L2057) | – |
| FEA-58/AK3 | Nachdem der Hinweis einmal erschienen ist, erscheint er auch beim Öffnen einer betroffenen Planung über «Öffnen» oder per Doppelklick nicht mehr. | ☑ | **gilt** | [ui.spec.js:1959](../e2e/ui.spec.js#L1959) | – |
| FEA-58/AK4 | Nicht betroffene Planungen (z. B. Strategie «Standard») zeigen keinen Hinweis und zählen nicht als «gezeigt»: Beispiel: erster Start mit einer Planung «Standard» ohne Hinweis, danach Öffnen einer betroffenen Planung zeigt den Hinweis einmal. | ☑ | **gilt** | [ui.spec.js:2064](../e2e/ui.spec.js#L2064) | – |
| FEA-58/AK5 | Dass der Hinweis gezeigt wurde, merkt sich die App in den lokalen Einstellungen (nicht in der Planungsdatei); die Planungsdatei bleibt unverändert. | ☑ | **gilt** | [ui.spec.js:1959](../e2e/ui.spec.js#L1959), [ui.spec.js:2049](../e2e/ui.spec.js#L2049) | – |
| FEA-58/AK6 | Die Rechenergebnisse ändern sich nicht (der Hinweis ist nur eine Information); Golden-Dateien unverändert. | ☑ | **gilt** | kein Test | – |
| FEA-58/AK7 | Hilfe und Spezifikation (FEA-17) beschreiben das neue Verhalten. | ☑ | **gilt** | kein Test | – |

### #75 ISS-15 Rauchtest: Prüfung der Jahreswerte erwartet veraltete Steuerspalten

[Issue #75](https://github.com/ingen63/VivaPlan/issues/75) · geschlossen · Umsetzung: `96526b0` (altes Repository)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| ISS-15/AK1 | `tools/ui-smoke.sh` meldet für `#results/bootstrap/table` «ok», wenn die Jahreswerte vollständig sind. | ☑ | **gilt** | kein Test | – |
| ISS-15/AK2 | Die Prüfung erkennt weiterhin, wenn eine geforderte Spalte fehlt (z. B. «Ausgaben total»). | ☑ | **gilt** | kein Test | – |
| ISS-15/AK3 | Die Steuerspalten werden mit den heutigen Namen geprüft, bei eingeblendeten Steuerspalten (oder die Prüfung prüft bewusst den eingeklappten Standard und die Begründung steht im Skript). | ☑ | **gilt** | kein Test | – |

### #76 FEA-59 Obligationen: kurzfristige Eidgenossen in der Anlagestrategie

[Issue #76](https://github.com/ingen63/VivaPlan/issues/76) · geschlossen · Umsetzung: `9c44012` (altes Repository)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-59/AK1 | Bei den Obligationen gibt es die Wahl «CH kurzfristig» neben CH, CH Pictet und USA. Hilfe und Info-Fenster nennen Quelle, Zeitraum und dass die Rendite praktisch dem Geldmarktsatz entspricht. | ☑ | **gilt**: «CH Pictet» gibt es seit FEA-72 nicht mehr. | [engine/tests/strategy.rs:369](../crates/engine/tests/strategy.rs#L369), [ui.spec.js:178](../e2e/ui.spec.js#L178) | – |
| FEA-59/AK2 | Die Jahresrenditen stammen bis 2020 aus der JST-Datenbank (`bill_rate`, Schweiz, lückenlos ab 1870, verwendet ab 1900) und danach aus einer Reihe der SNB. Der Übergang hat keine Lücke; ein Test rechnet ein Jahr vor und ein Jahr nach dem Übergang von Hand nach. | ☑ | **gilt** | [engine/tests/strategy.rs:343](../crates/engine/tests/strategy.rs#L343) | – |
| FEA-59/AK3 | Eine Strategie mit «CH kurzfristig» rechnet jährlich über die gemeinsamen Jahre mit der gewählten Aktienreihe, wie die anderen gemischten Kombinationen (FEA-53). | ☑ | **gilt** | [engine/tests/strategy.rs:369](../crates/engine/tests/strategy.rs#L369), [ui.spec.js:178](../e2e/ui.spec.js#L178) | – |
| FEA-59/AK4 | Bestehende Planungen rechnen unverändert (Golden-Dateien ohne Änderung). | ☑ | **gilt** | kein Test | – |
| FEA-59/AK5 | QUELLEN.md nennt Quelle, Abrufdatum und Lizenz der neuen Daten. | ☑ | **gilt** | kein Test | – |

### #77 FEA-60 Konzept: VivaPlan in Claude nutzen (MCP)

[Issue #77](https://github.com/ingen63/VivaPlan/issues/77) · geschlossen · Umsetzung: `5f3475f` (altes Repository)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-60/AK1 | Das Konzept liegt als `docs/spezifikation/FEA-60 VivaPlan in Claude (MCP).md` vor. | ☐ | **gilt** | kein Test | – |
| FEA-60/AK2 | Es beschreibt mindestens zwei Varianten und vergleicht sie, mindestens: (a) lokaler MCP-Server (stdio) auf Basis von `vivaplan-engine`/`vivaplan-cli`, der Werkzeuge bereitstellt, und (b) die Oberfläche als interaktive Ansicht in Claude (MCP Apps / UI-Ressourcen). Es enthält eine Empfehlung mit Begründung. | ☐ | **gilt** | kein Test | – |
| FEA-60/AK3 | Es listet die Werkzeuge (Tools) mit Name, Zweck, Eingaben und Ausgaben, mindestens: Planung öffnen, prüfen, Werte ändern, Vorschau, Simulation, Optimierung, Ergebnisse abrufen und vergleichen, Planung speichern. | ☐ | **gilt** | kein Test | – |
| FEA-60/AK4 | Es legt fest, welche Daten an Claude gehen, und wie das zur Regel «Alle Daten bleiben lokal» passt (die App selbst bleibt ohne Netzwerkzugriff; der Benutzer entscheidet bewusst, Daten an Claude zu geben). Dazu Hinweise für den Benutzer in der Hilfe. | ☐ | **gilt** | kein Test | – |
| FEA-60/AK5 | Es beschreibt, wie Änderungen durch Claude sichtbar und rückgängig zu machen sind und dass Planungsdateien nie ohne Bestätigung überschrieben werden. | ☐ | **gilt** | kein Test | – |
| FEA-60/AK6 | Es beschreibt Grenzen und Risiken (lange Rechenzeiten und Abbruch, Grösse der Ergebnisse, Fehlbedienung, Haftungsausschluss FEA-45). | ☐ | **gilt** | kein Test | – |
| FEA-60/AK7 | Es beschreibt Installation und Einrichtung in Claude Desktop und Claude Code, die Tests (wie der MCP-Server automatisch geprüft wird) und eine Aufteilung der Umsetzung in Folge-Issues mit Abnahmekriterien. | ☐ | **gilt** | kein Test | – |
| FEA-60/AK8 | Rechenkern und Oberfläche bleiben getrennt: Das Konzept nutzt die bestehende Schnittstelle (`api`) des Rechenkerns, ohne Berechnungen doppelt zu führen. | ☐ | **gilt** | kein Test | – |

### #78 FEA-61 Testdateien MSCI.csv und beispiel.rplan durch erfundene Daten ersetzen

[Issue #78](https://github.com/ingen63/VivaPlan/issues/78) · geschlossen · Umsetzung: `590a43d` (altes Repository)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-61/AK1 | `crates/engine/tests/fixtures/MSCI.csv` und `crates/engine/tests/fixtures/beispiel.rplan` sind gelöscht; `git ls-files` findet keine Datei mehr, deren Pfad `MSCI` oder `beispiel.rplan` enthält. | ☑ | **gilt** | kein Test | – |
| FEA-61/AK2 | Eine erfundene Renditereihe (neutraler Dateiname ohne «MSCI», gleiches CSV-Format) ersetzt `MSCI.csv` in allen oben genannten Tests; die Tests prüfen weiterhin dasselbe Verhalten (Laden, historische Rendite, `series_info` relativ zur Planung). | ☑ | **gilt** | kein Test | – |
| FEA-61/AK3 | Eine erfundene Planung im alten Format `.rplan` (neutraler Name, keine echten Namen oder Beträge) ersetzt `beispiel.rplan` in `legacy_format.rs`; das Lesen des alten Formats bleibt abgedeckt. | ☑ | **gilt**: Die erfundene Planung heisst `crates/engine/tests/fixtures/planung-altformat.rplan`. | kein Test | – |
| FEA-61/AK4 | `make test` läuft grün, die Abdeckung von Engine und App bleibt je mindestens 80 %. | ☑ | **gilt** | kein Test | – |
| FEA-61/AK5 | Die Golden-Dateien (`crates/engine/tests/golden/`) bleiben unverändert. | ☑ | **gilt** | kein Test | – |

### #79 ISS-16 Oberflächentests schlagen bei vollem Lauf fehl

[Issue #79](https://github.com/ingen63/VivaPlan/issues/79) · geschlossen · Umsetzung: `62333d7` (altes Repository) · Tests zum Issue ohne Kriterium: [ui-cancel.spec.js:27](../e2e/ui-cancel.spec.js#L27), [ui.spec.js:29](../e2e/ui.spec.js#L29) (+3)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| ISS-16/AK1 | Die Ursache ist im Issue beschrieben. | ☑ | **gilt** | kein Test | – |
| ISS-16/AK2 | Drei volle Läufe nacheinander (`npx playwright test`, frischer Server) sind grün. | ☑ | **gilt** | kein Test | – |
| ISS-16/AK3 | Welcher Test hätte ihn fangen müssen / Regressionstest: im Issue beantwortet. | ☑ | **gilt** | kein Test | – |

### #90 FEA-72 Pictet-Reihe aus App und Repository entfernen

[Issue #90](https://github.com/ingen63/VivaPlan/issues/90) · geschlossen · Umsetzung: `6f1ab9e` (altes Repository) · Tests zum Issue ohne Kriterium: [ui.spec.js:200](../e2e/ui.spec.js#L200)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-72/AK1 | `crates/engine/data/Pictet CH.csv` ist gelöscht; `git ls-files` findet keine Datei, deren Pfad `Pictet` enthält, und kein `include_str!` verweist mehr darauf. | ☑ | **gilt** | [engine/tests/library.rs:140](../crates/engine/tests/library.rs#L140) | – |
| FEA-72/AK2 | Die Anlagestrategie bietet «Schweiz (Pictet)» weder bei den Aktien noch bei den Obligationen an (Auswahl in der Oberfläche, Strategieangaben des Rechenkerns). | ☑ | **gilt** | [ui.spec.js:258](../e2e/ui.spec.js#L258) | – |
| FEA-72/AK3 | Eine Planung mit `"strategy": {"stocks": "chPictet", …}` öffnet ohne Fehler und erhält für die Aktien «Schweiz (JST)» (`chJst`); eine Planung mit `"bonds": "chPictet"` erhält «Eidgenossenschaft 10 Jahre» (`ch`). Ein Hinweis beim Öffnen nennt die Umstellung. | ☑ | **gilt** | [engine/tests/strategy.rs:475](../crates/engine/tests/strategy.rs#L475), [ui.spec.js:258](../e2e/ui.spec.js#L258) | – |
| FEA-72/AK4 | Eine ältere Planung mit `"series": "library:Pictet CH.csv"` öffnet ohne Fehler, erhält `chJst` + `ch` und denselben Hinweis. | ☑ | **gilt** | [engine/tests/strategy.rs:185](../crates/engine/tests/strategy.rs#L185), [ui.spec.js:249](../e2e/ui.spec.js#L249) | – |
| FEA-72/AK5 | Die App legt `Pictet CH.csv` nicht mehr im Datenordner an. Eine dort schon vorhandene Kopie wird weder gelöscht noch verändert und nicht mehr als mitgelieferte Reihe angeboten. | ☑ | **gilt** | [engine/tests/library.rs:140](../crates/engine/tests/library.rs#L140) | – |
| FEA-72/AK6 | Hilfe (`20-renditen.md`, `10-editor.md`, `60-referenz.md`), `README.md`, `docs/BESCHREIBUNG.md`, `crates/engine/data/QUELLEN.md` und die Spezifikation FEA-53 sind nachgeführt. Die Erwähnung von Pictet als Quelle der Schweizer JST-Aktienreihe darf bleiben. | ☑ | **gilt** | kein Test | – |
| FEA-72/AK7 | `make test` und die Oberflächentests (`e2e/`) laufen grün; die Abdeckung von Engine und App bleibt je mindestens 80 %; die Golden-Dateien (`crates/engine/tests/golden/`) bleiben unverändert. | ☑ | **gilt** | kein Test | – |

### #92 FEA-74 Test für eigene Konfigurationen ohne feste Dateinamen

[Issue #92](https://github.com/ingen63/VivaPlan/issues/92) · offen · Umsetzung: `5158ee8` (altes Repository) · Tests zum Issue ohne Kriterium: [engine/tests/legacy_import.rs:48](../crates/engine/tests/legacy_import.rs#L48), [engine/tests/legacy_import.rs:84](../crates/engine/tests/legacy_import.rs#L84)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-74/AK1 | Der Test liest alle Dateien mit der Endung `.json` im Ordner `local/eigene/` statt einer festen Liste von Dateinamen. | ☐ | **gilt** | [engine/tests/legacy_import.rs:61](../crates/engine/tests/legacy_import.rs#L61) | – |
| FEA-74/AK2 | Fehlt der Ordner oder enthält er keine `.json`-Datei, wird der Test ohne Fehler übersprungen (wie heute). | ☐ | **gilt** | [engine/tests/legacy_import.rs:61](../crates/engine/tests/legacy_import.rs#L61) | – |
| FEA-74/AK3 | Im aktuellen Stand findet `git grep -i <Name aus FEA-74>` keine Stelle mehr. | ☐ | **gilt**: Der gesuchte Name ist hier weggelassen, damit die Suche diese Datei nicht findet; er steht im Issue. | kein Test | – |
| FEA-74/AK4 | Mit den bisherigen Dateien in `local/eigene/` prüft der Test dieselben Konfigurationen wie vorher; `make test` läuft grün. | ☐ | **gilt** | kein Test | – |

### #94 FEA-76 Shiller-Originaltabelle nicht im Repository, Bedingungen von SNB und BFS in den Quellen

[Issue #94](https://github.com/ingen63/VivaPlan/issues/94) · geschlossen · Umsetzung: `49bbd62` (altes Repository) · Tests zum Issue ohne Kriterium: [app/src/main.rs:1339](../crates/app/src/main.rs#L1339), [app/src/main.rs:1345](../crates/app/src/main.rs#L1345) (+6)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| FEA-76/AK1 | `git ls-files` findet `shiller-index.ods` nicht mehr; der Pfad steht in `.gitignore`; eine lokal vorhandene Datei bleibt unverändert. | ☑ | **gilt** | kein Test | – |
| FEA-76/AK2 | Die Tests, die aus der Tabelle nachrechnen (`crates/engine/tests/library.rs`, `crates/engine/tests/data.rs`, `crates/cli/tests/cli.rs`, `crates/app/src/main.rs`), laufen mit vorhandener Datei wie bisher. Fehlt sie, werden sie ohne Fehler übersprungen und melden das. | ☑ | **gilt** | kein Test | – |
| FEA-76/AK3 | Die ausgelieferten Reihen `Shiller USD.csv`, `Shiller CHF.csv` und `Shiller CHF + CH-Obligationen.csv` und die Golden-Dateien bleiben unverändert. | ☑ | **gilt** | kein Test | – |
| FEA-76/AK4 | `crates/engine/data/QUELLEN.md` sagt, dass die Shiller-Tabelle nicht mitgeliefert wird und wo man sie herunterlädt; es nennt für die SNB-Daten die Bedingungen «nicht kommerziell», «Quellenangabe» und «nicht als Grundlage für Finanztransaktionen» und für die BFS-Daten die Quellenangabe mit Autor, Titel und Link. | ☑ | **gilt** | kein Test | – |
| FEA-76/AK5 | Die Hilfe (Datenquellen) nennt den Download der Shiller-Tabelle bei shillerdata.com und den Import in der App. | ☑ | **gilt** | kein Test | – |
| FEA-76/AK6 | `make test` und die Oberflächentests laufen grün; die Abdeckung von Engine und App bleibt je mindestens 80 %, auch ohne die Datei (wie auf GitHub). | ☑ | **gilt** | kein Test | – |
