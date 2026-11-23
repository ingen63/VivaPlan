# Abnahmeliste

Stand 14.11.2026 (#97), nachgeführt am 15.11.2026 mit der Sammelprüfung aller Funktions-Issues (siehe Spalte «Geprüft»). Die Liste nennt die Abnahmekriterien aller umgesetzten Issues und schätzt ein, ob sie zum
heutigen Funktionsstand noch passen. Sie ist die Grundlage für Einzelprüfungen: Claude prüft auf Anweisung ein Issue oder
ein Kriterium und trägt das Ergebnis in der Spalte «Geprüft» ein.

## Auswahl

Einbezogen sind alle geschlossenen Issues und alle offenen Issues mit dem Label `implemented`. Nicht einbezogen sind nur
offene Issues ohne `implemented` (noch nicht umgesetzt). Die Issues stehen nach Nummer (`#115`); alte IDs wie `FEA-…` aus
der Git-Historie führt `docs/ISSUE-IDS.md` auf die Nummer. Abfrage zum Nachführen:

```bash
gh issue list --state all --limit 500 --json number,title,state,labels,body,comments \
  | jq '[.[] | select(.state == "CLOSED" or (.labels | map(.name) | index("implemented")))] | sort_by(.number)'
```

Kriterien sind die Häkchen im Abschnitt «Abnahmekriterien» des Issues, gezählt in ihrer Reihenfolge (`AK2` = zweites
Häkchen); ohne diesen Abschnitt alle Häkchen im Text. Kriterien, die später in einem Kommentar dazukamen, stehen mit dem
Vermerk «(Kommentar)» in der Liste. Änderungen an Kriterien per Kommentar sind in der Einschätzung genannt.

## Spalten

- **Im Issue:** Häkchen im Issue (☑ abgehakt, ☐ offen).
- **Einschätzung:** **gilt** (passt zum heutigen Stand, allenfalls mit Hinweis auf geänderte Namen oder Pfade),
  **überholt** (ein späteres Issue oder eine spätere Entscheidung hat das Verhalten geändert; der Grund steht dabei) oder
  **unklar** (braucht einen Entscheid).
- **Test:** Tests, die das Kriterium mit seiner Kennung nennen (`#Nr/AKn` im Kommentar, `issue_Nr_akn_…` im Rust-Testnamen, in `e2e/`
  auch als `AKn` innerhalb eines Blocks mit der Issue-Nummer). Tests, die nur das Issue nennen, stehen beim Issue. Die Zeilennummern
  gelten für den Stand dieser Liste.
- **Geprüft:** Datum und Ergebnis der letzten Prüfung: `ok` (Test oder Durchsicht belegt das Kriterium), `teilweise: …`
  (nur ein Teil belegt), `überholt, nicht geprüft`, `Fehler: …` oder `nicht geprüft: …` (kein Test, keine Durchsicht möglich;
  braucht eine Prüfung von Hand oder einen neuen Test). Die Sammelprüfung vom 15.11.2026 (#3 bis #94) stützt sich auf
  den grünen Lauf aller Testsuiten (612 Rust, 156 e2e, 14 UI-Unit, 9 Berichtswerkzeuge) und auf die Durchsicht von
  Dateien und Hilfetexten; die Desktop-App und GitHub-Workflows wurden dabei nicht gestartet.

## Vorgehen bei der Einzelprüfung

1. Anweisung zum Beispiel «Prüfe #55» oder «Prüfe #55/AK4».
2. Claude prüft das Kriterium am heutigen Stand: vorhandenen Test laufen lassen, sonst in der Oberfläche oder im Code
   nachsehen. Bei «überholt» wird nur geprüft, ob das neue Verhalten gilt.
3. Claude trägt das Ergebnis in den Spalten «Test» und «Geprüft» ein, schreibt einen Kommentar ins Issue, hakt erfüllte
   Kriterien dort ab und schliesst das Issue, wenn alle erfüllt sind (`CLAUDE.md`, Ablauf Schritt 6). Kriterien werden
   nicht geändert.
4. Ein Fehler wird als neues Issue vorgeschlagen (Label `bug`, `found: acceptance`). Ein überholtes Kriterium bekommt
   auf Wunsch einen Kommentar im Issue.

## Übersicht

| | Anzahl |
|---|---|
| Issues in der Liste | 87 (davon 5 ohne Kriterien) |
| Kriterien | 494 |
| gilt | 464 |
| überholt | 15 |
| unklar | 1 |
| im Issue nicht abgehakt | 12 |
| ohne Test mit Kennung | 176 |
| geprüft am 15.11.2026: ok | 346 |
| geprüft: teilweise | 39 |
| geprüft: nicht geprüft | 23 |
| geprüft: vom Anwender geschlossen (nicht mehr belegbar) | 3 |
| geprüft: überholt, nicht geprüft | 11 |

### Überholt oder unklar

| Kennung | Einschätzung |
|---|---|
| #4/AK3 | **überholt**: durch #70 und #61: Beim ersten Start erscheinen Haftungsausschluss und Willkommensdialog; ohne gemerkte Datei lädt die Desktop-App `new.vplan`. |
| #12/AK2 | **gilt** in neuer Fassung: Die Spalte «Zeitreihe» zeigt den Namen der Anlagestrategie bzw. der Reihe (Kommentar in #12 vom 15.11.2026). |
| #13/AK8 | **überholt**: durch #25: Die Schwelle ist keine Einstellung geworden und gilt fest mit 5 % (Kommentar in #25). |
| #18/AK3 | **überholt**: durch #74: Der Hinweis erscheint nur einmal, beim ersten Laden einer betroffenen Planung. |
| #19/AK1 | **überholt**: durch #69: Die Karte «Zeitreihe» gibt es nicht mehr; die Erklärung steht im Info-Popup der Anlagestrategie. |
| #19/AK4 | **überholt**: durch den Kommentar im Issue (Info in der Tabellenzeile) und danach #69 (keine Tabelle der Reihen mehr). |
| #19/AK5 | **überholt**: durch #69: Seit #69 gibt es keine eigenen Dateien und keinen Ordnerpfad mehr (Kommentar in #19 vom 15.11.2026). |
| #19/AK6 | **überholt**: durch den Kommentar im Issue (Texte gekürzt) und #69 (Texte zur Anlagestrategie). |
| #19/K1 | **überholt**: durch #69: keine Tabelle der Reihen mehr. |
| #19/K2 | **überholt**: durch #69: keine Tabelle der Reihen mehr; das Popup gibt es je Anlagestrategie. |
| #19/K3 | **unklar**: Gilt die Grenze von 2 Zeilen auch für die Texte der Anlagestrategie? |
| #19/K4 | **überholt**: durch #69 und #90: Die vier Reihen sind nicht mehr einzeln wählbar, Pictet ist entfernt. |
| #36/AK3 | **überholt**: durch die Arbeitsweise in `CLAUDE.md`: keine Pull Requests mehr, gearbeitet wird direkt auf `main`; sinngemäss für den Push prüfen. |
| #44/AK7 | **überholt**: durch #26: Das öffentliche Repository beginnt mit einem einzigen Commit; die Historie liegt nur im alten, privaten Repository. |
| #48/AK8 | **überholt**: durch #26: keine übernommene Historie im öffentlichen Repository. |
| #51/AK7 | **überholt**: durch #25: Die Grenzen kommen aus den Einstellungen. |
| #52/AK11 | **überholt**: durch #25: Der Maximalbetrag kommt aus den Einstellungen. |
| #56/AK4 | **überholt**: durch #26: Das öffentliche Repository hat die alte Historie nicht; `.git-blame-ignore-revs` ist entfernt. |
| #69/AK6 | **überholt**: durch #90: «Schweiz (Pictet)» ist entfernt. |

### Im Issue nicht abgehakt

#13/AK2, #47/AK1, #47/AK2, #47/AK3, #47/AK4, #47/AK5, #48/AK3, #77/AK1, #77/AK2, #77/AK3, #77/AK4, #77/AK5, #77/AK6, #77/AK7, #77/AK8, #92/AK1, #92/AK2, #92/AK3, #92/AK4

## Offene Punkte (Stand 15.11.2026)

### Von Hand beim Anwender

- **Mit lokalen Daten:** #92 AK4 (`local/eigene/`), #94 AK2 (Shiller-Datei), #43 AK3/AK4/AK6 (Laufzeiten, Mutantenvergleich), #40 AK1 (voller Mutationslauf), #38 AK2 bis AK4 (Agentenlauf, Laufzeit des Hooks), #44 AK8 (Repository-Umbenennung).
- **Auf GitHub nach dem Push:** #80 AK2 (Schwelle 99), #86 AK3, #89 AK1/AK2, #41 AK6, #43 AK5, #48 AK3.
- **Zu entscheiden:** #102 (#102, Laufzeit des Monte-Carlo-Tests, #85 AK3), #61 AK5 (#61, wieder öffnen?), #19 K3 (2-Zeilen-Grenze), #39 AK1/AK2 (Pull Requests gibt es nicht mehr), #50 AK5 (seit #51 kaum noch darstellbar), #20 AK4 (Vergleich Ziel 1 und Ziel 2, Test fehlt).
- **Hilfetexte und `BESCHREIBUNG.md`** sind nur stichwortartig abgeglichen, nicht Satz für Satz (#18, #25, #32, #50, #52, #51, #54 und weitere).

### Offene und nicht umgesetzte Issues

- **Offen, umgesetzt, Nachweis fehlt** (Stand 17.11.2026): #89 (#89, Nachtlauf mit e2e). Offen und nicht umgesetzt: #102.
- **Nicht umgesetzt, deshalb nicht geprüft:** #62 bis #66 (#62 bis #66), #91, #93, #100, #101.

## Issues


Abgeschlossene Issues (geschlossen, alle Kriterien `ok` oder überholt) stehen in [ABNAHME-ARCHIV.md](ABNAHME-ARCHIV.md); die Übersicht oben zählt beide Dateien. Nach einer Einzelprüfung, die ein Issue abschliesst, wandert sein Abschnitt dorthin.

### #7 Jahreswerte: Entnahmerate als Spalte

[Issue #7](https://github.com/ingen63/VivaPlan/issues/7) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #7/AK1 | Entnahme 40'000 pro Jahr bei Total 1'000'000 und Liquid 500'000 am Jahresanfang ergibt 4.0 % bzw. 8.0 %. | ☑ | **gilt** | [ui-ergebnisse.spec.js:475](../e2e/ui-ergebnisse.spec.js#L475) | 15.11.2026 teilweise: Total-Spalte gerechnet; Liquid-Spalte nicht im Test |
| #7/AK2 | Im ersten Jahr einer im November startenden Simulation: Entnahme der zwei Monate mal 6, geteilt durch das Startvermögen. | ☑ | **gilt** | [ui-ergebnisse.spec.js:475](../e2e/ui-ergebnisse.spec.js#L475) | 15.11.2026 ok (erstes Jahr mit Startvermögen als Basis) |
| #7/AK3 | Jahre mit Einkommen über den Ausgaben zeigen leere Zellen. | ☑ | **gilt** | [ui-ergebnisse.spec.js:475](../e2e/ui-ergebnisse.spec.js#L475) | 15.11.2026 ok (leere Zellen bei Entnahme 0) |
| #7/AK4 | Wechsel nominal/real ändert die Werte nicht. | ☑ | **gilt** | [ui-ergebnisse.spec.js:475](../e2e/ui-ergebnisse.spec.js#L475) | 15.11.2026 ok (nominal gleich real in der Rate) |
| #7/AK5 | Jede Zeile stimmt mit der manuellen Rechnung aus den Spalten «Entnahme», «Total» und «Liquid» (Test mit `examples/Test 1.rplan`). | ☑ | **gilt**: Die Planung `examples/Test 1.rplan` gibt es nicht mehr (#26); prüfen mit der Beispielplanung `example.vplan` oder einer erfundenen Planung. | [ui-ergebnisse.spec.js:475](../e2e/ui-ergebnisse.spec.js#L475) | 15.11.2026 ok (Zeilen stimmen mit der Rechnung aus Entnahme und Vermögen) |

### #11 Gespeicherte Ergebnisse: Strategie-Tooltip mit Parametern

[Issue #11](https://github.com/ingen63/VivaPlan/issues/11) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui-gespeichert.spec.js:295](../e2e/ui-gespeichert.spec.js#L295)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #11/AK1 | Mouseover über «3-Topf» zeigt Jahre Topf 1 und 2 und die Renditeschwelle mit den Werten des Ergebnisses (nicht der aktuellen Planung). | ☑ | **gilt** | [ui-gespeichert.spec.js:32](../e2e/ui-gespeichert.spec.js#L32) | 15.11.2026 ok (Tooltip zeigt Werte des Ergebnisses, nicht der aktuellen Planung) |
| #11/AK2 | Jede der sechs Strategien zeigt genau ihre Parameter. | ☑ | **gilt**: Heute sieben Strategien (#6: «3-Topf + Guardrails»). | [ui-gespeichert.spec.js:32](../e2e/ui-gespeichert.spec.js#L32) | 15.11.2026 teilweise: nur 3-Topf im Test |
| #11/AK3 | Die Werte stimmen mit dem Reiter «Parameter» (#5) überein. | ☑ | **gilt** | [ui-einstieg.spec.js:241](../e2e/ui-einstieg.spec.js#L241) | 15.11.2026 ok (Test) |
| #11/AK4 | Der Tooltip ist per Tastaturfokus erreichbar. | ☑ | **gilt** | [ui-gespeichert.spec.js:32](../e2e/ui-gespeichert.spec.js#L32) | 15.11.2026 ok (Tastaturfokus zeigt den Tooltip) |
| #11/AK5 | Ein altes Ergebnis ohne Kurzform zeigt «Parameter nicht verfügbar», ohne Fehler. | ☑ | **gilt** | [ui-gespeichert.spec.js:32](../e2e/ui-gespeichert.spec.js#L32) | 15.11.2026 ok («Parameter nicht verfügbar») |
| #11/AK6 | Die Liste lädt nicht langsamer (Ergebnisdaten werden dafür nicht geladen). | ☑ | **gilt** | kein Test | 15.11.2026 ok (Prüfung in der App vom Anwender) |

### #13 Gespeicherte Ergebnisse: 5-%-Vermögen und Warnsymbol bei Notverkauf

[Issue #13](https://github.com/ingen63/VivaPlan/issues/13) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui-gespeichert.spec.js:74](../e2e/ui-gespeichert.spec.js#L74)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #13/AK1 | Ein Monte-Carlo-Ergebnis zeigt «Erfolgsquote · 5 %: Betrag», der Betrag entspricht dem 5. Perzentil der Ergebnistabelle der Übersicht; der Median steht im Tooltip. | ☑ | **gilt** | [ui-gespeichert.spec.js:74](../e2e/ui-gespeichert.spec.js#L74) | 15.11.2026 ok («90 % · 5 %: 1.2 Mio», Median im Tooltip) |
| #13/AK2 | Mit `examples/Test 1.rplan` und einem Notverkauf in mindestens 5 % der Läufe erscheint das Symbol, der Tooltip nennt Anteil und Anzahl Läufe. | ☑ | **gilt**: Die Planung `examples/Test 1.rplan` gibt es nicht mehr (#26); prüfen mit der Beispielplanung `example.vplan` oder einer erfundenen Planung. Im Issue nicht abgehakt. | [ui-gespeichert.spec.js:74](../e2e/ui-gespeichert.spec.js#L74) | 15.11.2026 ok (Tooltip nennt Anteil und Anzahl Läufe) |
| #13/AK3 | Bei weniger als 5 % Läufen mit Notverkauf erscheint kein Symbol. | ☑ | **gilt** | [ui-gespeichert.spec.js:74](../e2e/ui-gespeichert.spec.js#L74) | 15.11.2026 ok |
| #13/AK4 | Eine Planung ohne Notverkauf oder ohne Immobilien zeigt kein Symbol. | ☑ | **gilt** | [ui-gespeichert.spec.js:74](../e2e/ui-gespeichert.spec.js#L74) | 15.11.2026 ok |
| #13/AK5 | Ein geplanter Verkauf löst das Symbol nicht aus. | ☑ | **gilt** | [ui-einstieg.spec.js:262](../e2e/ui-einstieg.spec.js#L262) | 15.11.2026 ok (Test) |
| #13/AK6 | Ein Einzellauf mit Notverkauf zeigt das Symbol. | ☑ | **gilt** | [ui-gespeichert.spec.js:74](../e2e/ui-gespeichert.spec.js#L74) | 15.11.2026 ok |
| #13/AK7 | Alte Einträge ohne Angabe werden fehlerfrei und ohne Symbol angezeigt. | ☑ | **gilt** | [ui-gespeichert.spec.js:74](../e2e/ui-gespeichert.spec.js#L74) | 15.11.2026 ok |
| #13/AK8 | Wird in den Einstellungen (#25) eine andere Schwelle gewählt, gilt sie. | ☑ | **überholt**: durch #25: Die Schwelle ist keine Einstellung geworden und gilt fest mit 5 % (Kommentar in #25). | kein Test | nicht geprüft: überholt durch #25 (Schwelle fest 5 %) |

### #15 Meilensteine: «Verwendet» entfällt, Löschen mit Umstellung

[Issue #15](https://github.com/ingen63/VivaPlan/issues/15) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui-start.spec.js:334](../e2e/ui-start.spec.js#L334), [ui-start.spec.js:373](../e2e/ui-start.spec.js#L373)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #15/AK1 | Keine Spalte «Verwendet». | ☑ | **gilt** | [ui-start.spec.js:334](../e2e/ui-start.spec.js#L334) | 15.11.2026 ok |
| #15/AK2 | Meilenstein «AHV» löschen: Dialog nennt Verwendung und Datum; danach zeigt die AHV-Rente das feste Datum, die Rechnung liefert dieselben Zahlen wie vorher. | ☑ | **gilt** | [ui-start.spec.js:334](../e2e/ui-start.spec.js#L334) | 15.11.2026 ok (Dialog, Datum, gleiches Endvermögen) |
| #15/AK3 | Bei «Alter von» wird das berechnete Datum eingetragen. | ☑ | **gilt** | [ui-start.spec.js:373](../e2e/ui-start.spec.js#L373) | 15.11.2026 ok |
| #15/AK4 | Rückgängig stellt Meilenstein und Verwendungen wieder her. | ☑ | **gilt** | [ui-start.spec.js:334](../e2e/ui-start.spec.js#L334) | 15.11.2026 ok (Rückgängig) |
| #15/AK5 | «Pensionierung», «Simulationsstart» und «Simulationsende» lassen sich nicht löschen. | ☑ | **gilt**: «Pensionierung» heisst seit #51 «Bezug Pensionskasse»; gesperrt sind heute Simulationsstart, Simulationsende, Bezug Pensionskasse und AHV-Bezug (#51/AK1). | [ui-start.spec.js:334](../e2e/ui-start.spec.js#L334) | 15.11.2026 ok |
| #15/AK6 | Nicht verwendete Meilensteine lassen sich ohne Dialog löschen. | ☑ | **gilt** | [ui-start.spec.js:373](../e2e/ui-start.spec.js#L373) | 15.11.2026 ok |
| #15/AK7 | Doppelter Zeitpunkt in einer Reihe: genau eine Zeile bleibt, mit Hinweis. | ☑ | **gilt** | [ui-start.spec.js:373](../e2e/ui-start.spec.js#L373) | 15.11.2026 ok |
| #15/AK8 | Variation und Optimierung laufen nach dem Löschen eines anderen Meilensteins unverändert. | ☑ | **gilt** | [ui-start.spec.js:373](../e2e/ui-start.spec.js#L373) | 15.11.2026 teilweise: Parameter «Pensionierung» bleibt wählbar; Lauf nicht verglichen |

### #18 Liquiditätsreserve nur für die Strategie «Standard»

[Issue #18](https://github.com/ingen63/VivaPlan/issues/18) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/reserve_standard_only.rs:1](../crates/engine/tests/reserve_standard_only.rs#L1), [ui-optimierung.spec.js:61](../e2e/ui-optimierung.spec.js#L61)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #18/AK1 | «Vermögen & Anlage» enthält die Reserve nicht mehr, die Karte «Standard» enthält sie. | ☑ | **gilt**: «Liquiditätsreserve» heisst seit #54 «Cash-Reserve». | [ui-optimierung.spec.js:61](../e2e/ui-optimierung.spec.js#L61) | 15.11.2026 ok |
| #18/AK2 | Eine Planung mit Strategie Standard und Reserve 6 liefert dieselben Ergebnisse wie vor der Änderung (Regressionstest mit den Golden-Dateien). | ☑ | **gilt** | [tests/reserve_standard_only.rs:39](../crates/engine/tests/reserve_standard_only.rs#L39) | 15.11.2026 ok (Standard rechnet mit der Reserve; Golden-Dateien grün) |
| #18/AK3 | Eine Planung mit Guardrails und Reserve 6 rechnet mit 3 Monaten, der Hinweis erscheint beim Laden einmal. | ☑ | **überholt**: durch #74: Der Hinweis erscheint nur einmal, beim ersten Laden einer betroffenen Planung. | [ui-optimierung.spec.js:61](../e2e/ui-optimierung.spec.js#L61) | 15.11.2026 ok (Hinweis einmal (jetzt nach #74)) |
| #18/AK4 | Die Optimierung bietet die Reserve weiterhin nur für Standard an. | ☑ | **gilt** | [optimize_ranges.rs:168](../crates/engine/tests/optimize_ranges.rs#L168) | 15.11.2026 ok (Test) |
| #18/AK5 | Hilfe, Beschreibung und Tests sind nachgeführt. | ☑ | **gilt** | [ui-optimierung.spec.js:61](../e2e/ui-optimierung.spec.js#L61) | 15.11.2026 teilweise: Hilfe und Beschreibung nicht Satz für Satz verglichen |

### #20 Optimierung: Ziel wählbar

[Issue #20](https://github.com/ingen63/VivaPlan/issues/20) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/optimize_goal.rs:1](../crates/engine/tests/optimize_goal.rs#L1), [ui-optimierung.spec.js:300](../e2e/ui-optimierung.spec.js#L300)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #20/AK1 | Die Optimierung hat die Auswahl «Ziel»; Standard ist Ziel 1; die Ergebnisse für Ziel 1 sind identisch mit denen vor der Änderung (Regressionstest). | ☑ | **gilt** | [tests/optimize_goal.rs:44](../crates/engine/tests/optimize_goal.rs#L44) | 15.11.2026 ok (Ziel 1 ist Standard und wertet wie bisher) |
| #20/AK2 | Bei Ziel 2 zeigt die Ergebnistabelle je Strategie den Anteil Läufe mit ungeplantem Immobilienverkauf; die Reihenfolge richtet sich nach diesem Anteil, bei Gleichstand nach der Ruinwahrscheinlichkeit. | ☑ | **gilt** | [tests/optimize_goal.rs:61](../crates/engine/tests/optimize_goal.rs#L61) | 15.11.2026 ok (sortiert nach Anteil, dann Ruin) |
| #20/AK3 | Strategien mit Konsum unter K % der geplanten Ausgaben werden nicht als beste gewählt. | ☑ | **gilt** | [tests/optimize_goal.rs:76](../crates/engine/tests/optimize_goal.rs#L76) | 15.11.2026 ok (Mindestkonsum als Nebenbedingung) |
| #20/AK4 | Mit `examples/Test 1.rplan` liefert Ziel 2 eine Strategie mit höchstens so vielen ungeplanten Verkäufen wie Ziel 1. | ☑ | **gilt**: Die Planung `examples/Test 1.rplan` gibt es nicht mehr (#26); prüfen mit der Beispielplanung `example.vplan` oder einer erfundenen Planung. | kein Test | nicht geprüft: Beispielplanung nicht verglichen |
| #20/AK5 | Planungen ohne Immobilien: Ziel 2 ist nicht wählbar (mit Hinweis). | ☑ | **gilt** | [tests/optimize_goal.rs:136](../crates/engine/tests/optimize_goal.rs#L136) | 15.11.2026 ok (Ziel 2 braucht eine Immobilie) |
| #20/AK6 | Das Ziel steht in der Plandatei und in der Hilfe. | ☑ | **gilt** | [tests/optimize_goal.rs:155](../crates/engine/tests/optimize_goal.rs#L155) | 15.11.2026 ok (Ziel in der Plandatei; Hilfe nicht verglichen) |

### #25 Einstellungen (Grenzen AHV-/PK-Bezug, Maximalbetrag 3a)

[Issue #25](https://github.com/ingen63/VivaPlan/issues/25) · geschlossen · Umsetzung: `477381f` (altes Repository) · Tests zum Issue ohne Kriterium: [engine/src/upgrade.rs:488](../crates/engine/src/upgrade.rs#L488), [engine/src/validate.rs:1475](../crates/engine/src/validate.rs#L1475) (+3)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #25/AK1 | Die App hat einen erreichbaren Bereich «Einstellungen» mit den fünf Einstellungen der Tabelle, je mit Beschriftung, Einheit und Standardwert. | ☑ | **gilt**: Laut Kommentar fünf Einstellungen (die Schwelle von #13 entfällt). | [engine/src/settings.rs:69](../crates/engine/src/settings.rs#L69), [ui-optimierung.spec.js:421](../e2e/ui-optimierung.spec.js#L421) | 15.11.2026 ok (Test grün) |
| #25/AK2 | Eine Änderung wird gespeichert und gilt nach einem Neustart der App weiterhin. Ohne gespeicherten Wert gilt der Standardwert. | ☑ | **gilt** | [ui-optimierung.spec.js:421](../e2e/ui-optimierung.spec.js#L421) | 15.11.2026 ok (Test grün) |
| #25/AK3 | Eine Schaltfläche «Standardwerte» setzt alle Einstellungen (nach Rückfrage) auf die Standardwerte zurück. | ☑ | **gilt** | [ui-optimierung.spec.js:421](../e2e/ui-optimierung.spec.js#L421) | 15.11.2026 ok (Test grün) |
| #25/AK4 | Prüfung der Eingabe: Die untere Grenze ist höchstens so gross wie die obere Grenze. Alter sind ganze Jahre. Der Maximalbetrag ist 0 oder mehr. Ein ungültiger Wert wird mit klarer Meldung abgelehnt und nicht gespeichert (Beispiel: untere Grenze AHV 71 bei oberer Grenze 70). | ☑ | **gilt** | [engine/src/settings.rs:83](../crates/engine/src/settings.rs#L83), [ui-optimierung.spec.js:421](../e2e/ui-optimierung.spec.js#L421) | 15.11.2026 ok (Test grün) |
| #25/AK5 | Die Grenzen für AHV-Bezug und Bezug Pensionskasse (#51) und der Maximalbetrag der 3a (#52) werden an einer Stelle aus den Einstellungen gelesen und dem Rechenkern als Parameter übergeben. Der Rechenkern kennt die Standardwerte als Rückfall (für Kommandozeile und Tests). | ☑ | **gilt** | [engine/tests/milestones_limits.rs:169](../crates/engine/tests/milestones_limits.rs#L169) | 15.11.2026 ok (Test grün) |
| #25/AK6 | Beispiel: Obere Grenze Bezug Pensionskasse 67 statt 65: Ein Bezug mit 67 Jahren ist gültig, mit 68 wird er abgelehnt. Der Standard 65 des Meilensteins ändert sich dadurch nicht. | ☑ | **gilt** | [engine/tests/milestones_limits.rs:169](../crates/engine/tests/milestones_limits.rs#L169), [ui-optimierung.spec.js:456](../e2e/ui-optimierung.spec.js#L456) | 15.11.2026 ok (Test grün) |
| #25/AK7 | Die Einstellungen ändern keine Planungsdatei und kein gespeichertes Ergebnis. Liegt eine Planung nach einer Änderung ausserhalb der neuen Grenzen, meldet die Prüfung das mit klarer Meldung (siehe #51). | ☑ | **gilt** | [ui-abnahme.spec.js:67](../e2e/ui-abnahme.spec.js#L67) | 15.11.2026 ok (Test) |
| #25/AK8 | Hilfe, `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: Hilfe nicht Satz für Satz verglichen |

### #27 Beenden mit ungespeicherten Änderungen fragt nicht nach

[Issue #27](https://github.com/ingen63/VivaPlan/issues/27) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui-optimierung.spec.js:235](../e2e/ui-optimierung.spec.js#L235)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #27/AK1 | Schliessen-Knopf mit Änderungen: Dialog erscheint, «Abbrechen» lässt die App offen. | ☑ | **gilt** | kein Test | – |
| #27/AK2 | ⌘Q mit Änderungen: dasselbe Verhalten. | ☑ | **gilt** | kein Test | – |
| #27/AK3 | «Speichern» speichert (bei neuer Planung mit Ortsabfrage) und beendet; bricht die Ortsabfrage ab, bleibt die App offen. | ☑ | **gilt** | kein Test | – |
| #27/AK4 | «Verwerfen» beendet ohne Speichern. | ☑ | **gilt** | kein Test | – |
| #27/AK5 | Ohne Änderungen schliesst die App ohne Dialog. | ☑ | **gilt** | kein Test | – |
| #27/AK6 | Test für beide Wege (Fenster schliessen, ⌘Q). | ☑ | **gilt** | kein Test | – |

### #28 Immobilien: Verkauf auch bei «Kauf geplant» erfassen

[Issue #28](https://github.com/ingen63/VivaPlan/issues/28) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui-optimierung.spec.js:395](../e2e/ui-optimierung.spec.js#L395)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #28/AK1 | Bei «Kauf geplant» erscheint das Feld «Verkauf ab» mit Vorgabe «Nie». | ☑ | **gilt** | kein Test | – |
| #28/AK2 | Datum und Meilenstein sind wählbar; die Simulation verkauft das Objekt zum gewählten Zeitpunkt (Ereignis «Verkauf»). | ☑ | **gilt** | kein Test | – |
| #28/AK3 | Verkauf vor oder im Monat des Kaufs: Fehler am Feld, Start blockiert (Engine und Oberfläche). | ☑ | **gilt** | kein Test | – |
| #28/AK4 | Gleiche Prüfung bei «Besitz» gegenüber dem Kaufdatum. | ☑ | **gilt** | kein Test | – |
| #28/AK5 | Bei Kauf «nach Verkauf von …» keine Fehlermeldung zum Datum. | ☑ | **gilt** | kein Test | – |
| #28/AK6 | Testfall in der Engine für die Prüfung, Oberflächentest für das Feld. | ☑ | **gilt** | kein Test | – |
| #28/AK7 | Hilfe (`app/ui/hilfe/`) und Spezifikation sind nachgeführt. | ☑ | **gilt**: Pfad heute `crates/app/ui/hilfe/` (#48). | kein Test | – |

### #29 «Neu» setzt die Planung nicht zurück

[Issue #29](https://github.com/ingen63/VivaPlan/issues/29) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui-start.spec.js:52](../e2e/ui-start.spec.js#L52), [ui-optimierung.spec.js:331](../e2e/ui-optimierung.spec.js#L331)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #29/AK1 | «Neu» setzt alle Felder auf Standardwerte oder leer. | ☑ | **gilt** | kein Test | – |
| #29/AK2 | Mit ungespeicherten Änderungen erscheint die Rückfrage; «Abbrechen» ändert nichts. | ☑ | **gilt** | kein Test | – |
| #29/AK3 | Der Dateiname im Kopf zeigt «Neue Planung», das Speichern fragt nach einem Pfad. | ☑ | **gilt** | kein Test | – |
| #29/AK4 | Oberflächentest vorhanden. | ☑ | **gilt** | kein Test | – |

### #30 Meilenstein «Verkauf Haus» lässt sich nicht löschen

[Issue #30](https://github.com/ingen63/VivaPlan/issues/30) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui-start.spec.js:334](../e2e/ui-start.spec.js#L334), [ui-start.spec.js:347](../e2e/ui-start.spec.js#L347)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #30/AK1 | `Neu.rplan`: «Verkauf Haus» löschen fragt nach und nennt 2 Eingaben im Abschnitt Immobilien. | ☑ | **gilt**: Die Planung `Neu.rplan` mit «Verkauf Haus» gibt es nicht mehr (heute `new.vplan` ohne diesen Meilenstein); prüfen mit einer erfundenen Planung. | kein Test | – |
| #30/AK2 | Nach «Löschen und umstellen» tragen Verkauf und Kauf das feste Datum 06.2041. | ☑ | **gilt**: Wie AK1: braucht eine erfundene Planung mit dem Meilenstein. | kein Test | – |
| #30/AK3 | Ein Meilenstein ohne Verwendung lässt sich ohne Rückfrage löschen. | ☑ | **gilt** | kein Test | – |

### #31 Pflicht: Grundstückgewinnsteuer bei Immobilien

[Issue #31](https://github.com/ingen63/VivaPlan/issues/31) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #31/AK1 | Immobilie «Besitz» oder «Kauf geplant» ohne Grundstückgewinnsteuer: Fehler im Abschnitt Steuern mit Meldung, Start blockiert (Engine und Oberfläche). | ☑ | **gilt** | [ui-optimierung.spec.js:332](../e2e/ui-optimierung.spec.js#L332) | 15.11.2026 ok (Engine- und Oberflächentest (#29)) |
| #31/AK2 | Nur Mietobjekte oder keine Immobilie: kein Fehler. | ☑ | **gilt** | [ui-optimierung.spec.js:332](../e2e/ui-optimierung.spec.js#L332) | 15.11.2026 ok |
| #31/AK3 | Mit Vorlage oder eigenen Regeln (auch Satz 0 %): kein Fehler. | ☑ | **gilt** | [ui-optimierung.spec.js:332](../e2e/ui-optimierung.spec.js#L332) | 15.11.2026 ok |
| #31/AK4 | Die Pflichtliste (#14) in der Spezifikation und die Hilfe nennen die neue Pflicht. | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: Pflichtliste und Hilfe nicht Satz für Satz verglichen |
| #31/AK5 | Test in der Engine und in der Oberfläche. | ☑ | **gilt** | [ui-optimierung.spec.js:332](../e2e/ui-optimierung.spec.js#L332) | 15.11.2026 ok |

### #32 Immobilien: einheitliche Feldreihenfolge

[Issue #32](https://github.com/ingen63/VivaPlan/issues/32) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui-optimierung.spec.js:360](../e2e/ui-optimierung.spec.js#L360)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #32/AK1 | Reihenfolge und Plätze der Felder sind bei «Besitz» und «Kauf geplant» gleich. | ☑ | **gilt** | [ui-optimierung.spec.js:359](../e2e/ui-optimierung.spec.js#L359) | 15.11.2026 ok (Test grün) |
| #32/AK2 | Der Marktwert steht bei «Besitz» in der Kopfzeile neben dem Namen und fehlt bei «Kauf geplant». | ☑ | **gilt** | [ui-optimierung.spec.js:359](../e2e/ui-optimierung.spec.js#L359) | 15.11.2026 ok (Test grün) |
| #32/AK3 | Beim Statuswechsel bleiben Name, Kaufpreis, Verkauf und Nebenkosten an ihrem Platz. | ☑ | **gilt** | [ui-optimierung.spec.js:359](../e2e/ui-optimierung.spec.js#L359) | 15.11.2026 ok (Test grün) |
| #32/AK4 | Alle Felder bleiben erreichbar, erfasste Werte sind unverändert. | ☑ | **gilt** | [ui-optimierung.spec.js:359](../e2e/ui-optimierung.spec.js#L359) | 15.11.2026 ok (Test grün) |
| #32/AK5 | Schmales Fenster: Marktwert unter dem Namen. | ☑ | **gilt** | [ui-optimierung.spec.js:359](../e2e/ui-optimierung.spec.js#L359) | 15.11.2026 ok (Test grün) |
| #32/AK6 | Oberflächentest prüft die Reihenfolge. | ☑ | **gilt** | [ui-optimierung.spec.js:359](../e2e/ui-optimierung.spec.js#L359) | 15.11.2026 ok (Test grün) |
| #32/AK7 | Hilfe (`app/ui/hilfe/`) ist nachgeführt. | ☑ | **gilt**: Pfad heute `crates/app/ui/hilfe/` (#48). | kein Test | 15.11.2026 teilweise: Hilfe nicht Satz für Satz verglichen |

### #34 Optimum «≥ 100 %» bei Anteilen irreführend

[Issue #34](https://github.com/ingen63/VivaPlan/issues/34) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [optimum.spec.js:50](../e2e/optimum.spec.js#L50), [optimum.spec.js:55](../e2e/optimum.spec.js#L55) (+1)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #34/AK1 | Kapitalbezug Pensionskasse, bester Wert 100 %: Anzeige «100 %», nicht «≥ 100 %». | ☑ | **gilt** | [optimum.spec.js:54](../e2e/optimum.spec.js#L54) | – |
| #34/AK2 | Bester Wert 0 %: Anzeige «0 %», nicht «≤ 0 %». | ☑ | **gilt** | [optimum.spec.js:54](../e2e/optimum.spec.js#L54) | – |
| #34/AK3 | Anteil mit Bereich kleiner als 0 bis 100 % (zum Beispiel 0 bis 50 %), bester Wert 50 %: weiterhin «≥ 50 %». | ☑ | **gilt** | [optimum.spec.js:54](../e2e/optimum.spec.js#L54) | – |
| #34/AK4 | Ausgaben-Faktor am oberen Rand: weiterhin «≥». | ☑ | **gilt** | [optimum.spec.js:54](../e2e/optimum.spec.js#L54) | – |
| #34/AK5 | Die Erklärung unter der Tabelle ist angepasst. | ☑ | **gilt** | kein Test | – |
| #34/AK6 | Test für `Optimum` und die Anzeige. | ☑ | **gilt** | [optimum.spec.js:54](../e2e/optimum.spec.js#L54) | – |

### #35 coverage.yml: Tauri-Abhängigkeiten fehlen auf dem Runner

[Issue #35](https://github.com/ingen63/VivaPlan/issues/35) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #35/AK1 | Ein Lauf von `coverage.yml` auf GitHub ist grün (Engine und App). | ☑ | **gilt** | kein Test | – |
| #35/AK2 | Die Abhängigkeiten stehen an einer Stelle und werden von `coverage.yml` und `pakete.yml` gemeinsam verwendet, oder sind in beiden gleich. | ☑ | **gilt** | kein Test | – |
| #35/AK3 | Sinkt die Abdeckung von Engine oder App unter 80 %, schlägt der Lauf fehl (einmal geprüft). | ☑ | **gilt** | kein Test | – |
| #35/AK4 | Die Laufzeit des Workflows ist im Issue vermerkt. | ☑ | **gilt** | kein Test | – |

### #36 Einheitliche Mindestabdeckung 80 % je Paket

[Issue #36](https://github.com/ingen63/VivaPlan/issues/36) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #36/AK1 | `test-quality.yml` prüft 80 % je Paket statt 92 % über alles. | ☑ | **gilt** | kein Test | – |
| #36/AK2 | Hook, `coverage.yml`, `test-quality.yml`, `CLAUDE.md` und `TESTS.md` nennen dieselbe Schwelle. | ☑ | **gilt** | kein Test | – |
| #36/AK3 | Ein Pull Request, der Hook und `coverage.yml` besteht, besteht auch `test-quality.yml` bei der Abdeckung. | ☑ | **überholt**: durch die Arbeitsweise in `CLAUDE.md`: keine Pull Requests mehr, gearbeitet wird direkt auf `main`; sinngemäss für den Push prüfen. | kein Test | – |
| #36/AK4 | Sinkt ein Paket unter 80 %, schlägt `test-quality.yml` fehl (einmal geprüft). | ☑ | **gilt** | kein Test | – |

### #37 Abnahme-Befunde: Jahreswerte, Meilensteine, Texte

[Issue #37](https://github.com/ingen63/VivaPlan/issues/37) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [ui-start.spec.js:244](../e2e/ui-start.spec.js#L244), [ui-start.spec.js:278](../e2e/ui-start.spec.js#L278) (+2)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #37/AK1 | Spalte «Entnahme» steht nach «Ausgaben total», «Entnahmerate» nach «Einkommen total». | ☑ | **gilt** | kein Test | – |
| #37/AK2 | Der Knopf «Steuern anzeigen/ausblenden» ist sichtbar und blendet die fünf Steuerspalten ein und aus. | ☑ | **gilt** | kein Test | – |
| #37/AK3 | Punkte 1 bis 12 sind umgesetzt oder durch Rückfrage geklärt. | ☑ | **gilt** | kein Test | – |
| #37/AK4 | Hilfe (`app/ui/hilfe/`) und Spezifikation sind nachgeführt, wo sich Texte oder Spalten ändern. | ☑ | **gilt**: Pfad heute `crates/app/ui/hilfe/` (#48). | kein Test | – |
| #37/AK5 | Ein Oberflächentest je Änderung der Tabelle, der Meilensteine und der Warnung. | ☑ | **gilt** | kein Test | – |

### #38 Test-Agent: Mutationstest statt nur Abdeckung als Ziel

[Issue #38](https://github.com/ingen63/VivaPlan/issues/38) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #38/AK1 | Die Anweisung `.claude/agents/rust-test-agent.md` enthält den Mutationslauf, das neue Ziel und das Verbot von Tests ohne Prüfung. | ☑ | **gilt** | kein Test (Datei geprüft: `.claude/agents/rust-test-agent.md`) | 15.11.2026 ok (Mutationslauf und «Keine Tests ohne Prüfung» stehen in der Anweisung) |
| #38/AK2 | Ein Beispiellauf mit einer absichtlich schwach getesteten Änderung im Rechenkern meldet einen missed Mutanten, und der Agent ergänzt einen Test, der ihn tötet. | ☑ | **gilt** | kein Test | nicht geprüft: einmaliger Beispiellauf, nicht wiederholt |
| #38/AK3 | Der Bericht des Agenten listet verbleibende missed Mutanten. | ☑ | **gilt** | kein Test | nicht geprüft: Bericht des Agenten nicht geprüft |
| #38/AK4 | Die Laufzeit des pre-push-Hooks ist gemessen und im Issue vermerkt. | ☑ | **gilt** | kein Test | nicht geprüft: Laufzeit im Issue vermerkt? nicht geprüft |
| #38/AK5 | `TESTS.md` beschreibt das neue Vorgehen. | ☑ | **gilt** | kein Test (Datei geprüft: `docs/TESTS.md`) | 15.11.2026 ok (Mutationstest ist beschrieben) |

### #39 Abdeckung des geänderten Codes im Pull Request anzeigen

[Issue #39](https://github.com/ingen63/VivaPlan/issues/39) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #39/AK1 | Ein Pull Request mit neuem, ungetestetem Code zeigt in der Lauf-Zusammenfassung einen Diff-Abdeckungswert unter 100 % und die nicht abgedeckten Zeilen. | ☑ | **gilt**: Pull Requests werden nicht mehr verwendet (`CLAUDE.md`); sinngemäss für den Push und lokal mit `make coverage-diff` prüfen. | kein Test | nicht geprüft: Pull Requests werden nicht mehr verwendet |
| #39/AK2 | Ein Pull Request ohne Rust-Änderungen zeigt «keine Rust-Änderungen». | ☑ | **gilt**: Wie AK1. | kein Test | nicht geprüft: wie AK1 |
| #39/AK3 | Die Diff-Abdeckung lässt den Lauf nicht fehlschlagen. | ☑ | **gilt** | kein Test (Datei geprüft: `.github/workflows/test-quality.yml`) | 15.11.2026 ok (Schritt mit continue-on-error) |
| #39/AK4 | Der lokale Befehl liefert dasselbe Ergebnis. | ☑ | **gilt** | kein Test (Datei geprüft: `docs/TESTS.md`) | 15.11.2026 teilweise: make coverage-diff vorhanden; Gleichheit mit dem Lauf nicht verglichen |
| #39/AK5 | `TESTS.md` und `CLAUDE.md` nennen die Diff-Abdeckung und dass die 80 % eine Untergrenze sind. | ☑ | **gilt** | kein Test (Datei geprüft: `docs/TESTS.md`) | 15.11.2026 ok (TESTS.md und CLAUDE.md nennen es) |

### #40 Nicht erkannte Mutanten im Rechenkern einzeln beurteilen

[Issue #40](https://github.com/ingen63/VivaPlan/issues/40) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/src/api.rs:605](../crates/engine/src/api.rs#L605), [engine/src/batch.rs:398](../crates/engine/src/batch.rs#L398) (+16)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #40/AK1 | Ein voller Lauf `make mutants` auf `pensionsplaner-engine` ist ausgewertet; jeder missed Mutant ist getestet oder begründet ausgeschlossen. | ☑ | **gilt**: Das Paket heisst seit #44 `vivaplan-engine`. | kein Test | nicht geprüft: voller Lauf nicht wiederholt |
| #40/AK2 | Jeder Ausschluss trägt eine Begründung im Code oder in `.cargo/mutants.toml`. | ☑ | **gilt** | kein Test (Datei geprüft: `.cargo/mutants.toml`) | 15.11.2026 ok (Ausschlüsse mit Begründung) |
| #40/AK3 | Die Liste der offenen missed Mutanten ist dokumentiert (Ort in `TESTS.md` genannt). | ☑ | **gilt** | kein Test (Datei geprüft: `docs/TESTS.md`) | 15.11.2026 ok (offene Mutanten stehen dort) |
| #40/AK4 | `TESTS.md` beschreibt die Regel «einzeln beurteilen, kein Score-Ziel» und die Durchsicht pro Version. | ☑ | **gilt** | kein Test (Datei geprüft: `docs/TESTS.md`) | 15.11.2026 ok (Regel «einzeln beurteilen, kein Score-Ziel») |

### #41 Übersichtsseite Testqualität (Stufe 1: Hygiene und Angemessenheit)

[Issue #41](https://github.com/ingen63/VivaPlan/issues/41) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #41/AK1 | `make report` erzeugt die Seite nach einem Lauf von `make test` und `make coverage`; sie öffnet sich ohne Netzwerk. | ☑ | **gilt** | kein Test | 15.11.2026 ok (`report.py` erzeugt die Seite, sie enthält weder Skripte noch Netzwerkadressen) |
| #41/AK2 | Mit einem absichtlich fehlschlagenden Test zeigt Hygiene «nicht erfüllt» und nennt den Test. | ☑ | **gilt** | [tools/report/test_report.py:29](../tools/report/test_report.py#L29) | 15.11.2026 ok (fehlschlagender Test wird genannt (9 Tests grün)) |
| #41/AK3 | Mit einem vorhandenen `mutants.out/` stehen die missed Mutanten einzeln mit Datei und Zeile auf der Seite. | ☑ | **gilt** | [test_report.py:116](../tools/report/test_report.py#L116) | 15.11.2026 ok (Test) |
| #41/AK4 | Ohne Mutationslauf zeigt der Bereich «keine Daten» statt eines Fehlers. | ☑ | **gilt** | [test_report.py:128](../tools/report/test_report.py#L128) | 15.11.2026 ok (Test) |
| #41/AK5 | Es gibt keine Gesamtnote; jede Schicht hat ihren eigenen Status mit genannter Regel. | ☑ | **gilt** | [tools/report/test_report.py:14](../tools/report/test_report.py#L14) | 15.11.2026 teilweise: Regel je Schicht getestet |
| #41/AK6 | Ein Lauf von `test-quality.yml` enthält die Seite als Artefakt und die Kurzfassung in der Zusammenfassung. | ☑ | **gilt** | kein Test | 15.11.2026 ok (Lauf 43753527025: Artefakt test-quality-reports mit report-html/index.html, Kurzfassung in der Zusammenfassung) |
| #41/AK7 | `TESTS.md` beschreibt die Seite, den Befehl und die Statusregeln. | ☑ | **gilt** | kein Test (Datei geprüft: `docs/TESTS.md`) | 15.11.2026 ok |

### #42 Übersichtsseite Testqualität (Stufe 2: Anforderungsbezug und Ergebnis)

[Issue #42](https://github.com/ingen63/VivaPlan/issues/42) · geschlossen · Umsetzung: nicht vermerkt

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #42/AK1 | Ein Issue mit drei Kriterien, davon zwei mit gekennzeichnetem Test, erscheint mit 2 von 3 belegt. | ☑ | **gilt** | [tools/report/test_report.py:47](../tools/report/test_report.py#L47) | 15.11.2026 ok (2 von 3 belegt) |
| #42/AK2 | Ein Bug-Issue mit Label `found: usage` erscheint unter «Ergebnis» mit Ursache und Regressionstest. | ☑ | **gilt** | [test_report.py:134](../tools/report/test_report.py#L134) | 15.11.2026 ok (Test) |
| #42/AK3 | Der Vergleich zur vorherigen Version zeigt Veränderungen je Kennzahl. | ☑ | **gilt** | [test_report.py:145](../tools/report/test_report.py#L145) | 15.11.2026 ok (Test) |
| #42/AK4 | Ohne `gh`-Anmeldung erscheinen die beiden Schichten als «keine Daten», die übrige Seite bleibt vollständig. | ☑ | **gilt** | [test_report.py:161](../tools/report/test_report.py#L161) | 15.11.2026 ok (Test) |
| #42/AK5 | Labels und Kennzeichnung sind in `CLAUDE.md`, `TESTS.md` und den Issue-Vorlagen beschrieben. | ☑ | **gilt** | kein Test (Datei geprüft: `CLAUDE.md`) | 15.11.2026 teilweise: Labels in CLAUDE.md; TESTS.md und Vorlagen nicht verglichen |

### #43 Mutationstests beschleunigen (Timeout, schwere Tests, Shards)

[Issue #43](https://github.com/ingen63/VivaPlan/issues/43) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/all.rs:1](../crates/engine/tests/all.rs#L1), [engine/tests/all.rs:54](../crates/engine/tests/all.rs#L54)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #43/AK1 | `.cargo/mutants.toml` setzt den Timeout; ein Mutant mit Endlosschleife bricht nach der Höchstdauer ab. | ☑ | **gilt** | kein Test (Datei geprüft: `.cargo/mutants.toml`) | 15.11.2026 teilweise: Timeout gesetzt; Endlosschleife nicht erprobt |
| #43/AK2 | Mutationsläufe verwenden das eigene nextest-Profil ohne die schweren Tests; `make test` führt sie weiterhin aus. | ☑ | **gilt** | kein Test (Datei geprüft: `.config/nextest.toml`) | 15.11.2026 ok (Profil mutants; make test führt alle aus (612)) |
| #43/AK3 | Voller Lauf lokal (`make mutants`): Wanduhrzeit vorher und nachher im Issue, Ziel unter 8 h. | ☑ | **gilt** | kein Test | nicht geprüft: Laufzeit nur auf dem Rechner des Nutzers |
| #43/AK4 | `--in-diff` für eine mittlere Änderung (50–100 Zeilen im Rechenkern) dauert lokal unter 10 min. | ☑ | **gilt** | kein Test | nicht geprüft: Laufzeit nicht gemessen |
| #43/AK5 | `mutation-full.yml` läuft auf GitHub mit allen Shards innerhalb der Zeitgrenze durch. | ☑ | **gilt** | kein Test | nicht geprüft: nur auf GitHub prüfbar |
| #43/AK6 | Vergleich vorher/nachher: Anteil erkannter Mutanten und Liste neu nicht erkannter Mutanten im Issue. | ☑ | **gilt** | kein Test | nicht geprüft: nicht geprüft |
| #43/AK7 | `TESTS.md` beschreibt Timeout, Profil und ausgenommene Tests. | ☑ | **gilt** | kein Test (Datei geprüft: `docs/TESTS.md`) | 15.11.2026 ok |

### #44 App umbenennen: RetirementPlanner → VivaPlan (im bestehenden Repository, GitHub-Repository am Schluss umbenennen)

[Issue #44](https://github.com/ingen63/VivaPlan/issues/44) · geschlossen · Umsetzung: `0b67d93` (altes Repository) · Tests zum Issue ohne Kriterium: [engine/tests/legacy_format.rs:1](../crates/engine/tests/legacy_format.rs#L1), [engine/tests/legacy_format.rs:102](../crates/engine/tests/legacy_format.rs#L102) (+2)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #44/AK1 | Die Suche ohne Beachtung der Gross- und Kleinschreibung nach `retirement`, `rplan` und `pensionsplaner` findet nur noch markierte `legacy`-Stellen und die im Bericht genannten fachlichen Begriffe. | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: Treffer nur noch .rplan-Altformat und Legacy-Stellen (grep) |
| #44/AK2 | Die App startet als «VivaPlan»: Dock, Programmmenü, About-Dialog, Fenstertitel `VivaPlan - ‹Dateiname›`. | ☑ | **gilt** | [app/src/main.rs:1791](../crates/app/src/main.rs#L1791) | 15.11.2026 ok (Test grün) |
| #44/AK3 | Eine `.rplan`-Datei öffnet mit identischen Werten; «Speichern unter» schlägt `.vplan` vor; die `.vplan` öffnet wieder und ergibt dieselben Ergebnisse; die `.rplan` ist unverändert (Prüfsumme). | ☑ | **gilt** | [app/src/main.rs:1731](../crates/app/src/main.rs#L1731), [engine/tests/legacy_format.rs:82](../crates/engine/tests/legacy_format.rs#L82) | 15.11.2026 ok (Test grün) |
| #44/AK4 | Beim ersten Start werden die App-Daten aus dem alten Datenordner übernommen; der alte Ordner existiert danach noch. | ☑ | **gilt** | [app/src/migrate.rs:92](../crates/app/src/migrate.rs#L92) | 15.11.2026 ok (Test grün) |
| #44/AK5 | Ein Doppelklick auf eine `.vplan`-Datei im Finder öffnet VivaPlan. | ☑ | **gilt** | [app/src/main.rs:1803](../crates/app/src/main.rs#L1803) | 15.11.2026 ok (Test grün) |
| #44/AK6 | Alle Tests sind grün (`make test`, Oberflächentests, Rauchtest), Golden-Ergebnisse unverändert, ein Release-Build läuft durch, alle GitHub-Workflows laufen mit den neuen Paketnamen. | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: Tests heute grün (Rust, e2e, Smoke); Release-Build und Workflows nicht geprüft |
| #44/AK7 | Die Git-Historie umbenannter Dateien bleibt erhalten (`git log --follow`). | ☑ | **überholt**: durch #26: Das öffentliche Repository beginnt mit einem einzigen Commit; die Historie liegt nur im alten, privaten Repository. | kein Test | 15.11.2026 überholt, nicht geprüft |
| #44/AK8 | Das GitHub-Repository heisst `VivaPlan`, `origin` zeigt auf die neue Adresse, Push und Workflows funktionieren, im Repository verweist nichts mehr auf `RetirementPlanning` (ausser Weiterleitungs-Hinweis, falls gewünscht). | ☑ | **gilt**: Seit #26 ist `ingen63/VivaPlan` ein neues Repository; das alte heisst wieder `RetirementPlanning` und bleibt privat. | kein Test | nicht geprüft: Repository-Umbenennung nicht geprüft |

### #46 Nächtlicher Fuzz-Job baut für musl und bricht ab

[Issue #46](https://github.com/ingen63/VivaPlan/issues/46) · geschlossen · Umsetzung: `b5c8990` (altes Repository)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #46/AK1 | Der Job `fuzz` übergibt das Ziel-Triple des Hosts ausdrücklich (`--target`), unabhängig davon, welche Fassung von cargo-fuzz installiert ist. | ☑ | **gilt** | kein Test | – |
| #46/AK2 | Im nächsten nächtlichen Lauf (oder einem von Hand gestarteten Lauf) laufen `parse_csv`, `parse_snb` und `validate_plan` je 5 Minuten ohne Baufehler. | ☑ | **gilt** | kein Test | – |
| #46/AK3 | Die Ursache ist im Workflow kommentiert. | ☑ | **gilt** | kein Test | – |

### #48 Projektstruktur vereinfachen (Workspace mit crates/, docs/, tools/)

[Issue #48](https://github.com/ingen63/VivaPlan/issues/48) · geschlossen · Umsetzung: `faba2aa` (altes Repository)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #48/AK1 | `make test` läuft grün, die Golden-Dateien sind unverändert | ☑ | **gilt** | kein Test | 15.11.2026 ok (make test grün; Golden-Dateien seit #82 mit Summen ergänzt (begründet im Issue)) |
| #48/AK2 | Die Abdeckung von Engine und App erreicht weiterhin die Schwelle in `coverage-min`; `make coverage`, `make mutants` und `make report` laufen mit den neuen Pfaden | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: Abdeckung 95.9 % am 14.11.2026; heute nicht neu gemessen |
| #48/AK3 | Alle GitHub-Workflows (Abdeckung, Pakete, Mutationen, Testqualität) laufen mit den neuen Pfaden grün | ☐ | **gilt**: Im Issue nicht abgehakt; im neuen Repository mit `gh run list` prüfbar. | kein Test | nicht geprüft: nur auf GitHub prüfbar |
| #48/AK4 | Die Desktop-App lässt sich bauen und starten (`scripts/build-install.sh` bzw. dessen neuer Pfad, Git-Hook und Rauchtest `ui-smoke.sh` inbegriffen); die Oberflächentests laufen | ☑ | **gilt**: Das Skript heisst heute `tools/build-install.sh`. | kein Test | 15.11.2026 teilweise: Oberflächentests und Rauchtest laufen; App-Build nicht geprüft |
| #48/AK5 | Die Engine enthält ihre Daten und Beispiele selbst; `cargo package -p <engine>` schlägt nicht an fehlenden Dateien fehl | ☑ | **gilt** | kein Test | 15.11.2026 ok (`cargo package -p vivaplan-engine --allow-dirty --offline` baut das Paket aus den verpackten Dateien) |
| #48/AK6 | Im Wurzelverzeichnis liegen keine `mutants.out*`-Ordner mehr, neue Mutationsläufe schreiben nach `target/` oder `local/` | ☑ | **gilt** | kein Test | 15.11.2026 ok (keine mutants.out*-Ordner im Wurzelverzeichnis) |
| #48/AK7 | Hooks, `.claude/`, Makefile, CLAUDE.md, README, `docs/` und Spezifikation verweisen auf die neuen Pfade, keine toten Verweise | ☑ | **gilt** | [docs.test.js:40](../e2e/unit/docs.test.js#L40) | 15.11.2026 ok (Test auf tote Verweise in README, CLAUDE.md und docs/) |
| #48/AK8 | Die Git-Historie der verschobenen Dateien bleibt erhalten (`git mv`) | ☑ | **überholt**: durch #26: keine übernommene Historie im öffentlichen Repository. | kein Test | 15.11.2026 überholt, nicht geprüft |

### #50 Indirekte Amortisation nur bis zum AHV-Alter, danach direkt; 3a-Auszahlung beim AHV-Alter

[Issue #50](https://github.com/ingen63/VivaPlan/issues/50) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/src/validate.rs:1358](../crates/engine/src/validate.rs#L1358), [engine/src/validate.rs:1387](../crates/engine/src/validate.rs#L1387) (+4)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #50/AK1 | Bis zum Monat vor dem AHV-Bezug fliesst die Amortisation einer Tranche mit indirekter Amortisation (`amortMode: indirect`) in das 3a-Konto, wie bisher. | ☑ | **gilt** | [engine/tests/pillar3a.rs:215](../crates/engine/tests/pillar3a.rs#L215) | 15.11.2026 ok (Test grün) |
| #50/AK2 | Ab dem Monat des AHV-Bezugs wird dieselbe Amortisation als direkte Amortisation behandelt: Sie verringert die Hypothek und wird nicht mehr in die Säule 3a eingezahlt. Der Betrag je Quartal bleibt gleich. | ☑ | **gilt** | [engine/tests/pillar3a.rs:215](../crates/engine/tests/pillar3a.rs#L215) | 15.11.2026 ok (Test grün) |
| #50/AK3 | Beispiel mit erfundenen Zahlen: Hypothek 400'000, Tranche mit 750 pro Quartal indirekt, AHV-Bezug im Monat M. Vor M steigt der 3a-Saldo um 750 pro Quartal und die Hypothek bleibt 400'000. Ab M sinkt die Hypothek um 750 pro Quartal, und der 3a-Saldo bleibt unverändert (die Auszahlung des Rests in M ist Sache von #52). | ☑ | **gilt** | [engine/tests/pillar3a.rs:215](../crates/engine/tests/pillar3a.rs#L215) | 15.11.2026 ok (Test grün) |
| #50/AK4 | Eine Tranche oder Periode, die erst ab dem AHV-Alter beginnt (auch eine Erneuerung), kann nicht indirekt amortisieren; sie wird direkt behandelt. Die Eingabemaske schaltet die Auswahl auf «direkt» und nennt den Grund. | ☑ | **gilt** | [engine/tests/pillar3a.rs:243](../crates/engine/tests/pillar3a.rs#L243), [ui-planung.spec.js:52](../e2e/ui-planung.spec.js#L52) | 15.11.2026 ok (Test grün) |
| #50/AK5 | Gibt es keine Person mit AHV-Bezug in der Planung, bleibt das Verhalten unverändert. Das ist in der Hilfe benannt. | ☑ | **gilt** | kein Test | nicht geprüft: nicht im Test |
| #50/AK6 | Regressionstest für den Rechenkern; Golden-Dateien ändern nur mit Begründung im Issue. Hilfetexte (`app/ui/hilfe/`), `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt**: Pfad heute `crates/app/ui/hilfe/` (#48). | kein Test | 15.11.2026 teilweise: Hilfe nicht Satz für Satz verglichen |

### #52 Säule 3a als eigenes Konto mit Auszahlungsplan, Rest spätestens im AHV-Alter

[Issue #52](https://github.com/ingen63/VivaPlan/issues/52) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/src/validate.rs:1358](../crates/engine/src/validate.rs#L1358), [engine/src/validate.rs:1420](../crates/engine/src/validate.rs#L1420) (+12)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #52/AK1 | Die Oberfläche hat einen eigenen Abschnitt «Säule 3a» (nicht unter Immobilien, nicht unter Pensionskasse) mit Guthaben bei Simulationsstart und Verzinsung. | ☑ | **gilt** | [ui-planung.spec.js:13](../e2e/ui-planung.spec.js#L13) | 15.11.2026 ok (Test grün) |
| #52/AK2 | Der Auszahlungsplan ist eine Liste aus Monat/Jahr und Betrag. Zeilen lassen sich hinzufügen und löschen. | ☑ | **gilt** | [ui-planung.spec.js:13](../e2e/ui-planung.spec.js#L13) | 15.11.2026 ok (Test grün) |
| #52/AK3 | Beispiel mit erfundenen Zahlen: Guthaben 120'000, Plan 2041: 50'000 und 2042: 50'000, AHV-Bezug im Jahr 2045. Wird der AHV-Bezug auf 2046 verschoben, bleibt der Plan gültig (frühester Zeitpunkt 2041) und der Rest wird erst 2046 ausbezahlt. Ergebnis: Auszahlungen von 50'000 in 2041 und 2042 und Auszahlung des Rests (20'000 plus Zins) im Monat des AHV-Bezugs. Danach ist das Guthaben 0. | ☑ | **gilt** | [engine/tests/pillar3a.rs:62](../crates/engine/tests/pillar3a.rs#L62) | 15.11.2026 ok (Test grün) |
| #52/AK4 | Ein Plan-Eintrag früher als 5 Jahre vor dem Meilenstein «AHV-Bezug» wird mit klarer Meldung abgelehnt (Validierung im Rechenkern und in der Oberfläche). Genau 5 Jahre vorher ist gültig. | ☑ | **gilt** | [engine/src/validate.rs:1397](../crates/engine/src/validate.rs#L1397) | 15.11.2026 ok (Test grün) |
| #52/AK5 | Ist ein Betrag grösser als das Guthaben zum Zeitpunkt, wird nur das Guthaben ausbezahlt (Hinweis in den Ereignissen). | ☑ | **gilt** | [engine/src/validate.rs:1397](../crates/engine/src/validate.rs#L1397), [engine/tests/pillar3a.rs:102](../crates/engine/tests/pillar3a.rs#L102) | 15.11.2026 ok (Test grün) |
| #52/AK6 | Jede Auszahlung ist ein Kapitalbezug mit Kapitalbezugssteuer; Bezüge im selben Jahr werden mit PK-Kapitalbezügen für die Progression zusammengerechnet (wie bisher). | ☑ | **gilt** | [engine/tests/pillar3a.rs:128](../crates/engine/tests/pillar3a.rs#L128) | 15.11.2026 ok (Test grün) |
| #52/AK7 | Das 3a-Konto hat das Feld «Einzahlung pro Jahr» (Standard 0). Gültig sind Werte von 0 bis zum Maximalbetrag (7'258), beide Grenzen eingeschlossen. Ein negativer Wert oder ein Wert über dem Maximalbetrag wird mit klarer Meldung abgelehnt (Oberfläche und Validierung im Rechenkern). | ☑ | **gilt**: Der Maximalbetrag ist seit #25 einstellbar (Standard 7'258). | [validate.rs:1519](../crates/engine/src/validate.rs#L1519) | 15.11.2026 ok (Test) |
| #52/AK8 | Die Einzahlung erhöht das 3a-Guthaben und zählt zu den Ausgaben: Sie erscheint in «Ausgaben total» der Jahreswerte, erhöht die Entnahme bzw. verringert den Überschuss und wirkt in den Ausgaben der Strategien (3-Topf, Guardrails, Netto-Bedarf). Die Einzahlung verteilt sich auf zwölf gleiche Monatsbeträge. Beispiel mit erfundenen Zahlen: Einzahlung 6'000 pro Jahr, Guthaben 10'000, Zins 0: nach einem Jahr 16'000, die Ausgaben des Jahres sind um 6'000 höher (500 pro Monat im Monatsdurchschnitt). | ☑ | **gilt** | [engine/tests/pillar3a.rs:146](../crates/engine/tests/pillar3a.rs#L146), [ui-planung.spec.js:37](../e2e/ui-planung.spec.js#L37) | 15.11.2026 ok (Test grün) |
| #52/AK9 | Einzahlungen enden spätestens im Monat vor dem Meilenstein «AHV-Bezug». Danach zahlt keine Einzahlung mehr ein (Hinweis in der Eingabe). | ☑ | **gilt** | [engine/tests/pillar3a.rs:173](../crates/engine/tests/pillar3a.rs#L173) | 15.11.2026 ok (Test grün) |
| #52/AK10 | Die Einzahlungen und die indirekte Amortisation (#50) zusammen dürfen im Jahr den Maximalbetrag nicht überschreiten. Die Prüfung meldet eine Überschreitung mit klarer Warnung (keine Sperre). | ☑ | **gilt** | [validate.rs:1519](../crates/engine/src/validate.rs#L1519) | 15.11.2026 ok (Test) |
| #52/AK11 | Der Maximalbetrag ist an einer Stelle gebündelt, damit #25 ihn später aus den Einstellungen liest. | ☑ | **überholt**: durch #25: Der Maximalbetrag kommt aus den Einstellungen. | kein Test | 15.11.2026 überholt, nicht geprüft |
| #52/AK12 | Die Pensionskasse enthält kein 3a-Guthaben mehr. In den Auswertungen bleibt «Säule 3a» getrennt von «Pensionskasse» (heute schon im Diagramm Vorsorge). | ☑ | **gilt** | [pillar3a.rs:350](../crates/engine/tests/pillar3a.rs#L350) | 15.11.2026 ok (Test) |
| #52/AK13 | Alte Planungsdateien (`realestate.properties[].mortgage.pillar3a` mit Guthaben, Zins, `payout`) werden beim Laden in das neue 3a-Konto übernommen: Guthaben und Zins bleiben erhalten, die frühere Auszahlung wird ein einzelner Plan-Eintrag über das ganze Guthaben, auf den frühesten erlaubten Zeitpunkt begrenzt, mit Hinweis beim Laden. Mehrere Immobilien mit je einer 3a werden zu einem Konto zusammengefasst (Hinweis). | ☑ | **gilt** | [engine/src/upgrade.rs:533](../crates/engine/src/upgrade.rs#L533), [engine/src/upgrade.rs:592](../crates/engine/src/upgrade.rs#L592) (+2) | 15.11.2026 ok (Test grün) |
| #52/AK14 | Regressionstest im Rechenkern; Golden-Dateien ändern nur mit Begründung im Issue. Hilfe, `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: Hilfe nicht Satz für Satz verglichen |

### #54 Cash getrennt vom Portfolio, Strategien erst ab Pensionierung, «Liquidität» heisst «Cash»

[Issue #54](https://github.com/ingen63/VivaPlan/issues/54) · geschlossen · Umsetzung: nicht vermerkt · Tests zum Issue ohne Kriterium: [engine/tests/cash_housing.rs:1](../crates/engine/tests/cash_housing.rs#L1), [engine/tests/cash_housing.rs:192](../crates/engine/tests/cash_housing.rs#L192) (+4)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #54/AK1 | Die Aktienquote gilt nur für Aktien und Obligationen; Cash ist nicht Teil davon. Beschriftung und Hilfe sagen das. | ☑ | **gilt** | [ui-planung.spec.js:111](../e2e/ui-planung.spec.js#L111) | 15.11.2026 ok (Test grün) |
| #54/AK2 | Beispiel mit erfundenen Zahlen: Aktien 80'000, Obligationen 20'000, Cash 50'000, Aktienquote 80 %, keine Renditen, Einnahmen gleich Ausgaben: Bis zur Pensionierung bleiben Cash 50'000, Aktien 80'000, Obligationen 20'000 (kein Umschichten von Cash ins Portfolio). | ☑ | **gilt** | [engine/tests/cash_housing.rs:168](../crates/engine/tests/cash_housing.rs#L168) | 15.11.2026 ok (Test grün) |
| #54/AK3 | Beispiel Überschuss: wie oben, aber 2'000 Einkommen mehr als Ausgaben pro Monat. Cash bleibt 50'000, die 2'000 gehen ins Portfolio (1'600 Aktien, 400 Obligationen). | ☑ | **gilt** | [engine/tests/cash_housing.rs:180](../crates/engine/tests/cash_housing.rs#L180) | 15.11.2026 ok (Test grün) |
| #54/AK4 | Vor der Pensionierung gilt keine Entnahmestrategie, auch nicht «Standard»; die Liquiditäts- bzw. Cash-Reserve in Monaten wirkt erst ab der Pensionierung. | ☑ | **gilt** | [engine/tests/cash_housing.rs:214](../crates/engine/tests/cash_housing.rs#L214), [ui-planung.spec.js:111](../e2e/ui-planung.spec.js#L111) | 15.11.2026 ok (Test grün) |
| #54/AK5 | Ab der Pensionierung: «Standard» mit der konfigurierten Cash-Reserve (Monate), alle anderen Strategien wie bisher (3 Monate bzw. 3-Topf mit ihren Töpfen). | ☑ | **gilt** | [engine/tests/cash_housing.rs:214](../crates/engine/tests/cash_housing.rs#L214) | 15.11.2026 ok (Test grün) |
| #54/AK6 | «Liquidität…» heisst überall «Cash…» (Oberfläche, Hilfe, Jahreswerte, Grafiken, Berichte, Prüfmeldungen, Dokumentation). Die Feldnamen in der Planungsdatei bleiben (`wealth.cash`, `reserveMonths`), damit bestehende Dateien gültig bleiben. | ☑ | **gilt** | [docs.test.js:18](../e2e/unit/docs.test.js#L18) | 15.11.2026 ok (Test: «Liquidität» nur noch als Suchbegriff in den Stichwortzeilen der Hilfe) |
| #54/AK7 | Beispiel Nachfüllen und Teuerung: Cash 50'000, Teuerung 2 % pro Jahr, Ausgaben 1'000 über den Einnahmen pro Monat: Cash wird jeden Monat aus dem Portfolio auf den mit der Teuerung fortgeschriebenen Betrag gebracht (nach einem Jahr rund 51'000). | ☑ | **gilt** | [tests/cash_housing.rs:194](../crates/engine/tests/cash_housing.rs#L194) | 15.11.2026 ok (Nachfüllen und Teuerung) |
| #54/AK8 | Ältere Planungen rechnen nach dem neuen Verhalten; beim Öffnen weist ein Hinweis darauf hin, dass Cash vor der Pensionierung neu fest gehalten wird. | ☑ | **gilt** | [engine/tests/cash_housing.rs:143](../crates/engine/tests/cash_housing.rs#L143), [ui-planung.spec.js:121](../e2e/ui-planung.spec.js#L121) | 15.11.2026 ok (Test grün) |
| #54/AK9 | Regressionstest im Rechenkern; Golden-Dateien ändern nur mit Begründung im Issue. Hilfe, `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: Hilfe nicht Satz für Satz verglichen |

### #57 Fehlermeldung am Feld erst beim Verlassen, nicht während der Eingabe

[Issue #57](https://github.com/ingen63/VivaPlan/issues/57) · geschlossen · Umsetzung: `88cf6fe`, `c6d2073` (altes Repository) · Tests zum Issue ohne Kriterium: [ui-start.spec.js:184](../e2e/ui-start.spec.js#L184), [ui-start.spec.js:417](../e2e/ui-start.spec.js#L417) (+7)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #57/AK1 | Solange ein Feld den Fokus hat, erscheint an diesem Feld keine neue Fehlermeldung (weder vom Rechenkern noch zu ungültiger Eingabe wie falschem Datumsformat). Beispiel: Alter 60 → 6 → 65 ohne Meldung. | ☑ | **gilt** | [ui-planung.spec.js:213](../e2e/ui-planung.spec.js#L213), [ui-planung.spec.js:254](../e2e/ui-planung.spec.js#L254) | – |
| #57/AK2 | Beim Verlassen des Feldes (Tab, Klick daneben, Enter) erscheint die Meldung, wenn der Wert ungültig ist. Beispiel: Alter 6 eingeben und das Feld verlassen: Meldung «ausserhalb der Grenzen». | ☑ | **gilt** | [ui-planung.spec.js:213](../e2e/ui-planung.spec.js#L213), [ui-planung.spec.js:254](../e2e/ui-planung.spec.js#L254) | – |
| #57/AK3 | Ist an einem Feld bereits eine Meldung sichtbar und wird der Wert gültig, verschwindet sie sofort, schon während der Eingabe. | ☑ | **gilt**: Neu gefasst per Kommentar: Eine sichtbare Meldung bleibt während der Eingabe stehen und verschwindet erst beim Verlassen. | [ui-planung.spec.js:213](../e2e/ui-planung.spec.js#L213) | – |
| #57/AK4 | Die Vorschau, die Meldungen an anderen Feldern und die Prüfliste rechnen weiter bei jeder Eingabe; nur die Anzeige am aktiven Feld wartet. | ☑ | **gilt**: Neu gefasst per Kommentar: Solange ein Feld den Fokus hat, wartet alles (Vorschau, Meldungen, Prüfliste); Auswahllisten und Kästchen wirken sofort. | [ui-planung.spec.js:213](../e2e/ui-planung.spec.js#L213) | – |
| #57/AK5 | Oberflächentest für die drei Beispiele; Hilfe nachgeführt. | ☑ | **gilt** | kein Test | – |

### #59 Einstellungen auf macOS im Programmmenü

[Issue #59](https://github.com/ingen63/VivaPlan/issues/59) · geschlossen · Umsetzung: `77f560b` (altes Repository)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #59/AK1 | macOS: Das Menü «RetirementPlanner» zeigt «Über RetirementPlanner», Trennlinie, «Einstellungen…» (⌘,), Trennlinie, dann die Systemeinträge und «Beenden». | ☑ | **gilt**: Das Programmmenü heisst seit #44 «VivaPlan». | [app/src/main.rs:1370](../crates/app/src/main.rs#L1370) | 15.11.2026 ok (Test grün) |
| #59/AK2 | macOS: Das Menü «Datei» enthält keinen Eintrag «Einstellungen…». | ☑ | **gilt** | [app/src/main.rs:1370](../crates/app/src/main.rs#L1370) | 15.11.2026 ok (Test grün) |
| #59/AK3 | Windows und Linux: Das Menü «Datei» endet mit «Einstellungen…» (Ctrl+,), Trennlinie, «Beenden». | ☑ | **gilt** | [app/src/main.rs:1370](../crates/app/src/main.rs#L1370) | 15.11.2026 ok (Test grün) |
| #59/AK4 | Auf jedem System gibt es den Eintrag genau einmal; er öffnet den Dialog «Einstellungen» wie bisher. | ☑ | **gilt** | [app/src/main.rs:1370](../crates/app/src/main.rs#L1370) | 15.11.2026 ok (Test grün) |
| #59/AK5 | Hilfe, BESCHREIBUNG.md und Spezifikation nennen den Ort je System. | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: Hilfe und Beschreibung nennen den Ort (grep) |

### #60 Zeitreihen-CSV: Absturz bei Jahreszahl ausserhalb des Bereichs (Fuzz parse_csv)

[Issue #60](https://github.com/ingen63/VivaPlan/issues/60) · geschlossen · Umsetzung: `b900a7e` (altes Repository)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #60/AK1 | Eine jährliche Zeitreihe mit Jahr ausserhalb 1000 bis 9999 (z. B. `99999999999`, `-2147483648`, `2147483647`) ergibt eine Fehlermeldung mit Zeilennummer, keinen Absturz. | ☑ | **gilt** | [engine/tests/data.rs:99](../crates/engine/tests/data.rs#L99) | – |
| #60/AK2 | Die Prüfung auf fortlaufende Daten stürzt auch bei extremen Werten nicht ab. | ☑ | **gilt** | [engine/tests/data.rs:99](../crates/engine/tests/data.rs#L99) | – |
| #60/AK3 | Bestehende Zeitreihen (Monats- und Jahresdaten) lesen sich unverändert; alle Rust-Tests und Golden-Dateien bleiben gleich. | ☑ | **gilt** | kein Test | – |
| #60/AK4 | Regressionstest mit dem Absturzfall des Fuzzers; der Job `fuzz` läuft im nächsten nächtlichen oder von Hand gestarteten Lauf ohne Absturz in `parse_csv`. | ☑ | **gilt** | kein Test | – |

### #61 Haftungsausschluss: Bestätigung beim ersten Start, Fusszeile in den Ergebnissen, vollständiger Text im Info-Bereich

[Issue #61](https://github.com/ingen63/VivaPlan/issues/61) · geschlossen · Umsetzung: `d713431` (altes Repository) · Tests zum Issue ohne Kriterium: [cli/tests/cli.rs:55](../crates/cli/tests/cli.rs#L55), [ui-optimierung.spec.js:101](../e2e/ui-optimierung.spec.js#L101) (+2)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #61/AK1 | Beim ersten Start erscheint die Kurzfassung als Dialog. Weiter geht es erst, wenn das Kästchen «Ich habe den Hinweis gelesen und verstanden» angekreuzt und bestätigt ist; ohne Bestätigung ist die App nicht bedienbar. | ☑ | **gilt** | [ui-optimierung.spec.js:124](../e2e/ui-optimierung.spec.js#L124) | 15.11.2026 ok (Test grün) |
| #61/AK2 | Die Bestätigung wird lokal gespeichert (wie die Einstellungen, nicht in der Planungsdatei); bei späteren Starts erscheint der Dialog nicht mehr. Ändert sich der Text (Versionsnummer des Hinweises), wird erneut gefragt. | ☑ | **gilt** | [ui-optimierung.spec.js:124](../e2e/ui-optimierung.spec.js#L124), [ui-optimierung.spec.js:153](../e2e/ui-optimierung.spec.js#L153) (+1) | 15.11.2026 ok (Test grün) |
| #61/AK3 | Alle Ergebnisseiten (Übersicht, Grafiken, Ereignisse, Jahreswerte, Szenarien, Optimierung) zeigen unten die Fusszeile «Modellrechnung ohne Gewähr. Keine Finanz- oder Steuerberatung.»; auch in Ausdruck bzw. PDF-Export, falls vorhanden. | ☑ | **gilt** | [ui-einstieg.spec.js:98](../e2e/ui-einstieg.spec.js#L98), [ui-einstieg.spec.js:111](../e2e/ui-einstieg.spec.js#L111) | 15.11.2026 ok (Test grün) |
| #61/AK4 | Der vollständige Haftungsausschluss ist jederzeit erreichbar: im Dialog «Über …» (Menü und Hilfe) und als Seite in der Hilfe, mit Stand-Datum. | ☑ | **gilt** | [ui-einstieg.spec.js:124](../e2e/ui-einstieg.spec.js#L124) | 15.11.2026 ok (Test grün) |
| #61/AK5 | Oberflächentests für AK1 bis AK4 (erster Start, gespeicherte Bestätigung, neue Version fragt erneut, Fusszeile, Info-Bereich). Hilfe, `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt**: Kommentar vom 14.11.2026: «AK-5. Nicht erfüllt.» Das Issue ist trotzdem geschlossen und abgehakt; nachprüfen. | kein Test | nicht geprüft: Kommentar im Issue: «AK-5 nicht erfüllt» |

### #63 Einkommens- und Vermögenssteuer nach Kanton und Gemeinde

[Issue #63](https://github.com/ingen63/VivaPlan/issues/63) · offen (AK6 Erwerbstätige) · Umsetzung: 02fcbe5

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #63/AK1 | Im Abschnitt Steuern lässt sich der Modus «Wohnort» oder «Eigene Tabellen» wählen. Im Modus «Wohnort» gibt es für … | ☑ | **gilt** | [ui-steuern.spec.js](../e2e/ui-steuern.spec.js) «AK1» (#63) | 22.11.2026 ok (Test grün) |
| #63/AK2 | Alle 26 Kantone und alle Gemeinden des mitgelieferten Steuerjahres sind wählbar. | ☑ | **gilt** | [tax_location.rs](../crates/engine/tests/tax_location.rs) `issue_63_ak2_…`; [ui-steuern.spec.js](../e2e/ui-steuern.spec.js) «AK2» | 22.11.2026 ok (2110 Gemeinden) |
| #63/AK3 | Steuerbares Einkommen = Einkommen (Erwerb netto + PK-Rente + AHV) − Abzüge, nie negativ. Ohne eigene Abzüge (#131) … | ☑ | **gilt** | [tax_location.rs](../crates/engine/tests/tax_location.rs) `issue_63_ak3_…` (2 Tests) | 22.11.2026 ok (Beispiel Zürich 3000) |
| #63/AK4 | Einkommenssteuer = Kantonskurve(steuerbares Einkommen Kanton) × (Steuerfuss Kanton + Kirche der gewählten … | ☑ | **gilt** | [tax_location.rs](../crates/engine/tests/tax_location.rs) `issue_63_ak4_…` | 22.11.2026 ok |
| #63/AK5 | Bei Verheirateten (`household.married` und zweite Person) gelten die Kurven und Abzüge für Verheiratete. | ☑ | **gilt** | [tax_location.rs](../crates/engine/tests/tax_location.rs) `issue_63_ak5_…` | 22.11.2026 ok |
| #63/AK6 | Für je fünf Referenzfälle (Einkommen und Vermögen, Standardhaushalt ohne Kinder) in mindestens fünf Kantonen weicht … | ☐ | **gilt** | [tax_location.rs](../crates/engine/tests/tax_location.rs) `issue_63_ak6_…` | 22.11.2026 teilweise: Rentner ok (25 Fälle, unter 1 %); Erwerbstätige offen, Referenzwerte der ESTV fehlen |
| #63/AK7 | Bestehende Planungen öffnen im Modus «Eigene Tabellen» und rechnen unverändert; die Golden-Dateien ändern nicht. | ☑ | **gilt** | [tax_location.rs](../crates/engine/tests/tax_location.rs) `issue_63_ak7_…`; [ui-steuern.spec.js](../e2e/ui-steuern.spec.js) «AK7» | 22.11.2026 ok (Golden unverändert) |
| #63/AK8 | Fehlt der Kanton oder passt die Gemeinde nicht zum Kanton im Modus «Wohnort», meldet die Prüfung einen Fehler im … | ☑ | **gilt** | [tax_location.rs](../crates/engine/tests/tax_location.rs) `issue_63_ak8_…` (2 Tests); [ui-steuern.spec.js](../e2e/ui-steuern.spec.js) «AK8» | 22.11.2026 ok |
| #63/AK9 | Die Oberfläche nennt die Steuerfüsse der gewählten Gemeinde, das Steuerjahr und die Quelle der Daten und dass die … | ☑ | **gilt** | [ui-steuern.spec.js](../e2e/ui-steuern.spec.js) «AK9» | 22.11.2026 ok (Test grün) |
| #63/AK10 | Die Übersicht der Annahmen und der Reiter «Parameter» eines Laufs nennen Modus und Wohnort mit dem Gemeindenamen … | ☑ | **gilt** | [ui-steuern.spec.js](../e2e/ui-steuern.spec.js) «AK10» | 22.11.2026 ok (Test grün) |
| #63/AK11 | Ein Werkzeug unter `tools/steuerdaten/` erzeugt die Steuerdatei aus den ESTV-Grunddaten (Steuerfüsse als Excel-Export … | ☑ | **gilt** | [test_build.py](../tools/steuerdaten/test_build.py) (15 Tests) | 22.11.2026 ok (Tests grün, README mit Ablauf) |
| #63/AK12 | Hilfe (`crates/app/ui/hilfe/`), `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test (Durchsicht) | 22.11.2026 ok: Hilfe 10-editor in vier Sprachen, BESCHREIBUNG.md, ENTWICKLUNG.md |

### #66 Steuertarife mit der Teuerung

[Issue #66](https://github.com/ingen63/VivaPlan/issues/66) · offen (AK3 nach #63 belegt, Abhaken offen) · Umsetzung: 5e01d8f

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #66/AK1 | Die Beträge der Tabellen für Einkommen, Vermögen und Kapitalbezug wachsen jedes Jahr mit der kumulierten simulierten … | ☑ | **gilt** | [tax_indexed.rs](../crates/engine/tests/tax_indexed.rs) `issue_66_ak1_…` (3 Tests) | 22.11.2026 ok (Einkommen, Vermögen, Kapitalbezug, Bedarf der 3-Topf-Strategie) |
| #66/AK2 | Beispiel: Tabelle `[[0, 0], [200000, 0.10]]`, Teuerung 2 % pro Jahr. Im Jahr 10 liegt die Stufe bei 243 799 … | ☑ | **gilt** | [tax_indexed.rs](../crates/engine/tests/tax_indexed.rs) `issue_66_ak2_…`, `issue_66_nonpositive_index_…` | 22.11.2026 ok (Satz 5 % statt 6.09 %) |
| #66/AK3 | Die Anpassung gilt im Modus «Eigene Tabellen», auch für die Tabellen der Verheirateten, und im Modus «Wohnort» (#63 … | ☐ | **gilt** | [tax_indexed.rs](../crates/engine/tests/tax_indexed.rs) `issue_66_ak3_…`; [tax_location.rs](../crates/engine/tests/tax_location.rs) `issue_66_ak3_location_…` | 22.11.2026 ok: «Eigene Tabellen», Verheiratete und Modus «Wohnort» (nach #63, 02fcbe5); im Issue noch nicht abgehakt |
| #66/AK4 | Eine Option «Tarife mit der Teuerung» ist für neue Planungen eingeschaltet; bestehende Planungen und der Import alter … | ☑ | **gilt** | [tax_indexed.rs](../crates/engine/tests/tax_indexed.rs) `issue_66_ak4_…` (2 Tests); [ui-planung.spec.js](../e2e/ui-planung.spec.js) «AK4» (#66) | 22.11.2026 ok (Kästchen, «Neu», Import, Übersicht ja/nein) |
| #66/AK5 | Die Grundstückgewinnsteuer bleibt unverändert (ihr Tarif bezieht sich auf den einzelnen Gewinn). | ☑ | **gilt** | [tax_indexed.rs](../crates/engine/tests/tax_indexed.rs) `issue_66_ak5_…` | 22.11.2026 ok |
| #66/AK6 | Hilfe, `BESCHREIBUNG.md` und Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test (Durchsicht) | 22.11.2026 ok: Hilfe 10-editor in vier Sprachen, BESCHREIBUNG.md, ENTWICKLUNG.md |

### #69 Anlagestrategie statt Renditewahl, Gewichtung von CH, EU und USA, Reiter «Renditen & Daten» entfällt

[Issue #69](https://github.com/ingen63/VivaPlan/issues/69) · geschlossen · Umsetzung: `c2b4e38`, `93adc88` (altes Repository) · Tests zum Issue ohne Kriterium: [cli/tests/cli.rs:111](../crates/cli/tests/cli.rs#L111), [engine/tests/strategy.rs:1](../crates/engine/tests/strategy.rs#L1) (+2)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #69/AK1 | Der Reiter «Renditen & Daten» ist weg; die Strategie wird bei den Anlagen gewählt (Aktien und Obligationen getrennt). | ☑ | **gilt** | [ui-start.spec.js:127](../e2e/ui-start.spec.js#L127) | 15.11.2026 ok (Test grün) |
| #69/AK2 | «Welt individuell»: Die Gewichte ergeben immer 100 %; ungültige Eingaben werden angezeigt. Beispiel: EU 40 % auf 20 % halbiert jeden EU-Länderanteil. | ☑ | **gilt** | [engine/tests/strategy.rs:60](../crates/engine/tests/strategy.rs#L60), [engine/tests/strategy.rs:82](../crates/engine/tests/strategy.rs#L82) (+1) | 15.11.2026 ok (Test grün) |
| #69/AK3 | 100 % USA in «Welt individuell» ergibt dieselbe Reihe wie die USA-Reihe der Welt (JST) (Test). | ☑ | **gilt** | [engine/tests/strategy.rs:46](../crates/engine/tests/strategy.rs#L46) | 15.11.2026 ok (Test grün) |
| #69/AK4 | Feste Renditen funktionieren wie bisher; der Import eigener CSV-Dateien ist entfernt, die Hilfe nennt das. | ☑ | **gilt**: Der Import eigener Zeitreihen ist entfernt; das betrifft auch #91. | [engine/tests/strategy.rs:242](../crates/engine/tests/strategy.rs#L242), [engine/tests/strategy.rs:407](../crates/engine/tests/strategy.rs#L407) (+1) | 15.11.2026 ok (Test grün) |
| #69/AK5 | Alte Planungen werden beim Öffnen auf die neue Strategie abgebildet; die Abbildung ist beschrieben und getestet. | ☑ | **gilt** | [engine/tests/strategy.rs:179](../crates/engine/tests/strategy.rs#L179), [engine/tests/strategy.rs:228](../crates/engine/tests/strategy.rs#L228) (+2) | 15.11.2026 ok (Test grün) |
| #69/AK6 | Vor der Umsetzung ist die Frage «CH Pictet bleibt oder nicht» geklärt. | ☑ | **überholt**: durch #90: «Schweiz (Pictet)» ist entfernt. | kein Test | 15.11.2026 überholt, nicht geprüft |
| #69/AK7 | Hilfe, `BESCHREIBUNG.md`, `README.md` und Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: Beschreibung nennt es; README nicht |
| #69/AK8 (Kommentar) | Die Kurzinfo zur Strategie nennt den gemeinsamen Zeitraum, auf den sich die mittleren Renditen beziehen. Wird ein Index gekürzt, weil der andere kürzer ist, erscheint ein Hinweis mit dem Index, seinem eigenen Zeitraum und dem gekürzten Zeitraum, und dass die Simulation (Monte Carlo und historische Verläufe) nur diesen Zeitraum nutzt. Ohne Kürzung kein Hinweis. Test in Rechenkern und Oberfläche. | ☑ | **gilt**: Nachtrag im Kommentar vom 14.11.2026. Das Beispiel dort nutzt Pictet (seit #90 entfernt); mit einer anderen Kombination prüfen. | [engine/tests/strategy.rs:280](../crates/engine/tests/strategy.rs#L280), [engine/tests/strategy.rs:320](../crates/engine/tests/strategy.rs#L320) (+1) | – |

### #70 Willkommensdialog beim ersten Start

[Issue #70](https://github.com/ingen63/VivaPlan/issues/70) · geschlossen · Umsetzung: `7ba159d` (altes Repository) · Tests zum Issue ohne Kriterium: [ui-planung.spec.js:306](../e2e/ui-planung.spec.js#L306), [ui-planung.spec.js:312](../e2e/ui-planung.spec.js#L312)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #70/AK1 | Beim allerersten Start (es gibt noch keine `einstellungen.json`) erscheint der Dialog «Willkommen bei VivaPlan». Er enthält einen Satz zum Zweck der App und drei Knöpfe: «Beispiel erkunden», «Eigene Planung beginnen» und «Leere Planung». | ☑ | **gilt** | [app/src/main.rs:1934](../crates/app/src/main.rs#L1934), [ui-planung.spec.js:313](../e2e/ui-planung.spec.js#L313) | 15.11.2026 ok (Test grün) |
| #70/AK2 | «Beispiel erkunden» lässt die Beispielplanung geladen, wie heute, und öffnet die Hilfe beim Thema «Was VivaPlan kann» (`@start`). | ☑ | **gilt** | [ui-planung.spec.js:327](../e2e/ui-planung.spec.js#L327) | 15.11.2026 ok (Test grün) |
| #70/AK3 | «Eigene Planung beginnen» lädt die Beispielplanung als Vorlage, öffnet den Abschnitt «Haushalt & Projekt» und blendet die Checkliste «Erste Schritte» ein (#71). Solange #71 noch nicht umgesetzt ist, öffnet sich stattdessen die Hilfe beim Schnellstart (`@quickstart`). | ☑ | **gilt**: Nacharbeit laut Kommentar: Die Checkliste wird in den Blick gescrollt und erhält den Fokus. | [ui-planung.spec.js:336](../e2e/ui-planung.spec.js#L336), [helpers.js:45](../e2e/helpers.js#L45) | 15.11.2026 ok (Test grün) |
| #70/AK4 | «Leere Planung» wirkt genau wie Datei → Neu. | ☑ | **gilt** | [ui-planung.spec.js:350](../e2e/ui-planung.spec.js#L350) | 15.11.2026 ok (Test grün) |
| #70/AK5 | Esc oder der Schliessen-Knopf wirken wie «Beispiel erkunden», nur ohne die Hilfe zu öffnen. | ☑ | **gilt** | [ui-planung.spec.js:360](../e2e/ui-planung.spec.js#L360) | 15.11.2026 ok (Test grün) |
| #70/AK6 | Bei späteren Starts erscheint der Dialog nicht mehr, auch nicht, wenn die zuletzt geöffnete Datei fehlt. | ☑ | **gilt** | [app/src/main.rs:1934](../crates/app/src/main.rs#L1934) | 15.11.2026 ok (Test grün) |
| #70/AK7 | Hilfe → «Willkommen …» öffnet den Dialog jederzeit. Bei ungespeicherten Änderungen fragt die App vor dem Laden nach, wie bei Datei → Neu. | ☑ | **gilt** | [app/src/main.rs:1956](../crates/app/src/main.rs#L1956), [ui-planung.spec.js:379](../e2e/ui-planung.spec.js#L379) | 15.11.2026 ok (Test grün) |
| #70/AK8 | Der Dialog lässt sich ganz mit der Tastatur bedienen (Tab, Enter, Esc). | ☑ | **gilt** | [ui-planung.spec.js:360](../e2e/ui-planung.spec.js#L360) | 15.11.2026 ok (Test grün) |
| #70/AK9 | Die Hilfe (`00-einstieg.md`, `@bedienung`), `docs/BESCHREIBUNG.md` und die Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: Hilfe und Beschreibung nennen es (grep) |

### #71 Checkliste «Erste Schritte» im Editor

[Issue #71](https://github.com/ingen63/VivaPlan/issues/71) · geschlossen · Umsetzung: `7ba159d` (altes Repository) · Tests zum Issue ohne Kriterium: [ui-planung.spec.js:306](../e2e/ui-planung.spec.js#L306), [ui-planung.spec.js:394](../e2e/ui-planung.spec.js#L394)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #71/AK1 | Im Editor gibt es einen Kasten «Erste Schritte» mit den acht Schritten des Schnellstarts: Haushalt & Projekt, Meilensteine, Vorsorge, Einnahmen & Ausgaben, Vermögen & Anlage, Renditen & Daten, Steuern, Simulation starten. | ☑ | **gilt**: Der sechste Schritt heisst seit #69 «Simulation» statt «Renditen & Daten» (Kommentar im Issue). | [ui-planung.spec.js:395](../e2e/ui-planung.spec.js#L395) | 15.11.2026 ok (Test grün) |
| #71/AK2 | Ein Klick auf einen Schritt öffnet dessen Abschnitt. Das Zeichen ⓘ öffnet die Hilfe zum Schritt. | ☑ | **gilt** | [ui-planung.spec.js:404](../e2e/ui-planung.spec.js#L404) | 15.11.2026 ok (Test grün) |
| #71/AK3 | Ein Abschnitt-Schritt gilt als erledigt, sobald darin mindestens ein Wert geändert wurde oder der Benutzer ihn von Hand abhakt. «Simulation starten» ist erledigt, sobald eine Simulation ohne Abbruch durchgelaufen ist. | ☑ | **gilt** | [ui-planung.spec.js:413](../e2e/ui-planung.spec.js#L413), [ui-planung.spec.js:431](../e2e/ui-planung.spec.js#L431) | 15.11.2026 ok (Test grün) |
| #71/AK4 | Ein Haken bleibt, auch wenn die Änderung mit ⌘Z rückgängig gemacht wird. | ☑ | **gilt** | [ui-planung.spec.js:413](../e2e/ui-planung.spec.js#L413) | 15.11.2026 ok (Test grün) |
| #71/AK5 | Der Fortschritt steht im Kopf des Kastens, zum Beispiel «3 von 8». | ☑ | **gilt** | [ui-planung.spec.js:395](../e2e/ui-planung.spec.js#L395) | 15.11.2026 ok (Test grün) |
| #71/AK6 | Mit × lässt sich der Kasten ausblenden, über Hilfe → «Erste Schritte» wieder einblenden. | ☑ | **gilt** | [app/src/main.rs:1956](../crates/app/src/main.rs#L1956), [ui-planung.spec.js:438](../e2e/ui-planung.spec.js#L438) | 15.11.2026 ok (Test grün) |
| #71/AK7 | Sind alle acht Schritte erledigt, erscheint ein kurzer Hinweis. Danach klappt der Kasten zu. | ☑ | **gilt** | [ui-planung.spec.js:438](../e2e/ui-planung.spec.js#L438) | 15.11.2026 ok (Test grün) |
| #71/AK8 | Wählt der Benutzer im Willkommensdialog (#70) «Eigene Planung beginnen», wird die Checkliste eingeblendet und zurückgesetzt, auch wenn er sie vorher ausgeblendet hatte. Alle Schritte stehen dann auf offen. | ☑ | **gilt** | [ui-einstieg.spec.js:16](../e2e/ui-einstieg.spec.js#L16) | 15.11.2026 ok (Test grün) |
| #71/AK9 | Der Fortschritt wird in `einstellungen.json` gespeichert und gilt für den Benutzer, nicht für eine einzelne Planung. Das `.vplan`-Format bleibt unverändert. | ☑ | **gilt** | [ui-planung.spec.js:413](../e2e/ui-planung.spec.js#L413) | 15.11.2026 ok (Test grün) |
| #71/AK10 | Die Hilfe, `docs/BESCHREIBUNG.md` und die Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: Hilfe und Beschreibung nennen es (grep) |

### #72 Ergebnis in Worten in der Übersicht

[Issue #72](https://github.com/ingen63/VivaPlan/issues/72) · geschlossen · Umsetzung: `7ba159d` (altes Repository) · Tests zum Issue ohne Kriterium: [ui-planung.spec.js:306](../e2e/ui-planung.spec.js#L306), [ui-einstieg.spec.js:28](../e2e/ui-einstieg.spec.js#L28)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #72/AK1 | **Mehrere Läufe:** Die Einordnung steht in Klammern hinter der Zahl, zum Beispiel «In 87 % (mittel) der Läufe reicht das Vermögen bis 12.2055 (Alter 95).» Die Einordnung ist ein Link auf die Hilfe «Wie viel Erfolgsquote ist genug?» (`@r-overview`). Die Grenzen sind dieselben wie bei der Ampel: «hoch» ab 95 %, «mittel» von 80 % bis unter 95 %, «tief» unter 80 %. | ☑ | **gilt** | [ui-einstieg.spec.js:29](../e2e/ui-einstieg.spec.js#L29) | 15.11.2026 ok (Test grün) |
| #72/AK2 | An den Grenzen gilt 95,0 % als «hoch» und 80,0 % als «mittel». Massgebend ist derselbe gerundete Wert, den die Kennzahl «Erfolgsquote» anzeigt. | ☑ | **gilt** | [ui-einstieg.spec.js:45](../e2e/ui-einstieg.spec.js#L45) | 15.11.2026 ok (Test grün) |
| #72/AK3 | Geht in mindestens einem Lauf das Geld aus, folgt der Satz «Im ungünstigsten Lauf ist das Geld mit Alter … aufgebraucht.» Er nennt dasselbe Alter wie die Kennzahl «Früheste Erschöpfung». | ☑ | **gilt** | [ui-einstieg.spec.js:29](../e2e/ui-einstieg.spec.js#L29) | 15.11.2026 ok (Test grün) |
| #72/AK4 | **Ein einzelner Lauf:** «Das Vermögen reicht bis zum Ende der Planung und beträgt dann CHF ….» oder «Das Vermögen ist im 03.2049 (Alter 84) aufgebraucht.» | ☑ | **gilt** | [ui-einstieg.spec.js:52](../e2e/ui-einstieg.spec.js#L52) | 15.11.2026 ok (Test grün) |
| #72/AK5 | Beträge im Text folgen dem Umschalter «Nominal» bzw. «Heutige Kaufkraft». | ☑ | **gilt** | [ui-einstieg.spec.js:52](../e2e/ui-einstieg.spec.js#L52) | 15.11.2026 ok (Test grün) |
| #72/AK6 | Ein Hinweis auf den nächsten Schritt mit Link in die Hilfe: Liegt die Erfolgsquote unter 95 %, verweist er auf die Optimierung. Sonst verweist er auf die Szenarien. | ☑ | **gilt** | [ui-einstieg.spec.js:29](../e2e/ui-einstieg.spec.js#L29), [ui-einstieg.spec.js:71](../e2e/ui-einstieg.spec.js#L71) | 15.11.2026 ok (Test grün) |
| #72/AK7 | Ergebnisse aus der Liste der Simulationen zeigen den Text ebenfalls. | ☑ | **gilt** | [ui-einstieg.spec.js:52](../e2e/ui-einstieg.spec.js#L52) | 15.11.2026 ok (Test grün) |
| #72/AK8 | Der Text entsteht in der Oberfläche aus den vorhandenen Kennzahlen. Der Rechenkern und die Golden-Dateien bleiben unverändert. | ☑ | **gilt** | kein Test | 15.11.2026 vom Anwender geschlossen: nicht mehr belegbar (das öffentliche Repository hat nur einen Commit; die Golden-Tests laufen grün) |
| #72/AK9 | Die Hilfe (`@r-overview`), `docs/BESCHREIBUNG.md` und die Spezifikation sind nachgeführt. | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: Hilfe und Beschreibung nennen es (grep) |

### #74 Hinweis zur Cash-Reserve nur einmal, beim ersten Laden einer betroffenen Planung

[Issue #74](https://github.com/ingen63/VivaPlan/issues/74) · geschlossen · Umsetzung: `b73b0d8` (altes Repository)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #74/AK1 | Das erste Laden einer betroffenen Planung nach dem Update (beim Start, über «Öffnen» oder per Doppelklick) zeigt den Hinweis einmal. | ☑ | **gilt** | [ui-optimierung.spec.js:173](../e2e/ui-optimierung.spec.js#L173) | 15.11.2026 ok (Test grün) |
| #74/AK2 | Jeder weitere Start mit derselben oder einer anderen betroffenen Planung: Kein Hinweis. | ☑ | **gilt** | [ui-optimierung.spec.js:181](../e2e/ui-optimierung.spec.js#L181) | 15.11.2026 ok (Test grün) |
| #74/AK3 | Nachdem der Hinweis einmal erschienen ist, erscheint er auch beim Öffnen einer betroffenen Planung über «Öffnen» oder per Doppelklick nicht mehr. | ☑ | **gilt** | [ui-optimierung.spec.js:83](../e2e/ui-optimierung.spec.js#L83) | 15.11.2026 ok (Test grün) |
| #74/AK4 | Nicht betroffene Planungen (z. B. Strategie «Standard») zeigen keinen Hinweis und zählen nicht als «gezeigt»: Beispiel: erster Start mit einer Planung «Standard» ohne Hinweis, danach Öffnen einer betroffenen Planung zeigt den Hinweis einmal. | ☑ | **gilt** | [ui-optimierung.spec.js:188](../e2e/ui-optimierung.spec.js#L188) | 15.11.2026 ok (Test grün) |
| #74/AK5 | Dass der Hinweis gezeigt wurde, merkt sich die App in den lokalen Einstellungen (nicht in der Planungsdatei); die Planungsdatei bleibt unverändert. | ☑ | **gilt** | [ui-optimierung.spec.js:83](../e2e/ui-optimierung.spec.js#L83), [ui-optimierung.spec.js:173](../e2e/ui-optimierung.spec.js#L173) | 15.11.2026 ok (Test grün) |
| #74/AK6 | Die Rechenergebnisse ändern sich nicht (der Hinweis ist nur eine Information); Golden-Dateien unverändert. | ☑ | **gilt** | kein Test | 15.11.2026 vom Anwender geschlossen: nicht mehr belegbar (das öffentliche Repository hat nur einen Commit; die Golden-Tests laufen grün) |
| #74/AK7 | Hilfe und Spezifikation (#18) beschreiben das neue Verhalten. | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: Hilfe nennt es (grep) |

### #75 Rauchtest: Prüfung der Jahreswerte erwartet veraltete Steuerspalten

[Issue #75](https://github.com/ingen63/VivaPlan/issues/75) · geschlossen · Umsetzung: `96526b0` (altes Repository)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #75/AK1 | `tools/ui-smoke.sh` meldet für `#results/bootstrap/table` «ok», wenn die Jahreswerte vollständig sind. | ☑ | **gilt** | kein Test | – |
| #75/AK2 | Die Prüfung erkennt weiterhin, wenn eine geforderte Spalte fehlt (z. B. «Ausgaben total»). | ☑ | **gilt** | kein Test | – |
| #75/AK3 | Die Steuerspalten werden mit den heutigen Namen geprüft, bei eingeblendeten Steuerspalten (oder die Prüfung prüft bewusst den eingeklappten Standard und die Begründung steht im Skript). | ☑ | **gilt** | kein Test | – |

### #76 Obligationen: kurzfristige Eidgenossen in der Anlagestrategie

[Issue #76](https://github.com/ingen63/VivaPlan/issues/76) · geschlossen · Umsetzung: `9c44012` (altes Repository)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #76/AK1 | Bei den Obligationen gibt es die Wahl «CH kurzfristig» neben CH, CH Pictet und USA. Hilfe und Info-Fenster nennen Quelle, Zeitraum und dass die Rendite praktisch dem Geldmarktsatz entspricht. | ☑ | **gilt**: «CH Pictet» gibt es seit #90 nicht mehr. | [engine/tests/strategy.rs:369](../crates/engine/tests/strategy.rs#L369), [ui-start.spec.js:138](../e2e/ui-start.spec.js#L138) | 15.11.2026 ok (Test grün) |
| #76/AK2 | Die Jahresrenditen stammen bis 2020 aus der JST-Datenbank (`bill_rate`, Schweiz, lückenlos ab 1870, verwendet ab 1900) und danach aus einer Reihe der SNB. Der Übergang hat keine Lücke; ein Test rechnet ein Jahr vor und ein Jahr nach dem Übergang von Hand nach. | ☑ | **gilt** | [engine/tests/strategy.rs:343](../crates/engine/tests/strategy.rs#L343) | 15.11.2026 ok (Test grün) |
| #76/AK3 | Eine Strategie mit «CH kurzfristig» rechnet jährlich über die gemeinsamen Jahre mit der gewählten Aktienreihe, wie die anderen gemischten Kombinationen (#69). | ☑ | **gilt** | [engine/tests/strategy.rs:369](../crates/engine/tests/strategy.rs#L369), [ui-start.spec.js:138](../e2e/ui-start.spec.js#L138) | 15.11.2026 ok (Test grün) |
| #76/AK4 | Bestehende Planungen rechnen unverändert (Golden-Dateien ohne Änderung). | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: Golden-Dateien grün |
| #76/AK5 | QUELLEN.md nennt Quelle, Abrufdatum und Lizenz der neuen Daten. | ☑ | **gilt** | kein Test (Datei geprüft: `crates/engine/data/QUELLEN.md`) | 15.11.2026 teilweise: Quellen genannt; Abrufdatum für SARON nicht gefunden |

### #77 Konzept: VivaPlan in Claude nutzen (MCP)

[Issue #77](https://github.com/ingen63/VivaPlan/issues/77) · geschlossen · Umsetzung: `5f3475f` (altes Repository)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #77/AK1 | Das Konzept liegt als `docs/spezifikation/VivaPlan in Claude (MCP).md` vor. | ☑ | **gilt** | kein Test (Datei geprüft: `docs/VivaPlan in Claude (MCP).md`) | 15.11.2026 ok (Datei liegt seit #99 in docs/) |
| #77/AK2 | Es beschreibt mindestens zwei Varianten und vergleicht sie, mindestens: (a) lokaler MCP-Server (stdio) auf Basis von `vivaplan-engine`/`vivaplan-cli`, der Werkzeuge bereitstellt, und (b) die Oberfläche als interaktive Ansicht in Claude (MCP Apps / UI-Ressourcen). Es enthält eine Empfehlung mit Begründung. | ☑ | **gilt** | kein Test (Datei geprüft: `docs/VivaPlan in Claude (MCP).md`) | 15.11.2026 ok (Varianten (a), (b), (c) und Vergleich) |
| #77/AK3 | Es listet die Werkzeuge (Tools) mit Name, Zweck, Eingaben und Ausgaben, mindestens: Planung öffnen, prüfen, Werte ändern, Vorschau, Simulation, Optimierung, Ergebnisse abrufen und vergleichen, Planung speichern. | ☑ | **gilt** | kein Test (Datei geprüft: `docs/VivaPlan in Claude (MCP).md`) | 15.11.2026 ok (Abschnitt «Werkzeuge») |
| #77/AK4 | Es legt fest, welche Daten an Claude gehen, und wie das zur Regel «Alle Daten bleiben lokal» passt (die App selbst bleibt ohne Netzwerkzugriff; der Benutzer entscheidet bewusst, Daten an Claude zu geben). Dazu Hinweise für den Benutzer in der Hilfe. | ☑ | **gilt** | kein Test (Datei geprüft: `docs/VivaPlan in Claude (MCP).md`) | 15.11.2026 ok (Abschnitt «Daten und Datenschutz») |
| #77/AK5 | Es beschreibt, wie Änderungen durch Claude sichtbar und rückgängig zu machen sind und dass Planungsdateien nie ohne Bestätigung überschrieben werden. | ☑ | **gilt** | kein Test (Datei geprüft: `docs/VivaPlan in Claude (MCP).md`) | 15.11.2026 ok (Abschnitt «Änderungen sichtbar machen …») |
| #77/AK6 | Es beschreibt Grenzen und Risiken (lange Rechenzeiten und Abbruch, Grösse der Ergebnisse, Fehlbedienung, Haftungsausschluss #61). | ☑ | **gilt** | kein Test (Datei geprüft: `docs/VivaPlan in Claude (MCP).md`) | 15.11.2026 ok (Abschnitt «Grenzen und Risiken») |
| #77/AK7 | Es beschreibt Installation und Einrichtung in Claude Desktop und Claude Code, die Tests (wie der MCP-Server automatisch geprüft wird) und eine Aufteilung der Umsetzung in Folge-Issues mit Abnahmekriterien. | ☑ | **gilt** | kein Test (Datei geprüft: `docs/VivaPlan in Claude (MCP).md`) | 15.11.2026 ok (Abschnitte Installation, Tests, Folge-Issues) |
| #77/AK8 | Rechenkern und Oberfläche bleiben getrennt: Das Konzept nutzt die bestehende Schnittstelle (`api`) des Rechenkerns, ohne Berechnungen doppelt zu führen. | ☐ | **gilt** | kein Test (Datei geprüft: `docs/VivaPlan in Claude (MCP).md`) | 15.11.2026 teilweise: Abschnitt «Aufbau» nur nach Überschrift beurteilt |

### #78 Testdateien MSCI.csv und beispiel.rplan durch erfundene Daten ersetzen

[Issue #78](https://github.com/ingen63/VivaPlan/issues/78) · geschlossen · Umsetzung: `590a43d` (altes Repository)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #78/AK1 | `crates/engine/tests/fixtures/MSCI.csv` und `crates/engine/tests/fixtures/beispiel.rplan` sind gelöscht; `git ls-files` findet keine Datei mehr, deren Pfad `MSCI` oder `beispiel.rplan` enthält. | ☑ | **gilt** | kein Test | 15.11.2026 ok (git ls-files findet MSCI und beispiel.rplan nicht) |
| #78/AK2 | Eine erfundene Renditereihe (neutraler Dateiname ohne «MSCI», gleiches CSV-Format) ersetzt `MSCI.csv` in allen oben genannten Tests; die Tests prüfen weiterhin dasselbe Verhalten (Laden, historische Rendite, `series_info` relativ zur Planung). | ☑ | **gilt** | kein Test (Datei geprüft: `crates/engine/tests/fixtures/renditen-muster.csv`) | 15.11.2026 ok (erfundene Reihe vorhanden) |
| #78/AK3 | Eine erfundene Planung im alten Format `.rplan` (neutraler Name, keine echten Namen oder Beträge) ersetzt `beispiel.rplan` in `legacy_format.rs`; das Lesen des alten Formats bleibt abgedeckt. | ☑ | **gilt**: Die erfundene Planung heisst `crates/engine/tests/fixtures/planung-altformat.rplan`. | kein Test (Datei geprüft: `crates/engine/tests/fixtures/planung-altformat.rplan`) | 15.11.2026 ok |
| #78/AK4 | `make test` läuft grün, die Abdeckung von Engine und App bleibt je mindestens 80 %. | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: make test grün; Abdeckung am 14.11.2026 95.9 % |
| #78/AK5 | Die Golden-Dateien (`crates/engine/tests/golden/`) bleiben unverändert. | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: Golden-Dateien seit #82 ergänzt, begründet im Issue |

### #79 Oberflächentests schlagen bei vollem Lauf fehl

[Issue #79](https://github.com/ingen63/VivaPlan/issues/79) · geschlossen · Umsetzung: `62333d7` (altes Repository) · Tests zum Issue ohne Kriterium: [ui-cancel.spec.js:27](../e2e/ui-cancel.spec.js#L27), [helpers.js:26](../e2e/helpers.js#L26) (+3)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #79/AK1 | Die Ursache ist im Issue beschrieben. | ☑ | **gilt** | kein Test | – |
| #79/AK2 | Drei volle Läufe nacheinander (`npx playwright test`, frischer Server) sind grün. | ☑ | **gilt** | kein Test | – |
| #79/AK3 | Welcher Test hätte ihn fangen müssen / Regressionstest: im Issue beantwortet. | ☑ | **gilt** | kein Test | – |

### #80 CI: Tests und Abdeckung nur einmal je Push

[Issue #80](https://github.com/ingen63/VivaPlan/issues/80) · offen · Umsetzung: siehe Kommentar im Issue

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #80/AK1 | Ein Push auf `main` startet genau einen Lauf der Rust-Tests mit Abdeckung. | ☑ | **gilt** | kein Test (Datei geprüft: `.github/workflows/test-quality.yml`) | 15.11.2026 ok (Datei) |
| #80/AK2 | Liegt Engine oder App unter `tools/coverage-min`, schlägt der Lauf fehl (Beispiel: Schwelle vorübergehend 99). | ☑ | **gilt** | kein Test | 15.11.2026 ok (Schwelle 99: alle drei Pakete Exit 1) |
| #80/AK3 | Doku (`docs/TESTS.md`, Kommentare in den Workflows) nennt nur noch den verbleibenden Workflow. | ☑ | **gilt** | kein Test (Datei geprüft: `docs/TESTS.md`) | 15.11.2026 ok (Durchsicht) |

### #85 Zufällige Planungen über alle Renditequellen und Entnahmestrategien, mit Verkleinerung

[Issue #85](https://github.com/ingen63/VivaPlan/issues/85) · offen · Umsetzung: siehe Kommentar im Issue

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #85/AK1 | Der Generator ist eine proptest-Strategie; ein Fehlschlag wird verkleinert und als Regression gespeichert. | ☑ | **gilt** | [proptest_plans.rs](../crates/engine/tests/proptest_plans.rs) | 15.11.2026 ok (Test grün) |
| #85/AK2 | Erzeugt werden alle Renditequellen und alle sechs Strategien sowie Planungen mit Partner. | ☑ | **gilt** | [proptest_plans.rs](../crates/engine/tests/proptest_plans.rs) | 15.11.2026 ok (Test grün) |
| #85/AK3 | Laufzeit im Standard unter 5 s; Monte-Carlo-Fälle mit wenigen Pfaden (ca. 20); schwere Fälle nicht im Profil `mutants`. | ☐ | **gilt** | [proptest_plans.rs](../crates/engine/tests/proptest_plans.rs) | 15.11.2026 teilweise: Hauptlauf 1.8 s, Monte-Carlo-Lauf 11.9 s; nachverfolgt in #102 |

### #86 Fuzz-Ziel für die ganze Rechnung

[Issue #86](https://github.com/ingen63/VivaPlan/issues/86) · offen · Umsetzung: siehe Kommentar im Issue

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #86/AK1 | Neues Ziel verändert eine gültige Planung strukturiert und ruft `api::run` auf. | ☑ | **gilt** | kein Test (Fuzz-Ziel `fuzz/fuzz_targets/run_plan.rs`) | 15.11.2026 ok (clippy sauber) |
| #86/AK2 | Ein Startkorpus liegt im Repo; der nächtliche Lauf speichert den Korpus zwischen (Cache). | ☐ | **gilt** | kein Test | 15.11.2026 teilweise: Korpus und Cache in der Datei; GitHub nicht gelaufen |
| #86/AK3 | Ein Fund lässt den nächtlichen Lauf fehlschlagen und wird als Artefakt abgelegt (wie bisher). | ☐ | **gilt** | kein Test | 15.11.2026 teilweise: Konfiguration geprüft (Fuzz-Schritt schlägt fehl, Artefakt bei failure()); kein echter Fund erprobt |

### #88 clippy im Workflow

[Issue #88](https://github.com/ingen63/VivaPlan/issues/88) · offen · Umsetzung: siehe Kommentar im Issue

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #88/AK1 | `cargo clippy --workspace --all-targets -- -D warnings` läuft bei jedem Push und lässt ihn bei Warnungen fehlschlagen. | ☑ | **gilt** | kein Test (Datei geprüft: `test-quality.yml`) | 15.11.2026 ok (clippy-Job rot auf 7280069, grün auf 6bf8b21 und fb13872) |
| #88/AK2 | Bestehende Warnungen sind behoben oder begründet erlaubt. | ☑ | **gilt** | kein Test | 15.11.2026 ok (clippy sauber) |

### #89 Oberflächentests nächtlich auf GitHub

[Issue #89](https://github.com/ingen63/VivaPlan/issues/89) · offen · Umsetzung: siehe Kommentar im Issue

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #89/AK1 | Ein nächtlicher Job führt `e2e/` aus (nur bei neuem Commit, wie die übrigen nächtlichen Jobs). | ☐ | **gilt** | kein Test (Datei geprüft: `test-quality.yml`) | 18.11.2026 offen: Job lief grün im [Lauf 44375155792](https://github.com/ingen63/VivaPlan/actions/runs/44375155792) (`nightly=true`), schedule-Lauf mit e2e fehlt; früher: 16.11.2026 teilweise: drei Läufe von Hand grün (43850404495, 43851543289, 43852527223, nach #111; beim Aufräumen der Runs gelöscht, Ergebnis im Issue); zweite Prüfung 16.11.2026: noch kein Nachtlauf mit e2e; 17.11.2026 offen: GitHub hat am 16.11. und 17.11. keinen Nachtlauf gestartet (letzter schedule-Lauf 15.11., vor e2e) |
| #89/AK2 | Bei Fehlschlag sind Bericht und Traces als Artefakt abgelegt. | ☑ | **gilt** | kein Test (Datei geprüft: `test-quality.yml`) | 16.11.2026 ok: Artefakt e2e-failure mit Bericht und Traces vorhanden (Lauf 43746959803) |
| #89/AK3 | `e2e/results.json` fliesst in die Übersichtsseite ein. | ☐ | **gilt** | [test_report.py](../tools/report/test_report.py) | 18.11.2026 offen: Übersichtsseite des Laufs zeigt «keine Daten» (Änderung 26f98d1 noch nicht gepusht); früher: 16.11.2026 teilweise: Bericht liest die Datei, Artefakt e2e-results liegt vor; die Übersicht auf GitHub holt es nur aus Nachtläufen, noch keiner mit e2e (zweite Prüfung 16.11.2026 unverändert); 17.11.2026 offen: test_report.py lokal grün (18 Tests), weiterhin kein Nachtlauf mit e2e |

### #90 Pictet-Reihe aus App und Repository entfernen

[Issue #90](https://github.com/ingen63/VivaPlan/issues/90) · geschlossen · Umsetzung: `6f1ab9e` (altes Repository) · Tests zum Issue ohne Kriterium: [ui-start.spec.js:160](../e2e/ui-start.spec.js#L160)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #90/AK1 | `crates/engine/data/Pictet CH.csv` ist gelöscht; `git ls-files` findet keine Datei, deren Pfad `Pictet` enthält, und kein `include_str!` verweist mehr darauf. | ☑ | **gilt** | [engine/tests/library.rs:140](../crates/engine/tests/library.rs#L140) | 15.11.2026 ok (Test grün) |
| #90/AK2 | Die Anlagestrategie bietet «Schweiz (Pictet)» weder bei den Aktien noch bei den Obligationen an (Auswahl in der Oberfläche, Strategieangaben des Rechenkerns). | ☑ | **gilt** | [ui-start.spec.js:218](../e2e/ui-start.spec.js#L218) | 15.11.2026 ok (Test grün) |
| #90/AK3 | Eine Planung mit `"strategy": {"stocks": "chPictet", …}` öffnet ohne Fehler und erhält für die Aktien «Schweiz (JST)» (`chJst`); eine Planung mit `"bonds": "chPictet"` erhält «Eidgenossenschaft 10 Jahre» (`ch`). Ein Hinweis beim Öffnen nennt die Umstellung. | ☑ | **gilt** | [engine/tests/strategy.rs:475](../crates/engine/tests/strategy.rs#L475), [ui-start.spec.js:218](../e2e/ui-start.spec.js#L218) | 15.11.2026 ok (Test grün) |
| #90/AK4 | Eine ältere Planung mit `"series": "library:Pictet CH.csv"` öffnet ohne Fehler, erhält `chJst` + `ch` und denselben Hinweis. | ☑ | **gilt** | [engine/tests/strategy.rs:185](../crates/engine/tests/strategy.rs#L185), [ui-start.spec.js:209](../e2e/ui-start.spec.js#L209) | 15.11.2026 ok (Test grün) |
| #90/AK5 | Die App legt `Pictet CH.csv` nicht mehr im Datenordner an. Eine dort schon vorhandene Kopie wird weder gelöscht noch verändert und nicht mehr als mitgelieferte Reihe angeboten. | ☑ | **gilt** | [engine/tests/library.rs:140](../crates/engine/tests/library.rs#L140) | 15.11.2026 ok (Test grün) |
| #90/AK6 | Hilfe (`20-renditen.md`, `10-editor.md`, `60-referenz.md`), `README.md`, `docs/BESCHREIBUNG.md`, `crates/engine/data/QUELLEN.md` und die Spezifikation #69 sind nachgeführt. Die Erwähnung von Pictet als Quelle der Schweizer JST-Aktienreihe darf bleiben. | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: «Pictet» steht noch als Stichwort und als Quellenhinweis |
| #90/AK7 | `make test` und die Oberflächentests (`e2e/`) laufen grün; die Abdeckung von Engine und App bleibt je mindestens 80 %; die Golden-Dateien (`crates/engine/tests/golden/`) bleiben unverändert. | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: alle Tests heute grün; Abdeckung am 14.11.2026 |

### #92 Test für eigene Konfigurationen ohne feste Dateinamen

[Issue #92](https://github.com/ingen63/VivaPlan/issues/92) · offen · Umsetzung: `5158ee8` (altes Repository) · Tests zum Issue ohne Kriterium: [engine/tests/legacy_import.rs:48](../crates/engine/tests/legacy_import.rs#L48), [engine/tests/legacy_import.rs:84](../crates/engine/tests/legacy_import.rs#L84)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #92/AK1 | Der Test liest alle Dateien mit der Endung `.json` im Ordner `local/eigene/` statt einer festen Liste von Dateinamen. | ☑ | **gilt** | [engine/tests/legacy_import.rs:61](../crates/engine/tests/legacy_import.rs#L61) | 15.11.2026 ok (Test grün) |
| #92/AK2 | Fehlt der Ordner oder enthält er keine `.json`-Datei, wird der Test ohne Fehler übersprungen (wie heute). | ☑ | **gilt** | [engine/tests/legacy_import.rs:61](../crates/engine/tests/legacy_import.rs#L61) | 15.11.2026 ok (Test grün) |
| #92/AK3 | Im aktuellen Stand findet `git grep -i <Name aus #92>` keine Stelle mehr. | ☑ | **gilt**: Der gesuchte Name ist hier weggelassen, damit die Suche diese Datei nicht findet; er steht im Issue. | kein Test | 15.11.2026 ok (`git grep -i` auf den Namen findet im aktuellen Stand keine Stelle) |
| #92/AK4 | Mit den bisherigen Dateien in `local/eigene/` prüft der Test dieselben Konfigurationen wie vorher; `make test` läuft grün. | ☐ | **gilt** | kein Test | nicht geprüft: braucht die privaten Dateien in local/eigene/ |

### #94 Shiller-Originaltabelle nicht im Repository, Bedingungen von SNB und BFS in den Quellen

[Issue #94](https://github.com/ingen63/VivaPlan/issues/94) · geschlossen · Umsetzung: `49bbd62` (altes Repository) · Tests zum Issue ohne Kriterium: [app/src/main.rs:1339](../crates/app/src/main.rs#L1339), [app/src/main.rs:1345](../crates/app/src/main.rs#L1345) (+6)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #94/AK1 | `git ls-files` findet `shiller-index.ods` nicht mehr; der Pfad steht in `.gitignore`; eine lokal vorhandene Datei bleibt unverändert. | ☑ | **gilt** | kein Test | 15.11.2026 ok (shiller-index.ods nicht im Index, in .gitignore) |
| #94/AK2 | Die Tests, die aus der Tabelle nachrechnen (`crates/engine/tests/library.rs`, `crates/engine/tests/data.rs`, `crates/cli/tests/cli.rs`, `crates/app/src/main.rs`), laufen mit vorhandener Datei wie bisher. Fehlt sie, werden sie ohne Fehler übersprungen und melden das. | ☑ | **gilt** | kein Test | nicht geprüft: Test mit vorhandener Datei nicht ausgeführt |
| #94/AK3 | Die ausgelieferten Reihen `Shiller USD.csv`, `Shiller CHF.csv` und `Shiller CHF + CH-Obligationen.csv` und die Golden-Dateien bleiben unverändert. | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: Golden-Dateien grün |
| #94/AK4 | `crates/engine/data/QUELLEN.md` sagt, dass die Shiller-Tabelle nicht mitgeliefert wird und wo man sie herunterlädt; es nennt für die SNB-Daten die Bedingungen «nicht kommerziell», «Quellenangabe» und «nicht als Grundlage für Finanztransaktionen» und für die BFS-Daten die Quellenangabe mit Autor, Titel und Link. | ☑ | **gilt** | kein Test (Datei geprüft: `crates/engine/data/QUELLEN.md`) | 15.11.2026 ok |
| #94/AK5 | Die Hilfe (Datenquellen) nennt den Download der Shiller-Tabelle bei shillerdata.com und den Import in der App. | ☑ | **gilt** | kein Test (Datei geprüft: `crates/app/ui/hilfe/20-renditen.md`) | 15.11.2026 ok |
| #94/AK6 | `make test` und die Oberflächentests laufen grün; die Abdeckung von Engine und App bleibt je mindestens 80 %, auch ohne die Datei (wie auf GitHub). | ☑ | **gilt** | kein Test | 15.11.2026 teilweise: Tests heute grün (mit Datei, wenn vorhanden) |

### #97 Abnahmeliste: Abnahmekriterien aller umgesetzten Issues auf den aktuellen Stand prüfen

[Issue #97](https://github.com/ingen63/VivaPlan/issues/97) · offen · Umsetzung: siehe Kommentar im Issue

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #97/AK1 | Die Liste liegt als Datei im Repository (`docs/ABNAHME.md`) und nennt je Issue Nummer, ID, Titel, Status (offen/geschlossen) und, sofern bekannt, den Commit der Umsetzung. | ☑ | **gilt** | kein Test (Datei `docs/ABNAHME.md`) | 15.11.2026 ok |
| #97/AK2 | Je Abnahmekriterium stehen Kennung (`#Nr/AKn`), Text wie im Issue, Häkchenstand im Issue und eine Einschätzung: «gilt», «überholt durch <Issue>» mit kurzer Begründung, oder «unklar». | ☑ | **gilt** | kein Test (Datei `docs/ABNAHME.md`) | 15.11.2026 ok |
| #97/AK3 | Je Kriterium ist der belegende Test genannt, sofern einer mit der Kennung existiert (Rust oder `e2e/`), sonst «kein Test». | ☑ | **gilt** | kein Test (Datei `docs/ABNAHME.md`) | 15.11.2026 ok |
| #97/AK4 | Je Kriterium gibt es eine Spalte «Geprüft» (Datum, Ergebnis), die Claude bei einer Einzelprüfung auf Anweisung nachführt. Auf Anweisung hakt Claude dabei auch die erfüllten Kriterien im Issue ab und schliesst das Issue, wenn alle erfüllt sind (geändert per Kommentar am 15.11.2026). | ☑ | **gilt** | kein Test | 15.11.2026 ok (nach geänderter Fassung) |
| #97/AK5 | Überholte Kriterien werden in den Issues nicht geändert; die Liste schlägt vor, was zu tun ist (Kommentar im Issue oder neues Issue). | ☑ | **gilt** | kein Test (Datei `docs/ABNAHME.md`) | 15.11.2026 ok |
| #97/AK6 | Die Auswahl der Issues ist reproduzierbar beschrieben (Abfrage mit `gh`), damit die Liste später nachgeführt werden kann. | ☑ | **gilt** | kein Test (Datei `docs/ABNAHME.md`) | 15.11.2026 ok |

### #116 Mutationslauf bringt den Mac zum Absturz (endlose Schleife füllt den Speicher)

[Issue #116](https://github.com/ingen63/VivaPlan/issues/116) · geschlossen · Umsetzung: 51395f5

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #116/AK1 | Speichergrenze 1,5 GB in allen Testprogrammen von Engine, App und Kommandozeile. | ☑ | **gilt** | kein Test (Durchsicht der `#[global_allocator]`-Stellen) | 16.11.2026 ok |
| #116/AK2 | Grenze getestet: über der Grenze Fehler, normal gelingt, Zähler stimmt. | ☑ | **gilt** | [test_alloc.rs](../crates/engine/src/test_alloc.rs) «issue_116_ak2_allocations_above_the_limit_…», «issue_116_ak2_reserve_refuses_…» | 16.11.2026 ok |
| #116/AK3 | Schleife in `Parametric::path` prüft den Fortschritt; Mutanten an 288 und 299 enden sofort. | ☑ | **gilt** | kein Test (Mutanten von Hand eingesetzt) | 16.11.2026 ok (beide Mutanten: Fehler nach höchstens 5 s, kein Speicherwachstum) |
| #116/AK4 | Profil `mutants`: 4 Tests je Job, Mindestzeit 60 s. | ☑ | **gilt** | kein Test (Durchsicht der Konfiguration) | 16.11.2026 ok |
| #116/AK5 | Mutanten von `montecarlo.rs` laufen ohne Speicherwachstum durch. | ☑ | **gilt** | kein Test (Lauf mit `cargo mutants -f …`) | 16.11.2026 ok (im Issue abgehakt, nicht wiederholt) |
| #116/AK6 | `docs/TESTS.md` beschreibt Grenze und Einstellungen. | ☑ | **gilt** | kein Test (Durchsicht) | 16.11.2026 ok |

### #119 Name eines Ergebnisses mit Abkürzung von Renditequelle und Strategie

[Issue #119](https://github.com/ingen63/VivaPlan/issues/119) · geschlossen · Umsetzung: a48c37e, 7628f6f (Nacharbeit AK4)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #119/AK1 | Name «Planung · Quelle · Strategie · Datum, Zeit», Zahl der Läufe in der Zeile darunter. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#119/AK1, AK2, AK3: …» | 16.11.2026 ok |
| #119/AK2 | Abkürzungen der Renditequelle, bei Szenarien «Szenarien». | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#119/AK1, AK2, AK3: …» | 16.11.2026 ok |
| #119/AK3 | Abkürzungen der Strategie. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#119/AK1, AK2, AK3: …», «#119/AK4: …» | 16.11.2026 ok |
| #119/AK4 | Abkürzungen an einer Stelle, zusammen mit `SOURCES` bzw. `STRATS`; Test auf Vollständigkeit. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#119/AK4: …» | 16.11.2026 ok (nach Nacharbeit 7628f6f; vorher teilweise: bei `sourceName` statt bei `SOURCES`/`STRATS`) |
| #119/AK5 | Gespeicherte Ergebnisse behalten ihren Namen. | ☑ | **gilt** | [ui-vergleich.spec.js](../e2e/ui-vergleich.spec.js) «#119/AK5: …» | 16.11.2026 ok |
| #119/AK6 | Hilfe nennt Aufbau und Abkürzungen. | ☑ | **gilt** | kein Test (Hilfe durchgesehen) | 16.11.2026 ok |

### #132 Monte Carlo parametrisch rechnet schneller

[Issue #132](https://github.com/ingen63/VivaPlan/issues/132) · offen (`implemented`) · Umsetzung: 2f0d36a

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #132/AK1 | Referenzlauf: Musterplanung, Quelle «Monte Carlo parametrisch», 50'000 Pfade, Release-Build, Apple M1 Pro. Die Rechenzeit ist höchstens … | ☑ | **gilt** (Anforderung am 18.11.2026 von GPU auf Tempo umformuliert, Kommentar im Issue) | kein Test (Messung alt gegen neu, `batch::run_paths`) | 18.11.2026 ok (unter Last 2'954–2'997 gegen 1'445–1'511 ms, Verhältnis 0,50; ohne Last 0,46) |
| #132/AK2 | Die Ergebnisse sind auf macOS, Windows und Linux gleich: Die Golden-Tests laufen auf allen drei Plattformen mit der heutigen Toleranz … | ☐ | **gilt** | [golden.rs](../crates/engine/tests/golden.rs) | 18.11.2026 nicht geprüft: nur macOS; Linux (Push) und Windows (`tests-os`) erst nach dem Push |
| #132/AK3 | Gleiche Einstellungen und gleicher Zufallsstartwert ergeben weiterhin dieselben Ergebnisse, und die Detailansicht eines Perzentils zeigt … | ☑ | **gilt** | [rerun_paths.rs](../crates/engine/tests/rerun_paths.rs) «issue_132_ak3_…» | 18.11.2026 ok |
| #132/AK4 | Die Verteilung der Renditen bleibt die des Modells (Mittelwert, Streuung, Fettschwänze `df`, Sprünge `jumpLambda`, `jumpMu`, `jumpSigma`); … | ☑ | **gilt** | [montecarlo.rs](../crates/engine/src/montecarlo.rs) «issue_132_ak4_…» (3 Tests) | 18.11.2026 ok |
| #132/AK5 | Die übrigen Quellen (fix, historisch, alle Startzeitpunkte, Monte Carlo historisch) werden nicht langsamer (Messung im Issue). | ☑ | **gilt** | kein Test (Messung alt gegen neu) | 18.11.2026 ok (Ergebnisse bitgleich; Bootstrap +1 % im Rauschen) |
| #132/AK6 | Die Golden-Dateien (`crates/engine/tests/golden/`) ändern sich nicht, oder die Änderung wird im Issue begründet. | ☑ | **gilt** | [golden.rs](../crates/engine/tests/golden.rs) | 18.11.2026 ok (nur `08-parametrisch.json`, begründet im Issue) |
| #132/AK7 | Ändern sich die Zufallspfade, zeigt die App beim Öffnen einer älteren Planung mit «Monte Carlo parametrisch» einen Hinweis, dass sich das … | ☑ | **gilt** | [parametric_fit.rs](../crates/engine/tests/parametric_fit.rs) «issue_132_ak7_…», [ui-abnahme.spec.js](../e2e/ui-abnahme.spec.js) «AK7: …» | 18.11.2026 ok |
| #132/AK8 | Keine Netzwerkzugriffe und keine Abhängigkeit von Grafiktreibern; `docs/ENTWICKLUNG.md` (Abschnitt Monte Carlo) beschreibt das Verfahren. | ☑ | **gilt** | kein Test (Durchsicht `Cargo.toml`, `docs/ENTWICKLUNG.md`) | 18.11.2026 ok |

### #143 Steuer als Schätzung kennzeichnen

[Issue #143](https://github.com/ingen63/VivaPlan/issues/143) · offen (Teilumsetzung, Rest mit #63 in M3) · Umsetzung: 8ed364e

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #143/AK1 | Überschriften, Diagrammlegenden, Übersicht der Annahmen und Export sprechen von «geschätzter Steuer»; die Jahreswerte … | ☑ | **gilt** | [ui-ergebnisse.spec.js](../e2e/ui-ergebnisse.spec.js) «AK1» (#143); [cli/tests/cli.rs](../crates/cli/tests/cli.rs) `run_single_with_outputs` | 22.11.2026 ok (Kennzahl, Legende, Kopfzeilen «Steuer geschätzt», Übersicht, CSV `taxes_estimated`) |
| #143/AK2 | Die Oberfläche zeigt Steuerbeträge auf 100 Franken gerundet. Der Rechenkern rechnet weiter genau; Golden-Dateien … | ☑ | **gilt** | [ui-ergebnisse.spec.js](../e2e/ui-ergebnisse.spec.js) «AK2» (#143); [unit/util.test.js](../e2e/unit/util.test.js) `#143/AK2` | 22.11.2026 ok (Golden-Dateien unverändert, 865 Rust-Tests grün) |
| #143/AK3 | Der Abschnitt Steuern zeigt einen fest sichtbaren Hinweis: Steuerjahr und Quelle der Daten, Herkunft der Abzüge, … | ☐ | **gilt** | [ui-ergebnisse.spec.js](../e2e/ui-ergebnisse.spec.js) «AK3» (#143) | 22.11.2026 teilweise: Hinweis im Modus «Eigene Tabellen» ok; Steuerjahr, Herkunft der Abzüge und Näherungen des Kantons warten auf #63 |
| #143/AK4 | Unter der Steuerzerlegung und den Jahreswerten steht eine Zeile mit Steuerjahr der Daten, Herkunft der Abzüge und … | ☐ | **gilt** | [ui-ergebnisse.spec.js](../e2e/ui-ergebnisse.spec.js) «AK4» (#143) | 22.11.2026 teilweise: Zeile unter den Jahreswerten mit «Keine Steuerberatung» ok; Steuerjahr, Herkunft der Abzüge und Steuerzerlegung fehlen (#63) |
| #143/AK5 | Ist das laufende Kalenderjahr grösser als das Steuerjahr der Daten + 1, meldet die Prüfung eine Warnung. | ☐ | **gilt** | kein Test | nicht geprüft: zurückgestellt bis #63 (M3) |
| #143/AK6 | Bei Kapitalbezugs- und Grundstückgewinnsteuer steht zusätzlich: «Vor dem Entscheid mit dem Steuerrechner des Kantons … | ☑ | **gilt** | [ui-ergebnisse.spec.js](../e2e/ui-ergebnisse.spec.js) «AK6» (#143, 2 Tests) | 22.11.2026 ok |
| #143/AK7 | Der Haftungsausschluss nennt ausdrücklich: keine Steuer- oder Anlageberatung. | ☑ | **gilt** | [ui-ergebnisse.spec.js](../e2e/ui-ergebnisse.spec.js) «AK7» (#143); [ui-einstieg.spec.js](../e2e/ui-einstieg.spec.js) #61 | 22.11.2026 ok (Version 2, wird erneut bestätigt) |
| #143/AK8 | Hilfe und `BESCHREIBUNG.md` haben einen Abschnitt «Wie genau ist die Steuer?» mit den Referenzfällen, den Näherungen … | ☑ | **gilt** | [ui-ergebnisse.spec.js](../e2e/ui-ergebnisse.spec.js) «AK8» (#143); BESCHREIBUNG.md Durchsicht | 22.11.2026 ok (Hilfe in vier Sprachen) |
| #143/AK9 | Eine Bandbreite (±x %) wird nicht angezeigt. | ☑ | **gilt** | [ui-ergebnisse.spec.js](../e2e/ui-ergebnisse.spec.js) «AK9» (#143) | 22.11.2026 ok |

### #188 Release Notes in vier Sprachen

[Issue #188](https://github.com/ingen63/VivaPlan/issues/188) · offen (`implemented`) · Umsetzung: c17f7ff, 5ebe461 (Titel)

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #188/AK1 | Neben `CHANGELOG.md` gibt es `CHANGELOG.en.md`, `CHANGELOG.fr.md` und `CHANGELOG.it.md`, jede mit einem Eintrag für jede Version ab … | ☑ | **gilt** | [test_changelog.py](../tools/test_changelog.py) «issue_188_ak1_…» (2 Tests) | 21.11.2026 ok |
| #188/AK2 | Jeder Eintrag in jeder Sprache trennt die Abschnitte «Neu/Geändert/Behoben», «Kann Ergebnisse ändern» und «Tests und Entwicklung». … | ☑ | **gilt** (Reihenfolge: «Kann Ergebnisse ändern» zuerst) | [test_changelog.py](../tools/test_changelog.py) «issue_188_ak2_…» (5 Tests) | 21.11.2026 ok |
| #188/AK3 | Eine Prüfung meldet einen Fehler, wenn sich eine Übersetzung von der deutschen Fassung unterscheidet: in den Versionsüberschriften, … | ☑ | **gilt** | [test_changelog.py](../tools/test_changelog.py) «issue_188_ak3_…» (9 Tests) | 21.11.2026 ok (Beispiel #185 in fr) |
| #188/AK4 | Die Prüfung meldet in den Übersetzungen verbotene Varianten aus `docs/GLOSSAR.md`. Beispiel: «Liquidità» in `CHANGELOG.it.md` … | ☑ | **gilt** | [test_changelog.py](../tools/test_changelog.py) «issue_188_ak4_…» (3 Tests) | 21.11.2026 ok |
| #188/AK5 | Die Prüfung läuft in `make test-quiet` und im GitHub-Lauf (Job `english` oder ein eigener Job). | ☑ | **gilt** | [test_changelog.py](../tools/test_changelog.py) «issue_188_ak5_…» | 21.11.2026 ok: `make test-quiet` lokal, Lauf [44922524561](https://github.com/ingen63/VivaPlan/actions/runs/44922524561) |
| #188/AK6 | Ein Skript in `tools/` erzeugt aus den vier Dateien den Text für den Release einer Version: Englisch zuerst und offen; … | ☑ | **gilt** | [test_changelog.py](../tools/test_changelog.py) «issue_188_ak6_…» (6 Tests) | 21.11.2026 ok (Entwurf v1.0.0-beta.2 trägt den Text) |
| #188/AK7 | `pakete.yml` setzt diesen Text als Beschreibung des Release-Entwurfs. Der Paketbau bricht ab, wenn die Version des Tags in einer der … | ☐ | **gilt** (dazu der Titel, Kommentar im Issue) | [test_changelog.py](../tools/test_changelog.py) «issue_188_ak7_…» | 21.11.2026 nicht geprüft: nur lokal nachgestellt; auf GitHub erst beim nächsten Tag |
| #188/AK8 | `docs/ENTWICKLUNG.md` beschreibt den Ablauf pro Release: deutschen Eintrag schreiben, übersetzen, prüfen, taggen. | ☑ | **gilt** | [test_changelog.py](../tools/test_changelog.py) «issue_188_ak8_…» | 21.11.2026 ok |

### #202 GitHub-Actions-Verbrauch senken

[Issue #202](https://github.com/ingen63/VivaPlan/issues/202) · offen (`implemented`) · Umsetzung: 12c3a2f

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #202/AK1 | `mutants-diff` startet nicht mehr nächtlich, sondern einmal pro Woche zu einer festen Zeit (`schedule`). Die übrigen nächtlichen … | ☐ | **gilt** | kein Test (Durchsicht `test-quality.yml`, `actionlint`) | 22.11.2026 teilweise: Konfiguration ok (`17 2 * * 0`, übrige Jobs schliessen ihn aus); wöchentlicher Start erst nach Push belegbar |
| #202/AK2 | `mutants-diff` läuft in 4 Teilen (`--shard i/4`) statt in 16. | ☑ | **gilt** | kein Test (Durchsicht `test-quality.yml`) | 22.11.2026 ok |
| #202/AK3 | Der wöchentliche Lauf mutiert nur die geänderten Zeilen des Rechenkerns (`crates/engine`). Er vergleicht mit dem letzten … | ☐ | **gilt** | kein Test (Durchsicht; Abfrage `gh run list`/`gh run view` lokal gegen vorhandene Läufe) | 22.11.2026 teilweise: Pfadangabe und Basis-Abfrage lokal ok; echter wöchentlicher Lauf fehlt |
| #202/AK4 | Ein Start von Hand mit `nightly` startet `mutants-diff` weiterhin. Bei einem Pull Request bleibt das Verhalten gleich. | ☑ | **gilt** | kein Test (Durchsicht `test-quality.yml`) | 22.11.2026 ok |
| #202/AK5 | «Installationspakete» baut nur das `.dmg` für macOS, als Start von Hand und durch ein Versions-Tag. Der Release-Entwurf … | ☐ | **gilt** | kein Test (Durchsicht `pakete.yml`, `actionlint`) | 22.11.2026 teilweise: Matrix nur `macos`; Lauf mit Release-Entwurf fehlt |
| #202/AK6 | `docs/TESTS.md` und die Kommentare in den Workflows beschreiben den neuen Ablauf. Hilfe und Übersicht nennen die Pakete für … | ☑ | **gilt** | kein Test (Durchsicht, `grep`) | 22.11.2026 ok: TESTS, ENTWICKLUNG, BESCHREIBUNG, README; Hilfe nannte nie Pakete |

### #220 Auswahlmenu ohne kontrast

[Issue #220](https://github.com/ingen63/VivaPlan/issues/220) · offen (`implemented`) · Umsetzung: 1774c54, f78c56c · Kriterien im Klärungskommentar vom 23.11.2026

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #220/AK1 | In den Einstellungen gibt es «Darstellung: System / Hell / Dunkel», Vorgabe «System». Die Wahl wird in den … | ☑ | **gilt** | [ui-darstellung.spec.js](../e2e/ui-darstellung.spec.js) `AK1` (2 Tests) | 23.11.2026 ok (Browser-Speicher und App-Einstellungen über `settings_set`, Neuladen, «Standardwerte») |
| #220/AK2 | Bei «System» folgt die App `prefers-color-scheme`, auch wenn das Thema zur Laufzeit wechselt. | ☑ | **gilt** | [ui-darstellung.spec.js](../e2e/ui-darstellung.spec.js) `AK2` | 23.11.2026 ok |
| #220/AK3 | Im dunklen Theme sind Oberfläche, Auswahlmenüs (`select`/`option`), Eingabefelder, Dialoge, Menüs und Hilfe lesbar … | ☑ | **gilt** | [ui-darstellung.spec.js](../e2e/ui-darstellung.spec.js) `AK3 AK6` (4 Tests) | 23.11.2026 ok (Kontrast ≥ 4,5:1; bei der Abnahme behoben: ⓘ-Knöpfe mit nativem Hintergrund, f78c56c) |
| #220/AK4 | Die Diagramme (Achsen, Gitter, Beschriftungen, Linien, Flächen) sind im dunklen Theme lesbar. | ☑ | **gilt** | [ui-darstellung.spec.js](../e2e/ui-darstellung.spec.js) `AK4` | 23.11.2026 ok (dazu Sichtprüfung am Bildschirmfoto der Ergebnisse) |
| #220/AK5 | Im hellen Theme ändert sich das Aussehen nicht, auch nicht unter einem dunklen Systemthema (Ursache dieses Fehlers). | ☑ | **gilt** | [ui-darstellung.spec.js](../e2e/ui-darstellung.spec.js) `AK5` | 23.11.2026 ok (berechnete Farben aller Elemente gleich; aktiver Menüeintrag hell 4,1:1, bestehendes Design) |
| #220/AK6 | Ein e2e-Test prüft den Kontrast der wichtigsten Elemente (inkl. Auswahlmenü) in beiden Themen bei emuliertem … | ☑ | **gilt** | [ui-darstellung.spec.js](../e2e/ui-darstellung.spec.js) `AK3 AK6` | 23.11.2026 ok |
| #220/AK7 | Texte als Schlüssel in `i18n`; Hilfe und `BESCHREIBUNG.md` nachgeführt. | ☑ | **gilt** | `make ui-texts` | 23.11.2026 ok (Hilfe de/en/fr/it) |
| #220/AK8 | Unter Linux (AppImage, dunkles Systemthema) sind die Auswahlmenüs lesbar. Prüfbar erst mit dem nächsten Paket. | ☐ | **gilt** | kein Test | 23.11.2026 nicht geprüft: erst mit dem nächsten AppImage |

### #221 Robustheitstest über alle Rechenbefehle mit unsinnigen Argumenten

[Issue #221](https://github.com/ingen63/VivaPlan/issues/221) · offen (`implemented`) · Umsetzung: 4162603

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #221/AK1 | Ein Test schickt jeden Befehl der Tabelle mit unsinnigen Argumenten an `engine::dispatch`: falscher Typ, fehlendes … | ☑ | **gilt** (Mengenfelder mit Fehlermeldung, Kommentar vom 23.11.2026) | [command_sweep.rs](../crates/engine/tests/command_sweep.rs) `issue_221_ak1_…` (2 Tests); [optimize.rs](../crates/engine/src/optimize.rs) `issue_221_ak1_…` (2 Tests) | 23.11.2026 ok (Push-Umfang 133 Aufrufe in 9 s; nächtlicher Umfang lokal 1'358 Aufrufe in 89 s ohne Absturz) |
| #221/AK2 | Jeder Aufruf endet mit einem Ergebnis oder einer Fehlermeldung; ein Absturz in irgendeinem Thread (auch Rayon) lässt … | ☑ | **gilt** | [command_sweep.rs](../crates/engine/tests/command_sweep.rs) `issue_221_ak1_every_command_…` (Panic-Hook) | 23.11.2026 ok (während der Umsetzung meldete er echte Funde mit Befehl und Argumenten: `bench_plan` mit Liste als Planung, Speicherabbruch bei `rerun_paths` mit 10'000 Läufen; behoben) |
| #221/AK3 | Ein Hänger wird beendet und lässt den Lauf fehlschlagen (Zeitgrenze im nextest-Profil `ci`); die letzte Ausgabe … | ☐ | **gilt** | `.config/nextest.toml` (Override `command_sweep`) | 23.11.2026 teilweise: Konfiguration gelesen (`nextest list --profile ci`), jeder Aufruf steht vorher in der Ausgabe; Abbruch eines Hängers nur in CI belegbar |
| #221/AK4 | Ein neuer Befehl in `ipc-commands.json` ist ohne weiteres Zutun im Test enthalten. | ☑ | **gilt** | [command_sweep.rs](../crates/engine/tests/command_sweep.rs) `issue_221_ak4_…` | 23.11.2026 ok (Befehle und Beispielargumente aus der Tabelle gelesen) |
| #221/AK5 | Ein Befehl mit gültigen Argumenten, der ein Ergebnis mit NaN oder ∞ liefert, lässt den Test fehlschlagen. | ☑ | **gilt** | [command_sweep.rs](../crates/engine/tests/command_sweep.rs) `issue_221_ak5_…`; [finite.rs](../crates/engine/src/finite.rs) `issue_221_ak5_…` | 23.11.2026 ok (`dispatch` meldet NaN/∞ als `result_not_finite`, der Test verlangt mit gültigen Argumenten ein Ergebnis) |
| #221/AK6 | Die Fallzahl folgt dem Budget wie bei proptest (wenige Fälle bei jedem Push, viele nächtlich). | ☑ | **gilt** | [command_sweep.rs](../crates/engine/tests/command_sweep.rs) `issue_221_ak6_…` | 23.11.2026 ok (`PROPTEST_CASES` 256 → 133 Aufrufe, 10'000 → alle systematischen und 625 zufällige) |
| #221/AK7 | `docs/TESTS.md` nennt den Test in der Schicht 5. | ☑ | **gilt** | kein Test (Durchsicht) | 23.11.2026 ok |

### #226 Laufzeitgrenze für einzelne Tests

[Issue #226](https://github.com/ingen63/VivaPlan/issues/226) · offen (`implemented`) · Umsetzung: 3c97ec3

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #226/AK1 | Ein Skript liest die Laufzeiten aus `target/nextest/ci/junit.xml` und listet jeden Test über der Grenze ohne … | ☑ | **gilt** (Grenze je Testart, Klärung vom 23.11.2026) | [test_slow_tests.py](../tools/test_slow_tests.py) `test_issue_226_ak1_ak3_…` | 23.11.2026 ok (9 Tests grün; am echten Lauf: 80 Funde vor, 30 nach den Begründungen) |
| #226/AK2 | Der Befund erscheint in der Zusammenfassung von `test-quality.yml` und auf der Übersichtsseite (Schicht «Hygiene» … | ☐ | **gilt** | [test_slow_tests.py](../tools/test_slow_tests.py) `test_issue_226_ak2_…`; [test_report.py](../tools/report/test_report.py) `test_issue_226_ak2_…` | 23.11.2026 teilweise: Seite lokal mit echtem Lauf erzeugt (Hygiene «Hinweis», Liste), Markdown getestet, `actionlint` ohne neuen Befund; Ausgabe im CI-Lauf erst nach dem nächsten Push belegbar |
| #226/AK3 | Das Skript hat eigene Tests mit gestellten junit-Dateien (Test unter Grenze, über Grenze mit und ohne Begründung). | ☑ | **gilt** | [test_slow_tests.py](../tools/test_slow_tests.py) `test_issue_226_ak3_…` (3 Tests) | 23.11.2026 ok |
| #226/AK4 | Die heute langsamen Tests sind entweder schneller oder begründet; die Liste steht im Issue. | ☐ | **unklar** (30 Tests nur unter Last über der Grenze) | kein Test (Messung `--profile ci`, voll und `-j 1`) | 23.11.2026 teilweise: Liste im Issue; 50 auch einzeln langsame Tests mit `// slow:` begründet; 30 nur im parallelen Lauf über der Grenze, Entscheid offen |
| #226/AK5 | `docs/TESTS.md` nennt Grenze und Begründungsform. | ☑ | **gilt** | kein Test (Durchsicht) | 23.11.2026 ok |

### #225 Rust-Tests unter Windows bei jedem Push

[Issue #225](https://github.com/ingen63/VivaPlan/issues/225) · offen (`implemented`) · Umsetzung: 0099b33

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #225/AK1 | `test-quality.yml` führt die Rust-Tests bei jedem Push auf `main` auch unter Windows aus (ohne Abdeckung, ohne … | ☐ | **gilt** | kein Test (Durchsicht `test-quality.yml`, `actionlint`) | 23.11.2026 teilweise: Job `tests-os` mit Matrix `windows-latest` beim Push, `cargo nextest run --workspace` ohne llvm-cov/mutants; Lauf beim Push erst nach dem nächsten Push belegbar |
| #225/AK2 | Der Windows-Job läuft nur, wenn Rust-Code, `Cargo.toml`/`Cargo.lock` oder der Workflow geändert wurden (wie der Job … | ☑ | **gilt** | kein Test (Filter von Hand gegen echte Pushes) | 23.11.2026 ok: Ausgabe `rust` des Jobs `changes`; Compare-API gegen 4 Pushes mit Rust-Code (`true`) und 1 nur mit Doku (`false`); unbekannter Stand → `true` |
| #225/AK3 | Die Dauer des Jobs und die zusätzlichen Actions-Minuten pro Woche sind gemessen und im Issue festgehalten; überschreiten … | ☐ | **gilt** (Budget 300 min/Woche, Klärung vom 23.11.2026) | kein Test | 23.11.2026 teilweise: Schätzung aus nächtlichen Läufen im Issue (6–8 min, ≈160–184 min/Woche, unter Budget → ganzer Workspace); Messung beim Push fehlt |
| #225/AK4 | macOS bleibt nächtlich. | ☑ | **gilt** | kein Test (Durchsicht) | 23.11.2026 ok: Matrix beim Push nur `windows-latest`, sonst Windows und macOS |
| #225/AK5 | `docs/TESTS.md`, Abschnitt «Plattformen», ist nachgeführt. | ☑ | **gilt** | kein Test (Durchsicht) | 23.11.2026 ok |

### #224 Übersprungene Tests in CI als Fehler werten

[Issue #224](https://github.com/ingen63/VivaPlan/issues/224) · offen (`implemented`) · Umsetzung: ab71e68

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #224/AK1 | Jeder bedingt laufende Test meldet beim Überspringen einen Grund in der Testausgabe; kein stilles `return`. | ☑ | **gilt** | [test_resource.rs](../crates/engine/src/test_resource.rs) `issue_224_ak1_…` | 23.11.2026 ok (Windows-, Unix- und Datentests über `test_resource`, Grund mit `--success-output immediate` gesehen; `#[ignore]` mit Grund; e2e und `ui-smoke.sh` überspringen nichts) |
| #224/AK2 | Eine Umgebungsvariable je Ressource (z. B. `VIVAPLAN_WINDOWS_REQUIRED`): leer oder nicht gesetzt = überspringen, `1` = … | ☑ | **gilt** | [test_resource.rs](../crates/engine/src/test_resource.rs) `issue_224_ak2_…` (2 Tests) | 23.11.2026 ok (`WINDOWS`, `UNIX`, `DATA`; ungültiger Wert von Hand) |
| #224/AK3 | Die Jobs in `test-quality.yml`, die die Ressource haben (z. B. `tests-os` unter Windows für `issue_141_ak3_windows_paths`), … | ☐ | **gilt** | kein Test (Durchsicht `test-quality.yml`, `actionlint`) | 23.11.2026 teilweise: `tests-os` (Windows, macOS) und `tests-coverage` setzen die Variablen; Lauf in CI erst nach dem nächsten Push belegbar |
| #224/AK4 | Ein Test prüft, dass bei gesetzter Pflicht und fehlender Ressource ein Fehler entsteht. | ☑ | **gilt** | [test_resource.rs](../crates/engine/src/test_resource.rs) `issue_224_ak4_…` (2 Tests) | 23.11.2026 ok (zusätzlich von Hand: `VIVAPLAN_WINDOWS_REQUIRED=1` auf macOS → Fehler) |
| #224/AK5 | `docs/TESTS.md` listet die Variablen und die Jobs, die sie setzen. | ☑ | **gilt** | kein Test (Durchsicht) | 23.11.2026 ok |

### #237 Spendenlink «Buy me a coffee»

[Issue #237](https://github.com/ingen63/VivaPlan/issues/237) · offen (`implemented`) · Umsetzung: 0806a57

| Kennung | Kriterium | Im Issue | Einschätzung | Test | Geprüft |
|---|---|---|---|---|---|
| #237/AK1 | Im Info-/Hilfebereich steht ein Eintrag «Kaffee spendieren» (Text über `de.json`, Anrede «Sie»). | ☑ | **gilt** (Ort und Text nach Entscheid im Issue: Menü Hilfe «VivaPlan unterstützen…», Navigation, Fenster «Über VivaPlan») | `main.rs` `issue_237_ak1_…`, [ui-spende.spec.js](../e2e/ui-spende.spec.js) `AK1` (4 Tests) | 23.11.2026 ok (Eintrag vor «Über VivaPlan», graue Zeile unter `nav.localData`, Zeile «Unterstützen» unter «Website»; Texte in de, en, fr, it; Tooltip «Freiwillig einen Kaffee spendieren …») |
| #237/AK2 | Ein Klick öffnet die festgelegte Buy-me-a-coffee-Adresse im Standardbrowser, nicht in einem App-Fenster. | ☑ | **gilt** | [links.rs](../crates/app/src/links.rs) `issue_237_ak2_…` (2 Tests), `main.rs` `issue_237_ak2_open_link_rejects_addresses`, [ui-spende.spec.js](../e2e/ui-spende.spec.js) `AK2` | 23.11.2026 ok (Befehl `open_link` mit Kennung aus `links.json` über `tauri_plugin_opener::open_url`; fremde Adressen abgewiesen; im Browser neuer Tab; Klick in der installierten App nicht von Hand geprüft) |
| #237/AK3 | Die App sendet keine Anfrage ins Netz. Sie lädt weder Logo noch Zähler und prüft nicht, ob die Seite erreichbar ist. | ☑ | **gilt** | [ui-spende.spec.js](../e2e/ui-spende.spec.js) (alle Tests prüfen: keine Anfrage ausser an den lokalen Server) | 23.11.2026 ok (kein Bild, kein Abruf; CSP unverändert) |
| #237/AK4 | Der Hinweis stört nicht: kein Popup, kein Hinweis beim Start, keine Wiederholung. | ☑ | **gilt** | [ui-spende.spec.js](../e2e/ui-spende.spec.js) `AK4` | 23.11.2026 ok |
| #237/AK5 | Die Hilfe (`crates/app/ui/hilfe/`) und `docs/BESCHREIBUNG.md` erwähnen den Link. | ☑ | **gilt** | kein Test (Durchsicht) | 23.11.2026 ok (Hilfe de, en, fr, it «Bedienung und Tastenkürzel»; BESCHREIBUNG «Hilfe und Bedienung»; zusätzlich ENTWICKLUNG) |
| #237/AK6 | `README.md` hat vor dem Abschnitt «Lizenz» einen kurzen Absatz … mit Link auf https://buymeacoffee.com/vivaplan. | ☑ | **gilt** | kein Test (Durchsicht) | 23.11.2026 ok (Abschnitt «Unterstützen») |
| #237/AK7 | `.github/FUNDING.yml` enthält `buy_me_a_coffee: vivaplan`; auf der Repo-Seite erscheint der Knopf «Sponsor». | ☐ | **gilt** | kein Test (Durchsicht) | 23.11.2026 teilweise: Datei vorhanden; Knopf «Sponsor» erst nach dem Push prüfbar |
| #237/AK8 | `tools/check-network-free.sh` bleibt grün (`tauri-plugin-opener` zieht keine Netzwerkbibliothek nach). | ☑ | **gilt** | `tools/check-network-free.sh` | 23.11.2026 ok (neu u. a. `open`, `zbus`; keine gesperrte Bibliothek) |
