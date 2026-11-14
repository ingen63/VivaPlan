In der Liste «Gespeicherte Ergebnisse» und in der Vergleichsansicht zeigt die Spalte «Strategie» bei Mausberührung und bei Tastaturfokus die Parameter der verwendeten Strategie als einfachen Tooltip, mit deutschen Bezeichnungen und Einheiten wie im Editor. Pro Strategie erscheinen nur ihre eigenen Parameter: Standard (Aktienanteil, Liquiditätsreserve), 3-Topf (Jahre Topf 1, Jahre Topf 2, Renditeschwelle, Aktienanteil des Restvermögens falls vorhanden), Guardrails (Startrate, obere und untere Leitplanke, Anpassung, Untergrenze, Obergrenze, Aktienanteil), Feste Rate (Rate, Aktienanteil), VPW (Planungsalter, Realrendite, Aktienanteil), Endowment (Rate, Glättung, Aktienanteil). Die Kurzform der Parameter wird beim Speichern eines Ergebnisses in den Kopfdaten abgelegt; bei bereits gespeicherten Ergebnissen ohne diese Kurzform steht «Parameter nicht verfügbar».

**Abnahme**

- [ ] Mouseover über «3-Topf» zeigt Jahre Topf 1 und 2 und die Renditeschwelle mit den Werten des Ergebnisses (nicht der aktuellen Planung).
- [ ] Jede der sechs Strategien zeigt genau ihre Parameter.
- [ ] Die Werte stimmen mit dem Reiter «Parameter» (FEA-03) überein.
- [ ] Der Tooltip ist per Tastaturfokus erreichbar.
- [ ] Ein altes Ergebnis ohne Kurzform zeigt «Parameter nicht verfügbar», ohne Fehler.
- [ ] Die Liste lädt nicht langsamer (Ergebnisdaten werden dafür nicht geladen).
- Abhängigkeit: FEA-03 (gleiche Bezeichnungen und Werte).

**Umsetzungshinweis** (nicht Teil der Anforderung): Kurzform in `runMeta` ergänzen (`app/ui/app.js`).

> «Gespeicherte Ergebnisse: MouseOver über Strategie zeigt die Parameter»

---
Umsetzung laut Git-Log: dd4b187. Abnahmekriterien vor dem Schliessen prüfen.

_Ursprüngliche ID: FEA-10_
