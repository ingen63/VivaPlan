Die Karte «Ereignisse» in den Ergebnissen (Übersicht und Einzelläufe) enthält keine Meldungen der 3-Topf-Buchhaltung: keine Auffüllungen (Januar, A einschliesslich Start der Entnahmephase, B, C1, C2), keine Zuflüsse («extern → Topf 3», «extern → Reservierung») und keine Reservierungsbuchungen (zum Beispiel «Verkaufserlös … reserviert für Kauf im Folgemonat»). Alle anderen Ereignisse bleiben unverändert (Pensionierung, Kapitalbezug, Käufe, Verkäufe, Notverkäufe, Hypotheken, Entnahmestrategie, Leitplanken, Geld aufgebraucht) und ebenso die Meldungen zur Defizitphase der 3-Topf-Strategie (Warnzeichen, siehe Abschnitt 9 der 3-Topf-Spezifikation). Die Ereignisse werden weiterhin erzeugt und gespeichert, nur die Anzeige in dieser Karte filtert sie. Die 3-Topf-Angaben sind über die Herleitung in den Jahreswerten abrufbar (FEA-08). Die Zeitleiste im Editor, die Marken in den Grafiken und die Vorschau-Hinweise im Editor bleiben unverändert.

**Abnahme**

- [ ] Ein 3-Topf-Lauf zeigt in der Karte «Ereignisse» keine Zeile zu Auffüllung, Zufluss oder Reservierung.
- [ ] Pensionierung, Kapitalbezug, Käufe, Verkäufe, Notverkäufe, Defizitphase und Geld aufgebraucht stehen weiterhin in der Liste.
- [ ] Ein Lauf ohne 3-Topf-Strategie zeigt dieselbe Liste wie bisher.
- [ ] Jeder reservierte Verkaufserlös ist in den Jahreswerten unter der Herleitung (FEA-08) zu finden.
- [ ] Zeitleiste und Grafikmarken sind unverändert.
- [ ] Der UI-Test, der 3-Topf-Ereignisse in der Liste erwartet, ist angepasst.
- Abhängigkeit: FEA-08.

**Umsetzungshinweis** (nicht Teil der Anforderung): Filtern in `eventList` nach Ereignisart `buckets`; die Reservierungsmeldung bekommt ebenfalls diese Art oder wird beim Filtern miterfasst; die Defizitphase-Meldungen dürfen nicht unter den Filter fallen.

> «Ergebnisse: Kein 3 Topf Events in der Liste der Ereignisse»

---
Umsetzung laut Git-Log: dd4b187. Abnahmekriterien vor dem Schliessen prüfen.

_Ursprüngliche ID: FEA-09_
