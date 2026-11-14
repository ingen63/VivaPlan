Die Tabelle «Jahreswerte» hat zwei neue Spalten in Prozent: **Entnahmerate Total** und **Entnahmerate liquide**. Beide sind jährliche Entnahme geteilt durch das Vermögen am Jahresanfang, bezogen auf (a) das Gesamtvermögen (Spalte «Total») bzw. (b) das liquide Vermögen (Spalte «Liquid»).

- Jährliche Entnahme = Summe der Monatsentnahmen des Kalenderjahrs, wie in der Spalte «Entnahme» (tatsächliche Ausgaben minus Einkommen, vor Steuern). Bei unvollständigen Jahren Monatsdurchschnitt mal 12, die Zeile bleibt mit * markiert.
- Vermögen am Jahresanfang = Wert des Vorjahres in der jeweiligen Spalte, im ersten Jahr der Stand zum Simulationsstart.
- Entnahme null oder negativ: Zelle leer. Vermögen null oder negativ: «–».
- Berechnung immer aus nominalen Werten, unabhängig von der Ansicht nominal/real. Die Tabellenhilfe erklärt beide Formeln.

**Abnahme**

- [ ] Entnahme 40'000 pro Jahr bei Total 1'000'000 und Liquid 500'000 am Jahresanfang ergibt 4.0 % bzw. 8.0 %.
- [ ] Im ersten Jahr einer im November startenden Simulation: Entnahme der zwei Monate mal 6, geteilt durch das Startvermögen.
- [ ] Jahre mit Einkommen über den Ausgaben zeigen leere Zellen.
- [ ] Wechsel nominal/real ändert die Werte nicht.
- [ ] Jede Zeile stimmt mit der manuellen Rechnung aus den Spalten «Entnahme», «Total» und «Liquid» (Test mit `examples/Test 1.rplan`).

**Umsetzungshinweis** (nicht Teil der Anforderung): reine Oberfläche in `yearTable`, die Engine liefert alle Werte.

> «Ergebnisse -> Jahreswerte. Entnahmerate als Spalte hinzufügen (Entnahmerate = Entnahme/Vermögen Total)  »

---
Umsetzung laut Git-Log: dd4b187. Abnahmekriterien vor dem Schliessen prüfen.

_Ursprüngliche ID: FEA-06_
