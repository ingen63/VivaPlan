# FEA-53: Anlagestrategie statt Renditewahl

Issue: FEA-53 (#69). Umsetzung: `crates/engine/src/strategy.rs`, `crates/engine/src/upgrade.rs`, Oberfläche
`crates/app/ui/app.js`; Tests `crates/engine/tests/strategy.rs`, `e2e/ui.spec.js` (FEA-53).

## Entscheide (13.11.2026, vor der Umsetzung)

| Frage | Entscheid |
|---|---|
| Bleibt «Schweiz (Pictet)»? | Ja, für Aktien und Obligationen, solange die Anfrage bei Pictet offen ist. **Nachtrag FEA-72 (#90, 14.11.2026):** entfernt, weil keine offene Lizenz vorliegt. Planungen mit `chPictet` (Aktien oder Obligationen) oder `library:Pictet CH.csv` erhalten beim Öffnen `chJst` bzw. `ch`, mit Hinweis. Die Angaben zu Pictet weiter unten beschreiben den Stand vor FEA-72. |
| Wohin kommt die Methode (fixe Renditen, historisch, Monte Carlo)? | In den Reiter «Simulation» (ersetzt «Renditen & Daten», ohne Wahl der Zeitreihe). |
| Gemischte Häufigkeiten | Jährlich, sobald ein Teil jährlich ist; Zeitraum = gemeinsame Jahre. |
| Planungen in EUR oder USD | Alle Indizes in CHF, Hinweis wie bisher. |
| EU-Obligationen | Gibt es nicht. |
| Länder ausserhalb von CH, USA und EU | «Asien-Pazifik» (Japan, Australien) als vierte Region in «Welt individuell», damit dieselben Länder wählbar sind wie im Weltindex. |

## Planung

`returns.strategy` = `{ "stocks": …, "bonds": …, "weights": { "ch", "usa", "eu", "apac" } }`

- Aktien: `worldJst` (Welt (JST)), `worldCustom` (Welt individuell), `chJst` (Schweiz (JST)), `chPictet` (Schweiz
  (Pictet)), `sp500` (S&P 500, Shiller).
- Obligationen: `ch` (Eidgenossenschaft 10 Jahre), `chPictet` (Schweiz (Pictet)), `usa` (USA 10 Jahre, Shiller).
- Gewichte nur für `worldCustom`; jedes zwischen 0 und 1, zusammen 1 (Fehler im Abschnitt «Vermögen & Anlage» am
  Feld `returns.strategy.weights`). Vorgabe: Schweiz 20 %, USA 40 %, EU 30 %, Asien-Pazifik 10 %.
- Neue Planungen: `worldJst` mit `ch`.
- `returns.series` (ältere Planungen, Kommandozeile) hat Vorrang; in der App wird es beim Öffnen zur Strategie.

## Zeitreihe zur Strategie

- **Ausgelieferte Kombinationen** liefern genau die bisherige Reihe (aus der Datenbibliothek, sonst eingebaut):
  `sp500`+`usa` = «Shiller CHF» (monatlich ab 01.1922), `sp500`+`ch` = «Shiller CHF + CH-Obligationen» (monatlich
  02.1988–07.2025), `chPictet`+`chPictet` = «Pictet CH». Ältere Planungen rechnen damit genau gleich.
- **Alle anderen** Kombinationen sind jährlich über die gemeinsamen Jahre. Monatsrenditen (Shiller) werden zu
  Kalenderjahren verknüpft (nur ganze Jahre). Teuerung aus der Quelle der Aktien (Shiller: Dezemberwert gegenüber
  Vorjahr; Pictet; JST: Schweizer Konsumentenpreise). SARON: Jahresmittel der SNB-Monatswerte (ab 2000), sonst fix.
- **Welt individuell:** Rendite = Σ Gewicht × Region; jede Region nach BIP gewichtet (siehe FEA-51). Eine Änderung des
  EU-Gewichts ändert jedes EU-Land im gleichen Verhältnis.

| Aktien \ Obligationen | `ch` | `chPictet` | `usa` |
|---|---|---|---|
| `worldJst`, `worldCustom`, `chJst` | jährlich 1900–2020 | jährlich 1926–2020 | jährlich 1922–2020 |
| `chPictet` | jährlich 1926–2024 | Pictet CH, jährlich 1926–2024 | jährlich 1926–2024 |
| `sp500` | monatlich 02.1988–07.2025 | jährlich 1926–2024 | Shiller CHF, monatlich 01.1922–12.2025 |

## Übernahme älterer Planungen (AK5)

| `returns.series` | Strategie | Hinweis |
|---|---|---|
| `library:Shiller CHF.csv` oder keine | `sp500` + `usa` | nein (gleiches Ergebnis) |
| `library:Shiller CHF + CH-Obligationen.csv` | `sp500` + `ch` | nein |
| `library:Pictet CH.csv` | `chPictet` + `chPictet` | nein |
| `library:Welt (JST).csv` | `worldJst` + `ch` | nein |
| `library:Shiller USD.csv` | `sp500` + `usa` (in CHF) | ja, das Ergebnis ändert sich |
| eigene Datei | `worldJst` + `ch` | ja, eigene Zeitreihen werden nicht mehr unterstützt |

## Oberfläche

- «Vermögen & Anlage»: Karte «Anlagestrategie» mit Auswahl für Aktien und Obligationen, bei «Welt individuell» vier
  Gewichte mit Summe; darunter Zeitraum, Häufigkeit und mittlere Renditen im gemeinsamen Zeitraum (AK8: Hinweis, wenn
  ein Index auf diesen Zeitraum gekürzt wird, siehe unten); Knöpfe «Shiller-Daten aktualisieren…» und
  «SNB-Daten aktualisieren…».
- Reiter «Simulation» (vorher «Renditen & Daten»): Methode, Parameter, Fortschreibung, fixe Werte; die Zeitreihe ist
  nicht mehr wählbar, «Eigene Datei wählen…» entfällt.

## Nachtrag AK8: Gemeinsamer Zeitraum und gekürzte Indizes (14.11.2026)

Befund bei der Abnahme: Ein Wechsel nur des Aktienindex änderte die angezeigte mittlere Rendite der Obligationen (und
umgekehrt). Ursache ist der gemeinsame Zeitraum: Die Reihe umfasst nur die Jahre, für die beide Indizes Werte haben,
und die Mittelwerte gelten für diese Jahre. Beispiel: Obligationen «Schweiz (Pictet)» 1926–2024 haben im Mittel
3,99 %; mit Aktien «Schweiz (JST)» (1900–2020) zählen nur 1926–2020, das Mittel ist 4,18 %.

- **Rechenkern:** `strategy::dataset_with_parts` liefert zur Reihe die Teile (Aktien, Obligationen) mit ihren eigenen
  Jahren; `series_info` gibt sie als `parts` weiter (`library::SeriesPart`, leer bei einer einzelnen Zeitreihe). Bei
  den ausgelieferten Kombinationen gilt der Zeitraum der Reihe; bei «S&P 500» mit «Eidgenossenschaft 10 Jahre» haben
  die Aktien den Zeitraum von «Shiller CHF».
- **Oberfläche:** Die Kurzinfo lautet «Mittlere Jahresrendite im gemeinsamen Zeitraum 1926–2020 von Aktien und
  Obligationen». Reicht ein Teil über den gemeinsamen Zeitraum hinaus, folgt der Hinweis «⚠ <Teil> reichen von <von>
  bis <bis> und werden auf den gemeinsamen Zeitraum <von>–<bis> gekürzt. Die Simulation (Monte Carlo und historische
  Verläufe) nutzt nur diesen Zeitraum.» Ohne Kürzung erscheint kein Hinweis.
- **Tests:** `fea53_ak8_parts_and_common_period_of_the_averages` (Rechenkern), «FEA-53/AK8» in `e2e/ui.spec.js`.
