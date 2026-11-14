Die Liquiditätsreserve ist kein Parameter von «Vermögen & Anlage» mehr, sondern ein Parameter der Strategie «Standard» (Karte «Standard» der Entnahmestrategie, als Zeitreihe wie bisher). Für alle anderen Strategien (Guardrails, feste Rate, VPW, Endowment) und für 3-Topf vor der Pensionierung gilt eine feste, nicht einstellbare Reserve von 3 Monaten; die Hilfe nennt diese Annahme.

- Migration: Eine bestehende Planung behält ihren Wert für «Standard». Planungen mit anderer Strategie und einer Reserve ungleich 3 Monaten rechnen künftig mit 3 Monaten und können dadurch leicht andere Ergebnisse liefern. Beim Laden erscheint dazu ein einmaliger Hinweis; die Änderung steht in den Release Notes.

**Abnahme**

- [ ] «Vermögen & Anlage» enthält die Reserve nicht mehr, die Karte «Standard» enthält sie.
- [ ] Eine Planung mit Strategie Standard und Reserve 6 liefert dieselben Ergebnisse wie vor der Änderung (Regressionstest mit den Golden-Dateien).
- [ ] Eine Planung mit Guardrails und Reserve 6 rechnet mit 3 Monaten, der Hinweis erscheint beim Laden einmal.
- [ ] Die Optimierung bietet die Reserve weiterhin nur für Standard an.
- [ ] Hilfe, Beschreibung und Tests sind nachgeführt.
- Hinweis: Betrifft FEA-05 (zurückgestellt): Bei einer künftigen Trennung von Ausgabenregel und Anlage ist die Reserve neu zu klären.

**Umsetzungshinweis** (nicht Teil der Anforderung): Die Plandatei behält `wealth.reserveMonths`; die Engine wendet es nur noch bei Standard an, sonst fest 3.

> «Liquiditätsreserve als Parameter dort wegnehmen und nur für die  Entnahmestrategie Standard verwenden»

_Ursprüngliche ID: FEA-17_
