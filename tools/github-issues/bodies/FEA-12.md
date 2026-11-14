In der Liste «Gespeicherte Ergebnisse» zeigt die Spalte «Ergebnis» bei Monte-Carlo-Läufen «Erfolgsquote · 5-%-Vermögen» (5. Perzentil des Endvermögens) statt des Medians; der Median bleibt im Tooltip der Zelle abrufbar. Einzelläufe zeigen weiterhin das Endvermögen, Szenarien wie bisher «–».

- Warnsymbol (Haus mit Warnzeichen): Es erscheint, wenn es zu einem **Notverkauf einer Immobilie** kam, so wie die Engine ihn zählt. Das schliesst Verkäufe wegen fehlender Tragbarkeit bei der Hypothekenerneuerung ein, geplante Verkäufe aber nicht. Bei Monte-Carlo-Läufen erscheint es, wenn der Anteil der Läufe mit Notverkauf mindestens die **Schwelle** erreicht; der Tooltip nennt den Anteil («in 12 % der Läufe, 6 von 50 Läufen»). Die Schwelle ist in den Einstellungen konfigurierbar (immer, 1 %, 5 % usw., siehe FEA-24); bis dahin gilt fest **5 %**. Bei Einzelläufen erscheint das Symbol bei jedem Notverkauf (Schwelle gilt nicht). Bei Szenarien erscheint vorerst kein Symbol, weil die Szenarienauswertung keine Notverkäufe je Variante liefert. Planungen ohne Immobilien zeigen nie ein Symbol. Alte Ergebnisse ohne diese Angabe zeigen weder Symbol noch Hinweis.

**Abnahme**

- [ ] Ein Monte-Carlo-Ergebnis zeigt «Erfolgsquote · 5 %: Betrag», der Betrag entspricht dem 5. Perzentil der Ergebnistabelle der Übersicht; der Median steht im Tooltip.
- [ ] Mit `examples/Test 1.rplan` und einem Notverkauf in mindestens 5 % der Läufe erscheint das Symbol, der Tooltip nennt Anteil und Anzahl Läufe.
- [ ] Bei weniger als 5 % Läufen mit Notverkauf erscheint kein Symbol.
- [ ] Eine Planung ohne Notverkauf oder ohne Immobilien zeigt kein Symbol.
- [ ] Ein geplanter Verkauf löst das Symbol nicht aus.
- [ ] Ein Einzellauf mit Notverkauf zeigt das Symbol.
- [ ] Alte Einträge ohne Angabe werden fehlerfrei und ohne Symbol angezeigt.
- [ ] Wird in den Einstellungen (FEA-24) eine andere Schwelle gewählt, gilt sie.
- Abhängigkeit: FEA-24 (Einstellungen) für die konfigurierbare Schwelle.

**Umsetzungshinweis** (nicht Teil der Anforderung): `emergency_sale_runs` und Anzahl Läufe bei der Speicherung in die Kopfdaten übernehmen.

> «Gespeicherte Ergebnisse: Anstatt Median Vermögen 5% Vermögen, sowie Icon ob es zu einem ungeplanten Immoblienverkauf kahn. Icon Haus + Ausrufezeichen/Warndreieck»

_Ursprüngliche ID: FEA-12_
