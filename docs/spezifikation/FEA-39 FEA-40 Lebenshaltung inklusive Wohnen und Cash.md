# Lebenshaltung inklusive Wohnkosten (FEA-39) und Cash getrennt vom Portfolio (FEA-40)

Issues: FEA-39 (#53), FEA-40 (#54). Diese Seite beschreibt die Umsetzung; Anforderungen und Abnahmekriterien stehen
in den Issues.

## FEA-39 Lebenshaltung inklusive Wohnkosten

- Planungsfeld `flows.spendingIncludesHousing` (neue Planungen `true`). Fehlt es (ältere Planungen), gilt die alte
  Bedeutung (Lebenshaltung ohne Wohnkosten); beim Öffnen rechnet `upgrade` um: Wohnanteil zu jedem Wert addiert,
  Feld gesetzt, Hinweis. Das Ergebnis bleibt dadurch gleich.
- Wohnanteil (`crates/engine/src/housing.rs`, `start_housing`): Wohnkosten im Startmonat als Monatsdurchschnitt: Miete
  (Mietverhältnis ab spätestens Start, nicht «nach Verkauf»), Nebenkosten der Immobilien im Besitz (bzw. Kauf im
  Startmonat), Zins = Schuld × Satz / 12 der Periode, die den Start enthält (SARON: fixer Wert der Planung im
  Startmonat, mindestens 0, plus Marge), Amortisation = Quartalsbetrag / 3 (direkt und indirekt).
- Simulation: geplante Lebenshaltung = (Eingabe − Wohnanteil) × Faktor × Teuerungsindex; dazu die simulierten
  Wohnkosten. `MonthRecord::planned_spending` ist die Lebenshaltung ohne Wohnen; «Ausgaben total» = geplante
  Lebenshaltung + Wohnkosten + 3a-Einzahlung.
- Variation «Ausgaben-Faktor»: setzt `flows.spendingFactor` (gilt für die Lebenshaltung ohne Wohnen) statt die Reihe
  zu skalieren.
- Prüfung: Fehler, wenn eine Zeile kleiner als der Wohnanteil ist (Feld `flows.spending.value` bzw.
  `flows.spending.rows.N.value`).

## FEA-40 Cash getrennt vom Portfolio

- Start: Die Bestände werden unverändert übernommen, ausser es ist schon jemand pensioniert (dann teilt die Strategie
  wie bisher auf).
- Vor der Pensionierung (Bezug Pensionskasse der ersten Person, `any_retired`): monatlich Cash = `wealth.cash` ×
  Teuerungsindex; Überschuss nach der Aktienquote ins Portfolio, Fehlbetrag daraus nachgefüllt (`rebalance`).
- Ab der Pensionierung wie bisher: «Standard» mit `wealth.reserveMonths`, andere Strategien 3 Monate, 3-Topf mit
  ihren Töpfen.
- Begriffe: «Liquidität» heisst «Cash», «Liquiditätsreserve» «Cash-Reserve»; «liquides Vermögen» bleibt. Die
  Suchbegriffe der Hilfe enthalten weiter «Liquidität». Feldnamen der Planungsdatei bleiben.

## Golden-Dateien

Geändert: `01-einzelperson` (einzige Planung mit Pensionierung nach dem Start): Cash 100'000 bleibt bis zur
Pensionierung erhalten, statt in einer Reserve von 12 Monaten ungedeckter Ausgaben (hier 0) ins Portfolio zu gehen.
Alle anderen Planungen sind beim Start pensioniert und unverändert.
