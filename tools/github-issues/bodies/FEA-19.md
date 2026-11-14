Die Optimierung hat eine Auswahl «Ziel» mit zwei Einträgen; sie wird mit der Planung gespeichert (Standard: Ziel 1) und steht auch als Option im Kommandozeilenbefehl `optimize` zur Verfügung.

- **Ziel 1, Konsum bei begrenztem Ruinrisiko:** wie bisher (Beschreibung in der Hilfe, `app/ui/hilfe/40-optimierung.md`).
- **Ziel 2, Immobilie behalten:** Gesucht wird die Strategie mit dem geringsten Anteil an Läufen, in denen eine Immobilie ungeplant (Notverkauf) verkauft wird. Bei gleichem Anteil entscheidet die geringere Ruinwahrscheinlichkeit (Ruin wie in Ziel 1 definiert). Nebenbedingungen: Der Median des jährlichen Konsums ab Pensionierung beträgt mindestens K % der geplanten Ausgaben (Eingabe, Standard 90 %), und die Ruinwahrscheinlichkeit liegt unter der Grenze von Ziel 1 (Standard 5 %). Erfüllt keine Strategie die Nebenbedingungen, nennt das Ergebnis die am nächsten kommende und weist darauf hin.
- Die Ergebnisanzeige nennt das gewählte Ziel und bei Ziel 2 den Anteil der Läufe mit ungeplantem Verkauf je Strategie. Weitere Ziele sind nicht Teil dieser Anforderung.

**Abnahme**

- [ ] Die Optimierung hat die Auswahl «Ziel»; Standard ist Ziel 1; die Ergebnisse für Ziel 1 sind identisch mit denen vor der Änderung (Regressionstest).
- [ ] Bei Ziel 2 zeigt die Ergebnistabelle je Strategie den Anteil Läufe mit ungeplantem Immobilienverkauf; die Reihenfolge richtet sich nach diesem Anteil, bei Gleichstand nach der Ruinwahrscheinlichkeit.
- [ ] Strategien mit Konsum unter K % der geplanten Ausgaben werden nicht als beste gewählt.
- [ ] Mit `examples/Test 1.rplan` liefert Ziel 2 eine Strategie mit höchstens so vielen ungeplanten Verkäufen wie Ziel 1.
- [ ] Planungen ohne Immobilien: Ziel 2 ist nicht wählbar (mit Hinweis).
- [ ] Das Ziel steht in der Plandatei und in der Hilfe.

**Umsetzungshinweis** (nicht Teil der Anforderung): `OptimizerSettings` um `goal` und `minConsumption` erweitern; Zielfunktion in `engine/src/optimize.rs` je Ziel wählen.

> «Optimierung: Verwchieden Ziele auswählen können. 1. wie jetzt 2. Ziel Imobilie nicht ungeplannt verkaufen aber mit der geringsten Ruinwahrscheinlichkeit 3. Später folgen weitere»

_Ursprüngliche ID: FEA-19_
